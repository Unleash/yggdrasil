use core::str;
use std::{ffi::c_void, mem, slice};

use serde_json::Error;
use unleash_yggdrasil::{EngineState, UpdateMessage};

use getrandom::register_custom_getrandom;

register_custom_getrandom!(get_random_source);

#[unsafe(no_mangle)]
pub fn get_random() -> i32 {
    rand::random::<i32>()
}

static mut RETURN_LEN: i32 = 0;

//This is expected to be defined by the caller and passed to the WASM layer
unsafe extern "C" {
    fn fill_random(ptr: *mut u8, len: usize) -> i32;
}

fn get_random_source(buf: &mut [u8]) -> Result<(), getrandom::Error> {
    let result = unsafe { fill_random(buf.as_mut_ptr(), buf.len()) };
    if result == 0 {
        Ok(())
    } else {
        //probably the wrong error code here
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

fn export_string(s: String) -> *const u8 {
    let len = s.len() as i32;
    let ptr = alloc(len);

    unsafe {
        core::ptr::copy_nonoverlapping(s.as_ptr(), ptr as *mut u8, len as usize);
        RETURN_LEN = len;
    }

    core::mem::forget(s); // don't drop the string
    ptr
}

#[unsafe(no_mangle)]
pub extern "C" fn get_return_len() -> i32 {
    unsafe { RETURN_LEN }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn dealloc(ptr: &mut u8, len: i32) {
    let _ = unsafe { Vec::from_raw_parts(ptr, 0, len as usize) };
}

#[unsafe(no_mangle)]
pub fn new_engine() -> *mut c_void {
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
pub extern "C" fn take_state(engine_ptr: i32, json_ptr: i32, json_len: i32) -> *const u8 {
    unsafe {
        let engine = &mut *(engine_ptr as *mut EngineState);
        let json_str = materialize_string(json_ptr, json_len);

        let client_features: Result<UpdateMessage, Error> = serde_json::from_str(json_str);
        if client_features.is_err() {
            return export_string(format!("{:#?}", client_features.err().unwrap()));
        }
        return 0 as *const u8;
    }
}
