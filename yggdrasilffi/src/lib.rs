use std::{
    ffi::{c_char, CStr, CString, NulError},
    str::Utf8Error,
};

use unleash_types::client_features::ClientFeatures;
use unleash_yggdrasil::{Context, EngineState};

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
pub struct FFIVariantDef {
    name: *mut c_char,
    payload: *mut FFIPayload,
    enabled: bool,
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

/// Returns a boolean representing the state of the requested toggle for the given context
/// # Safety
///
/// The caller is responsible for ensuring the pointer to the engine is a valid pointer
/// and that the engine is not dropped while the pointer is still in use.
/// toggle_name and context must be pointers to valid UTF-8 strings.
#[no_mangle]
pub unsafe extern "C" fn engine_is_enabled(
    ptr: *mut libc::c_void,
    toggle_name: *const c_char,
    context: *const c_char,
) -> bool {
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

    match is_enabled(state, c_toggle_name, c_context) {
        Ok(is_enabled) => is_enabled,
        Err(err) => {
            println!("Error in engine_is_enabled: {:?}", err);
            false
        }
    }
}

fn is_enabled(
    state: &mut EngineState,
    c_toggle_name: &CStr,
    c_context: &CStr,
) -> Result<bool, FFIError> {
    let toggle_name: &str = c_toggle_name.to_str()?;
    let context_str: &str = c_context.to_str()?;
    let context: Context = serde_json::from_str(context_str)?;

    Ok(state.is_enabled(toggle_name, &context))
}

/// Resolves a variant from the underlying Yggdrasil engine
/// # Safety
///
/// The caller is responsible for ensuring the pointer to the engine is a valid pointer
/// Note that this function allocates memory on the heap and returns a pointer to it.
/// The caller should ensure that engine_free_variant_def is called once that memory
/// has been read by the caller
#[no_mangle]
pub unsafe extern "C" fn engine_get_variant(
    ptr: *mut libc::c_void,
    toggle_name_pointer: *const c_char,
    context_pointer: *const c_char,
) -> *mut c_char {
    let state = &*(ptr as *mut EngineState);
    let c_toggle_name = CStr::from_ptr(toggle_name_pointer);
    let c_context_str = CStr::from_ptr(context_pointer);

    match get_variant(state, c_toggle_name, c_context_str) {
        Ok(c_string) => c_string.into_raw(),
        Err(err) => {
            println!("Error in engine_get_variant: {:?}", err);
            std::ptr::null_mut()
        }
    }
}

fn get_variant(
    state: &EngineState,
    c_toggle_name: &CStr,
    c_context_str: &CStr,
) -> Result<CString, FFIError> {
    let toggle_name = c_toggle_name.to_str()?;
    let context_str: &str = c_context_str.to_str()?;
    let context: Context = serde_json::from_str(context_str)?;

    let variant = state.get_variant(toggle_name, &context);
    let json = serde_json::to_string(&variant)?;
    Ok(CString::new(json)?)
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
