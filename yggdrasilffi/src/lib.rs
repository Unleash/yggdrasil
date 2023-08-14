use std::{
    ffi::{c_char, CStr, CString, NulError},
    str::Utf8Error,
};

use serde::Serialize;
use unleash_types::client_features::ClientFeatures;
use unleash_yggdrasil::{Context, EngineState, VariantDef};

#[derive(Debug)]
enum FFIError {
    Utf8Error,
    NulError,
    InvalidJson,
}

impl From<Utf8Error> for FFIError {
    fn from(_: Utf8Error) -> Self {
        FFIError::Utf8Error
    }
}

impl From<NulError> for FFIError {
    fn from(_: NulError) -> Self {
        FFIError::NulError
    }
}

impl From<serde_json::Error> for FFIError {
    fn from(_: serde_json::Error) -> Self {
        FFIError::InvalidJson
    }
}

#[repr(C)]
pub struct FFIPayload {
    payload_type: *mut c_char,
    value: *mut c_char,
}

#[no_mangle]
pub extern "C" fn engine_new() -> *mut libc::c_void {
    let state = EngineState::default();
    Box::into_raw(Box::new(state)) as *mut libc::c_void
}

#[derive(Serialize)]
enum ResponseCode {
    Error = -2,
    NotFound = -1,
    Disabled = 0,
    Enabled = 1,
}

#[derive(Serialize)]
struct VariantResponse {
    code: i32,
    variant: VariantDef,
}

/// Frees the memory allocated for the engine
/// # Safety
///
/// The caller is responsible for ensuring the pointer to the engine is a valid pointer
/// This function must be called correctly to deallocate the memory allocated for the engine in
/// the engine_new function
#[no_mangle]
pub unsafe extern "C" fn engine_free(ptr: *mut libc::c_void) {
    if ptr.is_null() {
        return;
    }
    unsafe {
        drop(Box::from_raw(ptr as *mut EngineState));
    }
}

/// Takes a JSON string representing a set of toggles and returns a JSON string representing
/// the metrics that should be sent to the server
/// # Safety
///
/// The caller is responsible for ensuring the pointer to the engine is a valid pointer
/// and that the engine is not dropped while the pointer is still in use.
#[no_mangle]
pub unsafe extern "C" fn engine_take_state(
    ptr: *mut libc::c_void,
    json: *const c_char,
) -> *const c_char {
    let state = unsafe {
        assert!(!ptr.is_null());
        &mut *(ptr as *mut EngineState)
    };

    let c_str = unsafe {
        assert!(!json.is_null());
        CStr::from_ptr(json)
    };

    match take_state(state, c_str) {
        Ok(()) => std::ptr::null(),
        Err(err) => {
            println!("Error in engine_take_state: {:?}", err);
            std::ptr::null()
        }
    }
}

fn take_state(state: &mut EngineState, c_str: &CStr) -> Result<(), FFIError> {
    let str_slice: &str = c_str.to_str()?;
    let toggles: ClientFeatures = serde_json::from_str(str_slice)?;
    state.take_state(toggles);
    Ok(())
}

#[no_mangle]
pub unsafe extern "C" fn engine_check_enabled(
    ptr: *mut libc::c_void,
    toggle_name: *const c_char,
    context: *const c_char,
) -> i32 {
    let state = unsafe {
        assert!(!ptr.is_null());
        &mut *(ptr as *mut EngineState)
    };

    let c_toggle_name = unsafe {
        assert!(!toggle_name.is_null());
        CStr::from_ptr(toggle_name)
    };

    let c_context = unsafe {
        assert!(!context.is_null());
        CStr::from_ptr(context)
    };

    let enabled_state = match check_enabled(state, c_toggle_name, c_context) {
        Ok(Some(enabled_state)) => enabled_state,
        Ok(None) => ResponseCode::NotFound,
        Err(_) => ResponseCode::Error,
    };

    enabled_state as i32
}

fn check_enabled(
    state: &EngineState,
    c_toggle_name: &CStr,
    c_context_str: &CStr,
) -> Result<Option<ResponseCode>, FFIError> {
    let toggle_name = c_toggle_name.to_str()?;
    let context_str: &str = c_context_str.to_str()?;
    let context: Context = serde_json::from_str(context_str)?;

    Ok(state.check_enabled(toggle_name, &context).map(|enabled| {
        if enabled {
            ResponseCode::Enabled
        } else {
            ResponseCode::Disabled
        }
    }))
}

#[no_mangle]
pub unsafe extern "C" fn engine_check_variant(
    ptr: *mut libc::c_void,
    toggle_name: *const c_char,
    context: *const c_char,
) -> *mut c_char {
    let state = unsafe {
        assert!(!ptr.is_null());
        &mut *(ptr as *mut EngineState)
    };

    let c_toggle_name = unsafe {
        assert!(!toggle_name.is_null());
        CStr::from_ptr(toggle_name)
    };

    let c_context = unsafe {
        assert!(!context.is_null());
        CStr::from_ptr(context)
    };

    let variant = check_variant(state, c_toggle_name, c_context);
    let variant_response = match variant {
        Ok(Some(variant_def)) => VariantResponse {
            code: variant_def.enabled as i32,
            variant: variant_def,
        },
        Ok(None) => VariantResponse {
            code: ResponseCode::NotFound as i32,
            variant: VariantDef::default(),
        },
        Err(_) => VariantResponse {
            code: ResponseCode::Error as i32,
            variant: VariantDef::default(),
        },
    };
    let json = serde_json::to_string(&variant_response).unwrap();
    CString::new(json).unwrap().into_raw()
}

fn check_variant(
    state: &EngineState,
    c_toggle_name: &CStr,
    c_context_str: &CStr,
) -> Result<Option<VariantDef>, FFIError> {
    let toggle_name = c_toggle_name.to_str()?;
    let context_str: &str = c_context_str.to_str()?;
    let context: Context = serde_json::from_str(context_str)?;

    Ok(state.check_variant(toggle_name, &context))
}

/// Destroys a variant definition, this should only be called on a pointer returned by
/// engine_get_variant_def
///
/// # Safety
/// The caller is responsible for ensuring the pointer to the variant definition is a valid pointer
/// This function must be called on every pointer returned by engine_get_variant_def otherwise this
/// will leak memory
#[no_mangle]
pub unsafe extern "C" fn engine_free_variant_def(ptr: *mut c_char) {
    if ptr.is_null() {
        return;
    }
    drop(CString::from_raw(ptr));
}

#[no_mangle]
pub unsafe extern "C" fn engine_count_toggle(
    pts: *mut libc::c_void,
    toggle_name: *const c_char,
    enabled: bool,
) {
    let state = unsafe {
        assert!(!pts.is_null());
        &mut *(pts as *mut EngineState)
    };

    let c_toggle_name = unsafe {
        assert!(!toggle_name.is_null());
        CStr::from_ptr(toggle_name)
    };

    let toggle_name = c_toggle_name.to_str().unwrap();

    state.count_toggle(&toggle_name, enabled);
}

#[no_mangle]
pub unsafe extern "C" fn engine_count_variant(
    ptr: *mut libc::c_void,
    toggle_name: *const c_char,
    variant_name: *const c_char,
) {
    let state = unsafe {
        assert!(!ptr.is_null());
        &mut *(ptr as *mut EngineState)
    };

    let c_toggle_name = unsafe {
        assert!(!toggle_name.is_null());
        CStr::from_ptr(toggle_name)
    };

    let c_variant_name = unsafe {
        assert!(!variant_name.is_null());
        CStr::from_ptr(variant_name)
    };

    let toggle_name = c_toggle_name.to_str().unwrap();
    let variant_name = c_variant_name.to_str().unwrap();

    state.count_variant(&toggle_name, variant_name);
}

#[no_mangle]
pub unsafe extern "C" fn engine_get_metrics(ptr: *mut libc::c_void) -> *mut c_char {
    let state = unsafe {
        assert!(!ptr.is_null());
        &mut *(ptr as *mut EngineState)
    };

    let metrics = state.get_metrics();
    let json = serde_json::to_string(&metrics).unwrap();
    CString::new(json).unwrap().into_raw()
}
