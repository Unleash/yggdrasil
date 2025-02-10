use std::{
    collections::HashMap,
    ffi::{c_char, c_void},
};

use unleash_yggdrasil::Context;

use crate::{get_engine, FFIError};

#[repr(C, packed)]
#[derive(Debug)]
pub struct MessageHeader {
    toggle_name_offset: u32,
    user_id_offset: u32,
    session_id_offset: u32,
    remote_address_offset: u32,
    environment_offset: u32,
    app_name_offset: u32,
    properties_offset: u32,
    properties_count: u32,
    custom_strategies_offset: u32,
    custom_strategies_count: u32,
}

#[repr(C)]
pub struct EnabledResponse {
    pub value: u8,
    pub error: *mut c_char,
}

unsafe fn get_header(buffer: &[u8]) -> &MessageHeader {
    &*(buffer.as_ptr() as *const MessageHeader)
}

unsafe fn get_properties_table(
    buffer: &[u8],
    properties_offset: usize,
    properties_count: u32,
) -> &[u32] {
    std::slice::from_raw_parts(
        buffer.as_ptr().add(properties_offset) as *const u32,
        (properties_count * 2) as usize,
    )
}

unsafe fn get_strategies_table(
    buffer: &[u8],
    custom_strategies_offset: usize,
    custom_strategies_count: u32,
) -> &[u32] {
    std::slice::from_raw_parts(
        buffer.as_ptr().add(custom_strategies_offset) as *const u32,
        custom_strategies_count as usize,
    )
}

fn unpack_message(
    buffer: &[u8],
) -> Result<(String, Context, Option<HashMap<String, bool>>), FFIError> {
    if buffer.len() < std::mem::size_of::<MessageHeader>() {
        return Err(FFIError::InvalidMessageFormat);
    }

    let header: &MessageHeader = unsafe { get_header(buffer) };

    // Tear out a chunk of the buffer and convert it to an owned string
    // we could probably optimize this by returning a &str but that means
    // making the context lifetime be bounded by this buffer's lifetime
    fn get_string(offset: u32, data: &[u8]) -> Option<String> {
        if offset == 0 {
            return None;
        }
        let start = offset as usize;
        let end = data[start..].iter().position(|&b| b == 0).unwrap() + start;
        Some(String::from_utf8_lossy(&data[start..end]).to_string())
    }

    let toggle_name = get_string(header.toggle_name_offset, buffer).unwrap();

    let properties = if header.properties_count > 0 {
        // let mut properties = std::collections::HashMap::new();
        // let properties_offset = header.properties_offset as usize;
        // let properties_table =
        //     unsafe { get_properties_table(buffer, properties_offset, header.properties_count) };

        // for i in (0..properties_table.len()).step_by(2) {
        //     let key = get_string(properties_table[i], buffer).unwrap();
        //     let value = get_string(properties_table[i + 1], buffer).unwrap();
        //     properties.insert(key, value);
        // }
        // Some(properties)
        None
    } else {
        None
    };

    let custom_strategy_results = if header.custom_strategies_count > 0 {
        let mut custom_strategies = std::collections::HashMap::new();
        let strategies_offset = header.custom_strategies_offset as usize;
        let strategies_table = unsafe {
            get_strategies_table(buffer, strategies_offset, header.custom_strategies_count)
        };

        for i in (0..header.custom_strategies_count as usize).step_by(2) {
            let key = get_string(strategies_table[i], buffer).unwrap();
            let value = buffer[strategies_table[i + 1] as usize] != 0;
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
        current_time: None,
        properties: properties,
    };

    Ok((toggle_name, context, custom_strategy_results))
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
        let (toggle_name, context, custom_strategy_results) = unpack_message(message)?;

        Ok(engine.check_enabled(&toggle_name, &context, &custom_strategy_results))
    })();

    match result {
        Ok(Some(value)) => {
            EnabledResponse {
            value: value as u8,
            error: std::ptr::null_mut(),
        }},
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
