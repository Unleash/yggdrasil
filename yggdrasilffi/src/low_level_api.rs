use std::{
    collections::HashMap,
    ffi::{c_char, c_void},
};

use unleash_yggdrasil::{Context, VariantDef};

use crate::{get_engine, FFIError};

const STRATEGIES_ENTRY_SIZE: usize = 5;
const PROPERTIES_ENTRY_SIZE: usize = 8;

#[repr(C, packed)]
#[derive(Debug)]
pub struct MessageHeader {
    toggle_name_offset: u32,
    user_id_offset: u32,
    session_id_offset: u32,
    remote_address_offset: u32,
    environment_offset: u32,
    current_time_offset: u32,
    app_name_offset: u32,
    default_variant_name_offset: u32,
    properties_offset: u32,
    properties_count: u32,
    custom_strategies_offset: u32,
    custom_strategies_count: u32,
    pub metric_request: ToggleMetricRequest,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToggleMetricRequest {
    Always = 0,
    IfExists = 1,
    None = 2,
}

#[repr(C)]
pub struct EnabledResponse {
    pub value: u8,
    pub error: *mut c_char,
}

#[repr(C)]
pub struct VariantResponse {
    pub feature_enabled: u8,
    pub is_enabled: u8,
    pub variant_name: *mut c_char,
    pub error: *mut c_char,
}

unsafe fn get_header(buffer: &[u8]) -> &MessageHeader {
    &*(buffer.as_ptr() as *const MessageHeader)
}

fn unpack_message(
    buffer: &[u8],
) -> Result<
    (
        String,
        Context,
        Option<HashMap<String, bool>>,
        ToggleMetricRequest,
        Option<String>,
    ),
    FFIError,
> {
    if buffer.len() < std::mem::size_of::<MessageHeader>() {
        return Err(FFIError::InvalidMessageFormat);
    }

    let header: &MessageHeader = unsafe { get_header(buffer) };

    // Tear out a chunk of the buffer and convert it to an owned string
    // we could probably optimize this by returning a &str but that means
    // making the context lifetime be bounded by this buffers lifetime
    fn get_string(offset: u32, data: &[u8]) -> Option<String> {
        if offset == 0 {
            return None;
        }
        let start = offset as usize;
        let end = data[start..].iter().position(|&b| b == 0).unwrap() + start;
        Some(String::from_utf8_lossy(&data[start..end]).to_string())
    }

    let toggle_name = get_string(header.toggle_name_offset, buffer).unwrap();
    let default_variant_name = if header.default_variant_name_offset != 0 {
        Some(get_string(header.default_variant_name_offset, buffer).unwrap())
    } else {
        None
    };

    let properties = if header.properties_count > 0 {
        let mut properties = std::collections::HashMap::new();

        let properties_table = &buffer[header.properties_offset as usize..];
        for i in 0..header.properties_count as usize {
            let entry_offset = i * PROPERTIES_ENTRY_SIZE;

            let key_offset = u32::from_le_bytes([
                properties_table[entry_offset],
                properties_table[entry_offset + 1],
                properties_table[entry_offset + 2],
                properties_table[entry_offset + 3],
            ]) as usize;

            let value_offset = u32::from_le_bytes([
                properties_table[entry_offset + 4],
                properties_table[entry_offset + 5],
                properties_table[entry_offset + 6],
                properties_table[entry_offset + 7],
            ]) as usize;

            // value can be null, if its null it has an offset of 0, in which case
            // we can ignore this pair from the props table
            if value_offset == 0 {
                continue;
            }
            let key = get_string((key_offset as usize).try_into().unwrap(), buffer).unwrap();
            let value = get_string((value_offset as usize).try_into().unwrap(), buffer).unwrap();
            properties.insert(key, value);
        }
        Some(properties)
    } else {
        None
    };

    let custom_strategy_results = if header.custom_strategies_count > 0 {
        let mut custom_strategies = std::collections::HashMap::new();

        let strategies_table = &buffer[header.custom_strategies_offset as usize..];

        for i in 0..header.custom_strategies_count as usize {
            let entry_offset = i * STRATEGIES_ENTRY_SIZE;

            let key_offset = u32::from_le_bytes([
                strategies_table[entry_offset],
                strategies_table[entry_offset + 1],
                strategies_table[entry_offset + 2],
                strategies_table[entry_offset + 3],
            ]) as usize;

            let key = get_string((key_offset as usize).try_into().unwrap(), buffer).unwrap();

            let value = strategies_table[entry_offset + std::mem::size_of::<u32>()] != 0;
            custom_strategies.insert(key, value);
        }

        Some(custom_strategies)
    } else {
        None
    };

    let context = Context {
        user_id: get_string(header.user_id_offset, buffer),
        session_id: get_string(header.session_id_offset, buffer),
        remote_address: get_string(header.remote_address_offset, buffer),
        environment: get_string(header.environment_offset, buffer),
        app_name: get_string(header.app_name_offset, buffer),
        current_time: get_string(header.current_time_offset, buffer),
        properties: properties,
    };

    let toggle_metrics = header.metric_request;

    Ok((
        toggle_name,
        context,
        custom_strategy_results,
        toggle_metrics,
        default_variant_name,
    ))
}

#[no_mangle]
pub unsafe extern "C" fn quick_get_variant(
    engine_ptr: *mut c_void,
    message_ptr: *const u8,
    message_len: usize,
) -> VariantResponse {
    let result: Result<(bool, Option<VariantDef>), FFIError> = (|| {
        let engine = get_engine(engine_ptr)?;

        if message_ptr.is_null() || message_len == 0 {
            return Err(FFIError::Utf8Error); //wrong error for now
        }
        let message = std::slice::from_raw_parts(message_ptr, message_len);
        let (toggle_name, context, custom_strategy_results, metrics_request, default_variant_name) =
            unpack_message(message)?;

        let enabled = engine.check_enabled(&toggle_name, &context, &custom_strategy_results);

        let Some(enabled) = enabled else {
            if metrics_request == ToggleMetricRequest::Always {
                engine.count_toggle(&toggle_name, false);
            }
            engine.count_variant(
                &toggle_name,
                &default_variant_name.unwrap_or("disabled".to_string()),
            );
            return Ok((false, None));
        };

        let variant = engine.check_variant(&toggle_name, &context, &custom_strategy_results);

        match &variant {
            Some(variant) => {
                engine.count_variant(&toggle_name, &variant.name);
            }
            None => {
                engine.count_variant(
                    &toggle_name,
                    &default_variant_name.unwrap_or("disabled".to_string()),
                );
            }
        };
        if metrics_request == ToggleMetricRequest::Always
            || metrics_request == ToggleMetricRequest::IfExists
        {
            engine.count_toggle(&toggle_name, enabled);
        }
        Ok((enabled, variant))
    })();

    match result {
        Ok((enabled, Some(variant))) => {
            VariantResponse {
            error: std::ptr::null_mut(),
            feature_enabled: enabled as u8,
            is_enabled: variant.enabled as u8,
            variant_name: std::ffi::CString::new(variant.name).unwrap().into_raw(),
        }},
        Ok((enabled, None)) => {
            VariantResponse {
            error: std::ptr::null_mut(),
            feature_enabled: enabled as u8,
            is_enabled: false as u8,
            variant_name: std::ptr::null_mut(),
        }},
        Err(e) => VariantResponse {
            error: std::ffi::CString::new(e.to_string()).unwrap().into_raw(),
            feature_enabled: false as u8,
            is_enabled: false as u8,
            variant_name: std::ptr::null_mut(),
        },
    }
}

#[no_mangle]
pub unsafe extern "C" fn quick_check(
    engine_ptr: *mut c_void,
    message_ptr: *const u8,
    message_len: usize,
) -> EnabledResponse {
    let result: Result<Option<bool>, FFIError> = (|| {
        let engine = get_engine(engine_ptr)?;

        if message_ptr.is_null() || message_len == 0 {
            return Err(FFIError::Utf8Error); //wrong error for now
        }
        let message = std::slice::from_raw_parts(message_ptr, message_len);
        let (toggle_name, context, custom_strategy_results, metrics_request, _) =
            unpack_message(message)?;

        let enabled = engine.check_enabled(&toggle_name, &context, &custom_strategy_results);

        match enabled {
            Some(enabled) => {
                if metrics_request == ToggleMetricRequest::Always
                    || metrics_request == ToggleMetricRequest::IfExists
                {
                    engine.count_toggle(&toggle_name, enabled);
                }
            }
            None => {
                if metrics_request == ToggleMetricRequest::Always {
                    engine.count_toggle(&toggle_name, false);
                }
            }
        };

        Ok(enabled)
    })();

    match result {
        Ok(Some(value)) => EnabledResponse {
            value: value as u8,
            error: std::ptr::null_mut(),
        },
        Ok(None) => EnabledResponse {
            value: 2,
            error: std::ptr::null_mut(),
        },
        Err(e) => EnabledResponse {
            value: 3,
            error: std::ffi::CString::new(e.to_string()).unwrap().into_raw(),
        },
    }
}

#[no_mangle]
pub unsafe extern "C" fn free_enabled_response(response: *mut EnabledResponse) {
    if response.is_null() {
        return;
    }

    let response = &mut *response;

    if !response.error.is_null() {
        drop(std::ffi::CString::from_raw(response.error));
        response.error = std::ptr::null_mut();
    }
}

#[no_mangle]
pub unsafe extern "C" fn free_variant_response(response: *mut VariantResponse) {
    if response.is_null() {
        return;
    }

    let response = &mut *response;

    if !response.error.is_null() {
        drop(std::ffi::CString::from_raw(response.error));
        response.error = std::ptr::null_mut();
    }

    if !response.variant_name.is_null() {
        drop(std::ffi::CString::from_raw(response.variant_name));
        response.variant_name = std::ptr::null_mut();
    }
}
