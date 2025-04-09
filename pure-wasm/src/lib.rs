use core::str;
use std::{
    ffi::{CString, c_char, c_void},
    mem, slice,
};

use serde::{Deserialize, Serialize};
use unleash_yggdrasil::{Context, EngineState};

use getrandom::register_custom_getrandom;

register_custom_getrandom!(get_random_source);

#[unsafe(no_mangle)]
pub fn get_random() -> i32 {
    rand::random::<i32>()
}

#[derive(Serialize, Deserialize, PartialEq, Eq)]
enum ResponseCode {
    Error = -2,
    NotFound = -1,
    Ok = 1,
}

#[derive(Serialize, Deserialize)]
struct Response<T> {
    status_code: ResponseCode,
    value: Option<T>,
    error_message: Option<String>,
}

//This is expected to be defined by the caller and passed to the WASM layer
unsafe extern "C" {
    fn fill_random(ptr: *mut u8, len: usize) -> i32;
}

fn get_random_source(buf: &mut [u8]) -> Result<(), getrandom::Error> {
    let result = unsafe { fill_random(buf.as_mut_ptr(), buf.len()) };
    if result == 0 {
        Ok(())
    } else {
        // probably the wrong error code here, this may need a custom definition
        // good enough for a spike
        Err(getrandom::Error::UNEXPECTED)
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn alloc(len: i32) -> *const u8 {
    let mut buf = Vec::with_capacity(len as usize);
    let ptr = buf.as_mut_ptr();
    // needs to be paired with dealloc, otherwise something leaks, just not sure what yet
    mem::forget(buf);
    ptr
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn dealloc(ptr: &mut u8, len: i32) {
    unsafe { Vec::from_raw_parts(ptr, 0, len as usize) };
}

#[unsafe(no_mangle)]
pub fn new_engine() -> *mut c_void {
    // need to hydrate this from the caller otherwise the metrics will be off
    // doesn't matter for a spike though
    let engine = EngineState::initial_state("2022-01-25T12:00:00.000Z".parse().unwrap());
    Box::into_raw(Box::new(engine)) as *mut c_void
}

unsafe fn materialize_string<'a>(ptr: i32, len: i32) -> &'a str {
    unsafe {
        let bytes = slice::from_raw_parts(ptr as *const u8, len as usize);
        str::from_utf8_unchecked(bytes)
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn take_state(engine_ptr: i32, json_ptr: i32, json_len: i32) -> *mut c_char {
    unsafe {
        let engine = &mut *(engine_ptr as *mut EngineState);
        let json_str = materialize_string(json_ptr, json_len);

        match serde_json::from_str(json_str) {
            Ok(client_features) => {
                engine.take_state(client_features);
                CString::new("Updated features successfully")
                    .unwrap()
                    .into_raw()
            }
            Err(e) => {
                let err_msg = format!("Failed to parse JSON: {}", e);
                CString::new(err_msg).unwrap().into_raw()
            }
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn check_enabled(
    engine_ptr: i32,
    toggle_name_ptr: i32,
    toggle_name_len: i32,
    context_ptr: i32,
    context_len: i32,
) -> *const c_char {
    unsafe {
        let engine = &mut *(engine_ptr as *mut EngineState);
        let toggle_name = materialize_string(toggle_name_ptr, toggle_name_len);
        let context: Context =
            serde_json::from_str(materialize_string(context_ptr, context_len)).unwrap();

        let enabled = engine.check_enabled(toggle_name, &context, &None);

        let result = if let Some(enabled) = enabled {
            Response {
                error_message: None,
                status_code: ResponseCode::Ok,
                value: Some(enabled),
            }
        } else {
            Response {
                error_message: None,
                status_code: ResponseCode::NotFound,
                value: None,
            }
        };
        let response_message = serde_json::to_string(&result).unwrap();

        CString::new(response_message).unwrap().into_raw()
    }
}
