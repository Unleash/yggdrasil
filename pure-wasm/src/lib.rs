use core::str;
use chrono::{DateTime, TimeZone, Utc};
use std::{
    cell::RefCell,
    collections::HashMap,
    ffi::{CString, c_char, c_void},
    mem, slice,
};

mod messaging {
    #![allow(dead_code)]
    #![allow(non_snake_case)]
    include!("enabled-message_generated.rs");
}

use flatbuffers::FlatBufferBuilder;
use messaging::messaging::ResponseBuilder;
use messaging::messaging::MetricsBucketBuilder;

use unleash_yggdrasil::{Context as YggContext, EngineState};

use getrandom::register_custom_getrandom;

register_custom_getrandom!(get_random_source);

thread_local! {
    static BUILDER: RefCell<FlatBufferBuilder<'static>> =
        RefCell::new(FlatBufferBuilder::with_capacity(128));
}

#[unsafe(no_mangle)]
pub fn get_random() -> i32 {
    rand::random::<i32>()
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
        Err(getrandom::Error::NO_RDRAND)
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
pub extern "C" fn dealloc_response_buffer(ptr: *mut u8, len: usize) {
    if !ptr.is_null() && len > 0 {
        unsafe {
            let _ = Vec::from_raw_parts(ptr, 0, len);
        }
    }
}

// This is only used as a thread local so it's thread safe. If this gets moved out of the
// thread local this will ruin someone's day. So don't do that. Please bro
pub fn build_response(enabled: Option<bool>, error: Option<&str>) -> Vec<u8> {
    BUILDER.with(|cell| {
        let mut builder = cell.borrow_mut();
        builder.reset();

        let error_offset = error.map(|e| builder.create_string(e));

        let response = {
            let mut resp_builder = ResponseBuilder::new(&mut builder);
            if let Some(flag) = enabled {
                resp_builder.add_enabled(flag);
                resp_builder.add_has_enabled(true);
            }
            if let Some(err) = error_offset {
                resp_builder.add_error(err);
            }
            resp_builder.finish()
        };

        builder.finish(response, None);
        builder.finished_data().to_vec()
    })
}

pub fn build_metrics_response(
    metrics: Option<unleash_types::client_metrics::MetricBucket>,
) -> Vec<u8> {
    BUILDER.with(|cell| {
        let mut builder = cell.borrow_mut();
        builder.reset();

        let response = {
            let mut resp_builder = MetricsBucketBuilder::new(&mut builder);
            if let Some(metrics) = metrics {
                resp_builder.add_count(metrics.toggles.iter().count() as i32);
            }
            resp_builder.finish()
        };

        builder.finish(response, None);
        builder.finished_data().to_vec()
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn check_enabled(engine_ptr: i32, message_ptr: i32, message_len: i32) -> u64 {
    unsafe {
        let bytes = std::slice::from_raw_parts(message_ptr as *const u8, message_len as usize);
        let ctx: messaging::messaging::ContextMessage =
            flatbuffers::root::<messaging::messaging::ContextMessage>(bytes).expect("invalid context");

        let toggle_name = ctx.toggle_name().expect("You need to pass a toggle name and you also need to remove this expect before production!");

        let context = YggContext {
            user_id: ctx.user_id().map(|f| f.to_string()),
            session_id: ctx.session_id().map(|f| f.to_string()),
            environment: ctx.environment().map(|f| f.to_string()),
            app_name: ctx.app_name().map(|f| f.to_string()),
            current_time: ctx.current_time().map(|f| f.to_string()),
            remote_address: ctx.remote_address().map(|f| f.to_string()),
            properties: ctx.properties().map(|entries| {
                entries
                    .iter()
                    .filter_map(|entry| Some((entry.key().to_string(), entry.value()?.to_string())))
                    .collect::<HashMap<String, String>>()
            }),
        };

        let engine = &mut *(engine_ptr as *mut EngineState);
        let enabled = engine.check_enabled(toggle_name, &context, &None);
        engine.count_toggle(toggle_name, enabled.unwrap_or(false));

        let response = build_response(enabled, None);

        let ptr: u32 = response.as_ptr() as u32;
        let len: u32 = response.len() as u32;
        let packed: u64 = ((len as u64) << 32) | ptr as u64;
        std::mem::forget(response);

        packed
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn get_metrics(engine_ptr: i32, close_time: i64) -> u64 {
    unsafe {
        let engine = &mut *(engine_ptr as *mut EngineState);
        let metrics = engine.get_metrics(DateTime::from_timestamp_nanos(close_time));
        let response = build_metrics_response(metrics);

        let ptr: u32 = response.as_ptr() as u32;
        let len: u32 = response.len() as u32;
        let packed: u64 = ((len as u64) << 32) | ptr as u64;
        std::mem::forget(response);

        packed
    }
}
