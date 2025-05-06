use chrono::DateTime;
use core::str;
use random::get_random_source;
use serialisation::FlatbufferSerializable;
use std::{
    collections::HashMap,
    ffi::{CString, c_char, c_void},
    mem, slice,
};

use getrandom::register_custom_getrandom;
use messaging::messaging::{ContextMessage, FeatureDefs, MetricsBucket, Response, Variant};
use unleash_yggdrasil::{EngineState, ExtendedVariantDef, state::EnrichedContext};

mod logging;
mod random;
mod serialisation;
#[allow(clippy::all)]
mod messaging {
    #![allow(dead_code)]
    #![allow(non_snake_case)]
    #![allow(warnings)]
    include!("enabled-message_generated.rs");
}

register_custom_getrandom!(get_random_source);

#[derive(Debug)]
pub enum WasmError {
    InvalidContext,
}

impl TryFrom<ContextMessage<'_>> for EnrichedContext {
    type Error = WasmError;

    fn try_from(value: ContextMessage) -> Result<Self, Self::Error> {
        let toggle_name = value.toggle_name().ok_or(WasmError::InvalidContext)?;

        let context = EnrichedContext {
            external_results: None,
            toggle_name: toggle_name.to_string(),
            runtime_hostname: value.runtime_hostname().map(|f| f.to_string()),
            user_id: value.user_id().map(|f| f.to_string()),
            session_id: value.session_id().map(|f| f.to_string()),
            environment: value.environment().map(|f| f.to_string()),
            app_name: value.app_name().map(|f| f.to_string()),
            current_time: value.current_time().map(|f| f.to_string()),
            remote_address: value.remote_address().map(|f| f.to_string()),
            properties: value.properties().map(|entries| {
                entries
                    .iter()
                    .filter_map(|entry| Some((entry.key().to_string(), entry.value()?.to_string())))
                    .collect::<HashMap<String, String>>()
            }),
        };

        Ok(context)
    }
}

#[unsafe(no_mangle)]
pub fn new_engine() -> *mut c_void {
    // need to hydrate this from the caller otherwise the metrics will be off
    // doesn't matter for a spike though
    let engine = EngineState::initial_state("2022-01-25T12:00:00.000Z".parse().unwrap());
    Box::into_raw(Box::new(engine)) as *mut c_void
}

#[unsafe(no_mangle)]
pub extern "C" fn alloc(len: i32) -> *const u8 {
    let mut buf = Vec::with_capacity(len as usize);
    let ptr = buf.as_mut_ptr();
    mem::forget(buf);
    ptr
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn dealloc(ptr: &mut u8, len: i32) {
    unsafe { Vec::from_raw_parts(ptr, 0, len as usize) };
}

#[unsafe(no_mangle)]
pub extern "C" fn dealloc_response_buffer(ptr: *mut u8, len: usize) {
    if !ptr.is_null() && len > 0 {
        unsafe {
            let _ = Vec::from_raw_parts(ptr, 0, len);
        }
    }
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
pub extern "C" fn check_enabled(engine_ptr: i32, message_ptr: i32, message_len: i32) -> u64 {
    unsafe {
        let bytes = std::slice::from_raw_parts(message_ptr as *const u8, message_len as usize);
        let ctx: messaging::messaging::ContextMessage =
            flatbuffers::root::<messaging::messaging::ContextMessage>(bytes)
                .expect("invalid context");

        let context: EnrichedContext = ctx.try_into().expect("Failed to convert context");

        let engine = &mut *(engine_ptr as *mut EngineState);
        let enabled = engine.check_enabled(&context);
        engine.count_toggle(&context.toggle_name, enabled.unwrap_or(false));

        let response = Response::build_response(Ok(enabled));

        let ptr: u32 = response.as_ptr() as u32;
        let len: u32 = response.len() as u32;
        let packed: u64 = ((len as u64) << 32) | ptr as u64;
        std::mem::forget(response);

        packed
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn check_variant(engine_ptr: i32, message_ptr: i32, message_len: i32) -> u64 {
    unsafe {
        let bytes = std::slice::from_raw_parts(message_ptr as *const u8, message_len as usize);
        let ctx: messaging::messaging::ContextMessage =
            flatbuffers::root::<messaging::messaging::ContextMessage>(bytes)
                .expect("invalid context");

        let context: EnrichedContext = ctx.try_into().expect("Failed to convert context");

        let engine = &mut *(engine_ptr as *mut EngineState);
        let variant = engine.check_variant(&context);
        let enabled = engine.check_enabled(&context).unwrap_or_default();

        engine.count_toggle(&context.toggle_name, variant.is_some());

        if let Some(variant) = &variant {
            engine.count_variant(&context.toggle_name, &variant.name);
        }

        let extended_variant = variant.map(|variant| ExtendedVariantDef {
            enabled: variant.enabled,
            feature_enabled: enabled,
            name: variant.name,
            payload: variant.payload.clone(),
        });

        let response = Variant::build_response(Ok(extended_variant));

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
        let metrics = engine.get_metrics(DateTime::from_timestamp_millis(close_time).unwrap());
        let response = MetricsBucket::build_response(metrics);

        let ptr: u32 = response.as_ptr() as u32;
        let len: u32 = response.len() as u32;
        let packed: u64 = ((len as u64) << 32) | ptr as u64;
        std::mem::forget(response);

        packed
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn list_known_toggles(engine_ptr: i32) -> u64 {
    unsafe {
        let engine = &mut *(engine_ptr as *mut EngineState);
        let known_toggles = engine.list_known_toggles();
        let response = FeatureDefs::build_response(known_toggles);

        let ptr: u32 = response.as_ptr() as u32;
        let len: u32 = response.len() as u32;
        let packed: u64 = ((len as u64) << 32) | ptr as u64;
        std::mem::forget(response);

        packed
    }
}
