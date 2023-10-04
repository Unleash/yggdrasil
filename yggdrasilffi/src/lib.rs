use std::{
    ffi::{c_char, CStr, CString},
    str::Utf8Error,
};

use libc::c_void;
use serde::{Deserialize, Serialize};
use unleash_types::{client_features::ClientFeatures, client_metrics::MetricBucket};
use unleash_yggdrasil::{Context, EngineState, VariantDef};

#[derive(Serialize, Deserialize)]
struct Response<T> {
    status_code: ResponseCode,
    value: Option<T>,
    error_message: Option<String>,
}

impl<T> From<Result<Option<T>, FFIError>> for Response<T> {
    fn from(value: Result<Option<T>, FFIError>) -> Self {
        match value {
            Ok(Some(enabled)) => Response {
                status_code: ResponseCode::Ok,
                value: Some(enabled),
                error_message: None,
            },
            Ok(None) => Response {
                status_code: ResponseCode::NotFound,
                value: None,
                error_message: None,
            },
            Err(e) => Response {
                status_code: ResponseCode::Error,
                value: None,
                error_message: Some(format!("{:?}", e)),
            },
        }
    }
}

#[derive(Debug)]
enum FFIError {
    Utf8Error,
    InvalidJson,
    NullError(String),
}

impl From<Utf8Error> for FFIError {
    fn from(_: Utf8Error) -> Self {
        FFIError::Utf8Error
    }
}

impl From<serde_json::Error> for FFIError {
    fn from(_: serde_json::Error) -> Self {
        FFIError::InvalidJson
    }
}

#[no_mangle]
pub extern "C" fn engine_new() -> *mut c_void {
    let state = EngineState::default();
    Box::into_raw(Box::new(state)) as *mut c_void
}

#[derive(Serialize, Deserialize, PartialEq, Eq)]
enum ResponseCode {
    Error = -2,
    NotFound = -1,
    Ok = 1,
}

/// Frees the memory allocated for the engine
/// # Safety
///
/// The caller is responsible for ensuring the pointer to the engine is a valid pointer,
/// a passed null pointer is a no-op but an invalid pointer is undefined behavior.
/// This function must be called correctly to deallocate the memory allocated for the engine in
/// the engine_new function, failure to do so will result in a leak.
#[no_mangle]
pub unsafe extern "C" fn engine_free(ptr: *mut c_void) {
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
pub unsafe extern "C" fn engine_take_state(ptr: *mut c_void, json: *const c_char) -> *const c_char {
    let response: Response<()> = take_state(ptr, json).into();
    let json = serde_json::to_string(&response).unwrap();
    CString::new(json).unwrap().into_raw()
}

unsafe fn take_state(ptr: *mut c_void, json: *const c_char) -> Result<Option<()>, FFIError> {
    let engine = if !ptr.is_null() {
        &mut *(ptr as *mut EngineState)
    } else {
        return Err(FFIError::NullError("Null engine pointer".into()));
    };

    let toggles: ClientFeatures = if !json.is_null() {
        let json = CStr::from_ptr(json).to_str()?;
        serde_json::from_str(json)?
    } else {
        return Err(FFIError::NullError("Null json pointer".into()));
    };

    engine.take_state(toggles);
    Ok(Some(()))
}

/// Checks if a toggle is enabled for a given context, this function will always return a JSON encoded response of type `EnabledResponse`
/// # Safety
/// The first argument to this function must be a valid pointer to an EngineState struct. A null pointer will
/// result in an error message being returned to the caller, but an invalid pointer will result in undefined behavior.
/// This pointer should not be dropped for the lifetime of this function call.
///
/// The toggle_name and context parameters should be valid C Strings and not mutated for the lifetime of the function call,
/// null pointers for these values will result in an error message being returned to the caller.
///
/// The caller is responsible for freeing the memory allocated for the returned string, this can be done by calling
/// engine_free_response_message on the pointer returned by this method. Failure to do so will result in a leak.
#[no_mangle]
pub unsafe extern "C" fn engine_check_enabled(
    ptr: *mut c_void,
    toggle_name: *const c_char,
    context: *const c_char,
) -> *const c_char {
    let enabled_state: Response<bool> = check_enabled(ptr, toggle_name, context).into();
    let json = serde_json::to_string(&enabled_state).unwrap();
    CString::new(json).unwrap().into_raw()
}

unsafe fn check_enabled(
    ptr: *mut c_void,
    toggle_name: *const c_char,
    context: *const c_char,
) -> Result<Option<bool>, FFIError> {
    let engine = if !ptr.is_null() {
        &mut *(ptr as *mut EngineState)
    } else {
        return Err(FFIError::NullError("Null engine pointer".into()));
    };

    let toggle_name = if !toggle_name.is_null() {
        CStr::from_ptr(toggle_name).to_str()?
    } else {
        return Err(FFIError::NullError("Null toggle name pointer".into()));
    };

    let context: Context = if !context.is_null() {
        let context_str = CStr::from_ptr(context).to_str()?;
        serde_json::from_str(context_str)?
    } else {
        return Err(FFIError::NullError("Null context pointer".into()));
    };

    Ok(engine.check_enabled(toggle_name, &context))
}

/// Calculates the variant for a toggle, this function will always return a JSON encoded response of type `VariantResponse`
/// # Safety
/// The first argument to this function must be a valid pointer to an EngineState struct. A null pointer will
/// result in an error message being returned to the caller, but an invalid pointer will result in undefined behavior.
/// This pointer should not be dropped for the lifetime of this function call.
///
/// The toggle_name and context parameters should be valid C Strings and not mutated for the lifetime of the function call,
/// null pointers for these values will result in an error message being returned to the caller.
///
/// The caller is responsible for freeing the memory allocated for the returned string, this can be done by calling
/// engine_free_response_message on the pointer returned by this method. Failure to do so will result in a leak.
#[no_mangle]
pub unsafe extern "C" fn engine_check_variant(
    ptr: *mut c_void,
    toggle_name: *const c_char,
    context: *const c_char,
) -> *const c_char {
    let variant_state: Response<VariantDef> = check_variant(ptr, toggle_name, context).into();
    let json = serde_json::to_string(&variant_state).unwrap();
    CString::new(json).unwrap().into_raw()
}

unsafe fn check_variant(
    ptr: *mut c_void,
    toggle_name: *const c_char,
    context: *const c_char,
) -> Result<Option<VariantDef>, FFIError> {
    let engine = if !ptr.is_null() {
        &mut *(ptr as *mut EngineState)
    } else {
        return Err(FFIError::NullError("Null engine pointer".into()));
    };

    let toggle_name = if !toggle_name.is_null() {
        CStr::from_ptr(toggle_name).to_str()?
    } else {
        return Err(FFIError::NullError("Null toggle name pointer".into()));
    };

    let context: Context = if !context.is_null() {
        let context_str = CStr::from_ptr(context).to_str()?;
        serde_json::from_str(context_str)?
    } else {
        return Err(FFIError::NullError("Null context pointer".into()));
    };

    Ok(engine.check_variant(toggle_name, &context))
}

/// Frees the memory allocated for a response message created by check_enabled and check_variant
///
/// # Safety
/// The passed pointer should be a valid pointer to a string allocated by check_enabled or check_variant,
/// passing a null pointer is a no-op but passing an invalid pointer is undefined behavior.
#[no_mangle]
pub unsafe extern "C" fn engine_free_response_message(ptr: *mut c_char) {
    if ptr.is_null() {
        return;
    }
    drop(CString::from_raw(ptr));
}

/// Marks a toggle as being counted for purposes of metrics. This function needs to be paired with a call
/// to get_metrics at a later point in time to retrieve the metrics.
///
/// # Safety
/// If any of the arguments are a null pointer, this function becomes a no-op.
/// The caller is responsible for ensuring that the pointer to the engine is a valid pointer to an EngineState
/// and that the pointer to the toggle_name is a valid pointer to a C String.
/// Failure to uphold these conditions may result in undefined behavior.
///
/// This function does not allocate any resources which need to be manually freed later.
#[no_mangle]
pub unsafe extern "C" fn engine_count_toggle(
    ptr: *mut c_void,
    toggle_name: *const c_char,
    enabled: bool,
) {
    if ptr.is_null() || toggle_name.is_null() {
        return;
    }
    let state = &mut *(ptr as *mut EngineState);
    let c_toggle_name = CStr::from_ptr(toggle_name);

    if let Ok(toggle_name) = c_toggle_name.to_str() {
        state.count_toggle(toggle_name, enabled);
    }
}

/// Marks a variant as being counted for purposes of metrics. This function needs to be paired with a call
/// to get_metrics at a later point in time to retrieve the metrics.
///
/// # Safety
/// If any of the arguments are a null pointer, this function becomes a no-op.
/// The caller is responsible for ensuring that the pointer to the engine is a valid pointer, and that the
/// pointers to the toggle_name and variant_name are valid pointers to C Strings.
/// Failure to uphold these conditions may result in undefined behavior.
///
/// This function does not allocate any resources which need to be manually freed later.

#[no_mangle]
pub unsafe extern "C" fn engine_count_variant(
    ptr: *mut c_void,
    toggle_name: *const c_char,
    variant_name: *const c_char,
) {
    if ptr.is_null() || variant_name.is_null() || toggle_name.is_null() {
        return;
    }
    let state = &mut *(ptr as *mut EngineState);
    let c_toggle_name = CStr::from_ptr(toggle_name);
    let c_variant_name = CStr::from_ptr(variant_name);

    if let (Ok(toggle_name), Ok(variant_name)) = (c_toggle_name.to_str(), c_variant_name.to_str()) {
        state.count_variant(toggle_name, variant_name);
    }
}

/// Returns a JSON encoded string representing the number of times each toggle and variant has been
/// counted since the last time this function was called.
///
/// # Safety
/// The passed pointer should be a valid pointer to an EngineState struct. This function may choose to
/// return a null pointer if the passed pointer is null, or no metrics have been gathered since the last
/// time this function was called. If a non null response is returned, the caller is responsible for freeing
/// the memory allocated for the returned string by calling engine_free_response_message on the returned pointer.
#[no_mangle]
pub unsafe extern "C" fn engine_get_metrics(ptr: *mut c_void) -> *mut c_char {
    let metrics: Response<MetricBucket> = get_metrics(ptr).into();
    let json = serde_json::to_string(&metrics).unwrap();
    CString::new(json).unwrap().into_raw()
}

unsafe fn get_metrics(ptr: *mut c_void) -> Result<Option<MetricBucket>, FFIError> {
    let engine = if !ptr.is_null() {
        &mut *(ptr as *mut EngineState)
    } else {
        return Err(FFIError::NullError("Null engine pointer".into()));
    };

    Ok(engine.get_metrics())
}

#[cfg(test)]
mod tests {
    use std::ffi::{CStr, CString};

    use unleash_types::client_features::{ClientFeature, ClientFeatures, Strategy};
    use unleash_yggdrasil::EngineState;

    use crate::{engine_check_enabled, engine_new, Response, ResponseCode};

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
            let enabled_response: Response<bool> = serde_json::from_str(response).unwrap();

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
            let enabled_response: Response<bool> = serde_json::from_str(response).unwrap();

            assert!(enabled_response.status_code == ResponseCode::Ok);
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
            let enabled_response: Response<bool> = serde_json::from_str(response).unwrap();

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
            let enabled_response: Response<bool> = serde_json::from_str(response).unwrap();

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
            let enabled_response: Response<bool> = serde_json::from_str(response).unwrap();

            assert!(enabled_response.status_code == ResponseCode::Error);
            assert!(enabled_response.error_message.is_some());
        }
    }
}
