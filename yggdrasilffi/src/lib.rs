use std::{
    ffi::{c_char, CStr, CString, NulError},
    str::Utf8Error,
};

use serde::{Deserialize, Serialize};
use unleash_types::client_features::ClientFeatures;
use unleash_yggdrasil::{Context, EngineState, VariantDef};

#[derive(Serialize, Deserialize, PartialEq, Eq)]
struct EnabledResponse {
    status_code: ResponseCode, // 1 enabled, 0 disabled, -1 not found, -2 error
    error_message: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct VariantResponse {
    status_code: ResponseCode, // 1 enabled, 0 disabled, -1 not found, -2 error
    variant: Option<VariantDef>,
    error_message: Option<String>,
}

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

#[derive(Serialize, Deserialize, PartialEq, Eq)]
enum ResponseCode {
    Error = -2,
    NotFound = -1,
    Disabled = 0,
    Enabled = 1,
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

fn error_if_null(ptr: *mut libc::c_void) -> Option<EnabledResponse> {
    if ptr.is_null() {
        Some(EnabledResponse {
            status_code: ResponseCode::Error,
            error_message: Some("Null pointer to unleash engine".into()),
        })
    } else {
        None
    }
}

#[no_mangle]
pub unsafe extern "C" fn engine_check_enabled(
    ptr: *mut libc::c_void,
    toggle_name: *const c_char,
    context: *const c_char,
) -> *const c_char {
    let state = unsafe {
        if let Some(error) = error_if_null(ptr) {
            let json = serde_json::to_string(&error).unwrap();
            return CString::new(json).unwrap().into_raw();
        }
        &mut *(ptr as *mut EngineState)
    };

    let c_toggle_name = unsafe {
        if toggle_name.is_null() {
            let json = serde_json::to_string(&EnabledResponse {
                error_message: Some("Null pointer to toggle name".into()),
                status_code: ResponseCode::Error,
            })
            .unwrap();
            return CString::new(json).unwrap().into_raw();
        }
        CStr::from_ptr(toggle_name)
    };

    let c_context = unsafe {
        if context.is_null() {
            let json = serde_json::to_string(&EnabledResponse {
                error_message: Some("Null pointer to context".into()),
                status_code: ResponseCode::Error,
            })
            .unwrap();
            return CString::new(json).unwrap().into_raw();
        }

        CStr::from_ptr(context)
    };

    let enabled_state = match check_enabled(state, c_toggle_name, c_context) {
        Ok(Some(enabled_state)) => EnabledResponse {
            status_code: enabled_state,
            error_message: None,
        },
        Ok(None) => EnabledResponse {
            status_code: ResponseCode::NotFound,
            error_message: None,
        },
        Err(_) => EnabledResponse {
            status_code: ResponseCode::Error,
            error_message: "Something went wrong in the core FFI layers, this is a fatal issue"
                .to_string()
                .into(),
        },
    };

    let json = serde_json::to_string(&enabled_state).unwrap();
    CString::new(json).unwrap().into_raw()
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
) -> *const c_char {
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

    let variant_state = match check_variant(state, c_toggle_name, c_context) {
        Ok(Some(variant_state)) => VariantResponse {
            status_code: if variant_state.enabled {
                ResponseCode::Enabled
            } else {
                ResponseCode::Disabled
            },
            variant: Some(variant_state),
            error_message: None,
        },
        Ok(None) => VariantResponse {
            status_code: ResponseCode::NotFound,
            variant: None,
            error_message: None,
        },
        Err(_) => VariantResponse {
            status_code: ResponseCode::Error,
            variant: None,
            error_message: "Something went wrong in the core FFI layers, this is a fatal issue"
                .to_string()
                .into(),
        },
    };

    let json = serde_json::to_string(&variant_state).unwrap();
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

#[cfg(test)]
mod tests {
    use std::ffi::{CStr, CString};

    use unleash_types::client_features::{ClientFeature, ClientFeatures, Strategy};
    use unleash_yggdrasil::EngineState;

    use crate::{engine_check_enabled, engine_new, EnabledResponse, ResponseCode};

    #[test]
    fn when_requesting_a_toggle_that_does_not_exist_then_a_response_with_no_error_and_not_found_is_returned(
    ) {
        let engine_ptr = engine_new();

        let c_toggle_name = CString::new("some-toggle").unwrap();
        let c_context = CString::new("{}").unwrap();

        let toggle_name_ptr = c_toggle_name.as_ptr();
        let context_ptr = c_context.as_ptr();

        unsafe {
            let string_response = engine_check_enabled(engine_ptr, toggle_name_ptr, context_ptr);
            let response = CStr::from_ptr(string_response).to_str().unwrap();
            let enabled_response: EnabledResponse = serde_json::from_str(response).unwrap();

            assert!(enabled_response.status_code == ResponseCode::NotFound);
            assert!(enabled_response.error_message.is_none());
        }
    }

    #[test]
    fn when_requesting_a_toggle_that_does_exist_and_is_enabled_then_a_response_with_no_error_and_enabled_status_is_returned(
    ) {
        let engine_ptr = engine_new();
        let toggle_under_test = "some-toggle";

        let c_toggle_name = CString::new(toggle_under_test).unwrap();
        let c_context = CString::new("{}").unwrap();

        let toggle_name_ptr = c_toggle_name.as_ptr();
        let context_ptr = c_context.as_ptr();

        let client_features = ClientFeatures {
            features: vec![ClientFeature {
                name: toggle_under_test.into(),
                enabled: true,
                strategies: Some(vec![Strategy {
                    name: "default".into(),
                    constraints: None,
                    parameters: None,
                    segments: None,
                    sort_order: None,
                    variants: None,
                }]),
                ..Default::default()
            }],
            query: None,
            segments: None,
            version: 2,
        };

        unsafe {
            let engine = &mut *(engine_ptr as *mut EngineState);
            engine.take_state(client_features);

            let string_response = engine_check_enabled(engine_ptr, toggle_name_ptr, context_ptr);
            let response = CStr::from_ptr(string_response).to_str().unwrap();
            let enabled_response: EnabledResponse = serde_json::from_str(response).unwrap();

            assert!(enabled_response.status_code == ResponseCode::Enabled);
            assert!(enabled_response.error_message.is_none());
        }
    }

    #[test]
    fn when_given_a_null_engine_pointer_then_a_error_is_returned() {
        let engine_ptr = std::ptr::null_mut();

        unsafe {
            let c_toggle_name = CString::new("some-toggle").unwrap();
            let c_context = CString::new("{}").unwrap();

            let toggle_name_ptr = c_toggle_name.as_ptr();
            let context_ptr = c_context.as_ptr();

            let string_response = engine_check_enabled(engine_ptr, toggle_name_ptr, context_ptr);
            let response = CStr::from_ptr(string_response).to_str().unwrap();
            let enabled_response: EnabledResponse = serde_json::from_str(response).unwrap();

            assert!(enabled_response.status_code == ResponseCode::Error);
            assert!(enabled_response.error_message.is_some());
        }
    }

    #[test]
    fn when_given_a_null_toggle_name_pointer_then_a_error_is_returned() {
        let engine_ptr = engine_new();

        unsafe {
            let c_context = CString::new("{}").unwrap();

            let toggle_name_ptr = std::ptr::null();
            let context_ptr = c_context.as_ptr();

            let string_response = engine_check_enabled(engine_ptr, toggle_name_ptr, context_ptr);
            let response = CStr::from_ptr(string_response).to_str().unwrap();
            let enabled_response: EnabledResponse = serde_json::from_str(response).unwrap();

            assert!(enabled_response.status_code == ResponseCode::Error);
            assert!(enabled_response.error_message.is_some());
        }
    }

    #[test]
    fn when_given_a_null_context_pointer_then_a_error_is_returned() {
        let engine_ptr = engine_new();

        unsafe {
            let c_toggle_name = CString::new("some-toggle").unwrap();

            let toggle_name_ptr = c_toggle_name.as_ptr();
            let context_ptr = std::ptr::null();

            let string_response = engine_check_enabled(engine_ptr, toggle_name_ptr, context_ptr);
            let response = CStr::from_ptr(string_response).to_str().unwrap();
            let enabled_response: EnabledResponse = serde_json::from_str(response).unwrap();

            assert!(enabled_response.status_code == ResponseCode::Error);
            assert!(enabled_response.error_message.is_some());
        }
    }
}
