use std::{
    collections::HashMap,
    ffi::{c_char, c_void},
};

use unleash_yggdrasil::{Context, VariantDef};

use crate::{get_engine, FFIError};

const STRATEGIES_ENTRY_SIZE: usize = 5;
const PROPERTIES_ENTRY_SIZE: usize = 8;

#[repr(C)]
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
    pub impression_data: u8,
    pub error: *mut c_char,
}

#[repr(C)]
pub struct VariantResponse {
    pub feature_enabled: u8,
    pub is_enabled: u8,
    pub variant_name: *mut c_char,
    pub payload_type: *mut c_char,
    pub payload_value: *mut c_char,
    pub impression_data: u8,
    pub error: *mut c_char,
}

unsafe fn get_header(buffer: &[u8]) -> &MessageHeader {
    &*(buffer.as_ptr() as *const MessageHeader)
}

struct Message {
    toggle_name: String,
    context: Context,
    custom_strategy_results: Option<HashMap<String, bool>>,
    metrics_request: ToggleMetricRequest,
    default_variant_name: Option<String>,
}

#[inline(always)]
fn unpack_message(buffer: &[u8]) -> Result<Message, FFIError> {
    if buffer.len() < std::mem::size_of::<MessageHeader>() {
        return Err(FFIError::InvalidMessageFormat);
    }

    let header: &MessageHeader = unsafe { get_header(buffer) };

    #[inline(always)]
    fn get_string(offset: u32, data: &[u8]) -> Option<&str> {
        if offset == 0 {
            return None;
        }
        let start = offset as usize;
        let end = data[start..].iter().position(|&b| b == 0).unwrap() + start;
        Some(unsafe { std::str::from_utf8_unchecked(&data[start..end]) }) // from_utf8_lossy allocates, this doesn't
    }

    let toggle_name = get_string(header.toggle_name_offset, buffer)
        .unwrap()
        .to_string();
    let default_variant_name =
        get_string(header.default_variant_name_offset, buffer).map(ToString::to_string);

    let mut properties = (header.properties_count > 0).then(HashMap::new);

    if let Some(props) = properties.as_mut() {
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

            if let (Some(key), Some(value)) = (
                get_string(key_offset as u32, buffer),
                get_string(value_offset as u32, buffer),
            ) {
                props.insert(key.to_string(), value.to_string());
            }
        }
    }

    let mut custom_strategy_results = (header.custom_strategies_count > 0).then(HashMap::new);

    if let Some(strategies) = custom_strategy_results.as_mut() {
        let strategies_table = &buffer[header.custom_strategies_offset as usize..];
        for i in 0..header.custom_strategies_count as usize {
            let entry_offset = i * STRATEGIES_ENTRY_SIZE;

            let key_offset = u32::from_le_bytes([
                strategies_table[entry_offset],
                strategies_table[entry_offset + 1],
                strategies_table[entry_offset + 2],
                strategies_table[entry_offset + 3],
            ]) as usize;

            if let Some(key) = get_string(key_offset as u32, buffer) {
                let value = strategies_table[entry_offset + std::mem::size_of::<u32>()] != 0;
                strategies.insert(key.to_string(), value);
            }
        }
    }

    let context = Context {
        user_id: get_string(header.user_id_offset, buffer).map(ToString::to_string),
        session_id: get_string(header.session_id_offset, buffer).map(ToString::to_string),
        remote_address: get_string(header.remote_address_offset, buffer).map(ToString::to_string),
        environment: get_string(header.environment_offset, buffer).map(ToString::to_string),
        app_name: get_string(header.app_name_offset, buffer).map(ToString::to_string),
        current_time: get_string(header.current_time_offset, buffer).map(ToString::to_string),
        properties,
    };

    let toggle_metrics = header.metric_request;

    Ok(Message {
        toggle_name,
        context,
        custom_strategy_results,
        metrics_request: toggle_metrics,
        default_variant_name,
    })
}

/// # Safety
///
/// The caller **must** ensure the following conditions are met:
///
/// ## `engine_ptr` Must Be a Valid Pointer
/// - `engine_ptr` **must point to a valid `Engine` instance**.
/// - Passing a null or dangling pointer results in **undefined behavior (UB)**.
/// - The caller **must ensure `engine_ptr` remains valid** while this function executes.
///
/// ## `message_ptr` Must Be Non-Null and `message_len` Must Be Valid
/// - If `message_ptr` is null or `message_len == 0`, the function returns an error (`FFIError::NullError`).
/// - The function **creates a byte slice** using `std::slice::from_raw_parts(message_ptr, message_len)`, which:
///   - **Assumes `message_len` is accurate** (overreading causes UB).
///   - **Assumes `message_ptr` points to a valid, readable memory region** of at least `message_len` bytes.
/// - Ensure `message_ptr` points to a properly formatted buffer that is **alive and not mutated** during execution.
///
/// ## `VariantResponse.error` and CString Fields Must Be Freed by the Caller
/// - If an error occurs, `error` is allocated using `CString::into_raw()` and **must be freed by the caller**.
/// - The following fields are also allocated with `CString::into_raw()` and **must be freed by the caller**:
///   - `variant_name`
///   - `payload_type`
///   - `payload_value`
/// - **Caller Responsibility:** If any of these fields are non-null, free them using `free_variant_response(response)`.
///
/// ## Integer Representation and Enum Consistency
/// - `ToggleMetricRequest` is a `#[repr(u8)]` enum, ensuring it fits within a single byte.
/// - `feature_enabled`, `is_enabled`, and `impression_data` are also `u8`, avoiding alignment mismatches.
/// - The caller **must ensure valid numeric values** are passed for `ToggleMetricRequest`.
#[no_mangle]
pub unsafe extern "C" fn one_shot_get_variant(
    engine_ptr: *mut c_void,
    message_ptr: *const u8,
    message_len: usize,
) -> VariantResponse {
    let result: Result<(bool, bool, Option<VariantDef>), FFIError> = (|| {
        let engine = get_engine(engine_ptr)?;

        if message_ptr.is_null() || message_len == 0 {
            return Err(FFIError::NullError);
        }
        let message = std::slice::from_raw_parts(message_ptr, message_len);
        let Message {
            toggle_name,
            context,
            custom_strategy_results,
            metrics_request,
            default_variant_name,
        } = unpack_message(message)?;

        let enabled = engine.check_enabled(&toggle_name, &context, &custom_strategy_results);

        let Some(enabled) = enabled else {
            if metrics_request == ToggleMetricRequest::Always {
                engine.count_toggle(&toggle_name, false);
            }
            return Ok((false, false, None));
        };

        let variant = engine.check_variant(&toggle_name, &context, &custom_strategy_results);
        let impression_data = engine.should_emit_impression_event(&toggle_name);

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
        Ok((enabled, impression_data, variant))
    })();

    match result {
        Ok((enabled, impression_data, Some(variant))) => {
            let (payload_type, payload_value) = if let Some(payload) = &variant.payload {
                let payload_type = std::ffi::CString::new(payload.payload_type.clone())
                    .unwrap()
                    .into_raw();
                let payload_value = std::ffi::CString::new(payload.value.clone())
                    .unwrap()
                    .into_raw();
                (payload_type, payload_value)
            } else {
                (std::ptr::null_mut(), std::ptr::null_mut())
            };

            VariantResponse {
                error: std::ptr::null_mut(),
                feature_enabled: enabled as u8,
                is_enabled: variant.enabled as u8,
                variant_name: std::ffi::CString::new(variant.name).unwrap().into_raw(),
                impression_data: impression_data as u8,
                payload_type,
                payload_value,
            }
        }
        Ok((enabled, impression_data, None)) => VariantResponse {
            error: std::ptr::null_mut(),
            feature_enabled: enabled as u8,
            is_enabled: false as u8,
            variant_name: std::ptr::null_mut(),
            impression_data: impression_data as u8,
            payload_type: std::ptr::null_mut(),
            payload_value: std::ptr::null_mut(),
        },
        Err(e) => VariantResponse {
            error: std::ffi::CString::new(e.to_string()).unwrap().into_raw(),
            feature_enabled: false as u8,
            is_enabled: false as u8,
            variant_name: std::ptr::null_mut(),
            impression_data: false as u8,
            payload_type: std::ptr::null_mut(),
            payload_value: std::ptr::null_mut(),
        },
    }
}

/// # Safety
///
/// The caller **must** ensure the following conditions are met:
///
/// `engine_ptr` Must be a valid pointer, passing a null or dangling pointer results in undefined behavior (UB)
///  Ensure `engine_ptr` was obtained from a valid `Engine`
///  and is not used after being freed.
///
/// If `message_ptr` is null or `message_len == 0`, the function returns an error (`FFIError::NullError`).
/// The function **creates a byte slice** using `std::slice::from_raw_parts(message_ptr, message_len)`, which:
/// **Assumes `message_len` is accurate** (overreading causes UB).
/// **Assumes `message_ptr` points to a valid, readable memory region** of at least `message_len` bytes.
/// Ensure `message_ptr` points to a properly formatted buffer that is
/// **alive and not mutated** while this function executes.
///
/// `EnabledResponse.error` must be freed by the caller by invoking `free_enabled_response(response)`.
///
/// ## Integer Representation and Enum Consistency
/// - `ToggleMetricRequest` is a `#[repr(u8)]` enum, ensuring it fits within a single byte.
/// - `value` (boolean state) and `impression_data` are also `u8`, avoiding alignment mismatches.
/// - It's the callers responsibility to ensure that bytes sent only have their significant bits set.
/// - The enum should match the Rust enum (`0 = Always`, `1 = IfExists`, `2 = None`).
#[no_mangle]
pub unsafe extern "C" fn one_shot_is_enabled(
    engine_ptr: *mut c_void,
    message_ptr: *const u8,
    message_len: usize,
) -> EnabledResponse {
    let result: Result<(Option<bool>, bool), FFIError> = (|| {
        let engine = get_engine(engine_ptr)?;

        if message_ptr.is_null() || message_len == 0 {
            return Err(FFIError::NullError);
        }
        let message = std::slice::from_raw_parts(message_ptr, message_len);
        let Message {
            toggle_name,
            context,
            custom_strategy_results,
            metrics_request,
            default_variant_name: _,
        } = unpack_message(message)?;

        let enabled = engine.check_enabled(&toggle_name, &context, &custom_strategy_results);
        let impression_data = engine.should_emit_impression_event(&toggle_name);

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

        Ok((enabled, impression_data))
    })();

    match result {
        Ok((Some(value), impression_data)) => EnabledResponse {
            value: value as u8,
            impression_data: impression_data as u8,
            error: std::ptr::null_mut(),
        },
        Ok((None, impression_data)) => EnabledResponse {
            value: 2,
            impression_data: impression_data as u8,
            error: std::ptr::null_mut(),
        },
        Err(e) => EnabledResponse {
            value: 3,
            impression_data: false as u8,
            error: std::ffi::CString::new(e.to_string()).unwrap().into_raw(),
        },
    }
}

/// # Safety
///
/// - This function **must only be called on a valid `EnabledResponse`** returned from `one_shot_is_enabled`.
/// - **`response` must be a non-null, valid pointer.**
///   - If `response` is null, this function does nothing.
///   - Passing a dangling or already-freed pointer results in **undefined behavior (UB)**.
///
/// ## `error` Must Be Freed Only Once
/// - If `error` is non-null, this function **takes ownership and frees it**.
/// - The pointer is then set to `null_mut()` to prevent double frees.
/// - **Caller Responsibility:** Never call this function more than once on the same `EnabledResponse`,
///   and do not manually free `error` after calling this function.
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

/// # Safety
///
/// - This function **must only be called on a valid `VariantResponse`** returned from `one_shot_get_variant`.
/// - **`response` must be a non-null, valid pointer.**
///   - If `response` is null, this function does nothing.
///   - Passing a dangling or already-freed pointer results in **undefined behavior (UB)**.
///
/// ## CString Fields Must Be Freed Only Once
/// - If any of the following fields are non-null, this function **takes ownership and frees them**:
///   - `error`
///   - `variant_name`
///   - `payload_type`
///   - `payload_value`
/// - The pointers are then set to `null_mut()` to prevent double frees.
///
/// ## Caller Responsibilities
/// - **Never call this function more than once on the same `VariantResponse`.**
/// - **Do not manually free individual fields after calling this function.**
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

    if !response.payload_type.is_null() {
        drop(std::ffi::CString::from_raw(response.payload_type));
        response.payload_type = std::ptr::null_mut();
    }

    if !response.payload_value.is_null() {
        drop(std::ffi::CString::from_raw(response.payload_value));
        response.payload_value = std::ptr::null_mut();
    }
}
