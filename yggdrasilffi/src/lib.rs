use std::{
    collections::HashMap,
    ffi::{c_char, CStr, CString},
    fmt::{self, Display, Formatter},
    mem::forget,
    str::Utf8Error,
    sync::{Arc, Mutex, MutexGuard},
};

use chrono::Utc;
use libc::c_void;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use unleash_types::client_metrics::MetricBucket;
use unleash_yggdrasil::{
    state::EnrichedContext, Context, EngineState, EvalWarning, ExtendedVariantDef,
    ToggleDefinition, UpdateMessage, CORE_VERSION, KNOWN_STRATEGIES,
};

static CORE_VERSION_CSTRING: std::sync::LazyLock<CString> =
    std::sync::LazyLock::new(|| CString::new(CORE_VERSION).expect("CString::new failed"));

#[derive(Serialize, Deserialize)]
struct Response<T> {
    status_code: ResponseCode,
    value: Option<T>,
    error_message: Option<String>,
}

type RawPointerDataType = Mutex<EngineState>;
type ManagedEngine = Arc<RawPointerDataType>;
type CustomStrategyResults = HashMap<String, bool>;

#[derive(Serialize, Deserialize, PartialEq, Eq)]
enum ResponseCode {
    Error = -2,
    NotFound = -1,
    Ok = 1,
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
                error_message: Some(e.to_string()),
            },
        }
    }
}

#[derive(Debug)]
enum FFIError {
    Utf8Error,
    NullError,
    InvalidJson(String),
    PartialUpdate(Vec<EvalWarning>),
}

impl Display for FFIError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            FFIError::Utf8Error => write!(f, "Detected a non UTF-8 string in the input, this is a serious issue and you should report this as a bug."),
            FFIError::NullError => write!(f, "Null error detected, this is a serious issue and you should report this as a bug."),
            FFIError::InvalidJson(message) => write!(f, "Failed to parse JSON: {}", message),
            FFIError::PartialUpdate(messages) => write!(
                f,
                "Engine state was updated but warnings were reported, this may result in some flags evaluating in unexpected ways, please report this: {:?}",
                messages
            ),
        }
    }
}

impl From<Utf8Error> for FFIError {
    fn from(_: Utf8Error) -> Self {
        FFIError::Utf8Error
    }
}

impl From<serde_json::Error> for FFIError {
    fn from(e: serde_json::Error) -> Self {
        FFIError::InvalidJson(e.to_string())
    }
}

unsafe fn get_str<'a>(ptr: *const c_char) -> Result<&'a str, FFIError> {
    if ptr.is_null() {
        Err(FFIError::NullError)
    } else {
        unsafe { CStr::from_ptr(ptr).to_str().map_err(FFIError::from) }
    }
}

unsafe fn get_json<T: DeserializeOwned>(json_ptr: *const c_char) -> Result<T, FFIError> {
    let json_str = get_str(json_ptr)?;
    serde_json::from_str(json_str).map_err(FFIError::from)
}

fn result_to_json_ptr<T: Serialize>(result: Result<Option<T>, FFIError>) -> *mut c_char {
    let response: Response<T> = result.into();
    let json_string = serde_json::to_string(&response).unwrap();
    CString::new(json_string).unwrap().into_raw()
}

unsafe fn get_engine(engine_ptr: *mut c_void) -> Result<ManagedEngine, FFIError> {
    if engine_ptr.is_null() {
        return Err(FFIError::NullError);
    }
    let arc_instance = Arc::from_raw(engine_ptr as *const RawPointerDataType);

    let cloned_arc = arc_instance.clone();
    forget(arc_instance);

    Ok(cloned_arc)
}

fn recover_lock<T>(lock: &Mutex<T>) -> MutexGuard<T> {
    match lock.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

/// Instantiates a new engine. Returns a pointer to the engine.
///
/// # Safety
///
/// The caller is responsible for freeing the allocated memory. This can be done by calling
/// `free_engine` and passing in the pointer returned by this method. Failure to do so will result in a leak.
#[no_mangle]
pub extern "C" fn new_engine() -> *mut c_void {
    let engine = Mutex::new(EngineState::default());
    let arc = Arc::new(engine);
    Arc::into_raw(arc) as *mut c_void
}

/// Frees the memory allocated for the engine.
///
/// # Safety
///
/// The caller is responsible for ensuring the argument is a valid pointer.
/// Null pointers will result in a no-op, but any invalid pointers will result in undefined behavior.
/// These pointers should not be dropped for the lifetime of this function call.
///
/// This function must be called correctly in order to deallocate the memory allocated for the engine in
/// the `new_engine` function. Failure to do so will result in a leak.
#[no_mangle]
pub unsafe extern "C" fn free_engine(engine_ptr: *mut c_void) {
    if engine_ptr.is_null() {
        return;
    }
    drop(Arc::from_raw(engine_ptr as *const RawPointerDataType));
}

/// Takes a JSON string representing a set of toggles. Returns a JSON encoded response object
/// specifying whether the update was successful or not. The caller is responsible
/// for freeing this response object.
///
/// # Safety
///
/// The caller is responsible for ensuring all arguments are valid pointers.
/// Null pointers will result in an error message being returned to the caller,
/// but any invalid pointers will result in undefined behavior.
/// These pointers should not be dropped for the lifetime of this function call.
#[no_mangle]
pub unsafe extern "C" fn take_state(
    engine_ptr: *mut c_void,
    json_ptr: *const c_char,
) -> *const c_char {
    let result: Result<Option<()>, FFIError> = (|| {
        let guard = get_engine(engine_ptr)?;
        let mut engine = recover_lock(&guard);

        let toggles: UpdateMessage = get_json(json_ptr)?;

        if let Some(warnings) = engine.take_state(toggles) {
            Err(FFIError::PartialUpdate(warnings))
        } else {
            Ok(Some(()))
        }
    })();

    result_to_json_ptr(result)
}

/// Checks if a toggle is enabled for a given context. Returns a JSON encoded response of type `EnabledResponse`.
///
/// # Safety
///
/// The caller is responsible for ensuring all arguments are valid pointers.
/// Null pointers will result in an error message being returned to the caller,
/// but any invalid pointers will result in undefined behavior.
/// These pointers should not be dropped for the lifetime of this function call.
///
/// The caller is responsible for freeing the allocated memory. This can be done by calling
/// `free_response` and passing in the pointer returned by this method. Failure to do so will result in a leak.
#[no_mangle]
pub unsafe extern "C" fn check_enabled(
    engine_ptr: *mut c_void,
    toggle_name_ptr: *const c_char,
    context_ptr: *const c_char,
    custom_strategy_results_ptr: *const c_char,
) -> *const c_char {
    let result: Result<Option<bool>, FFIError> = (|| {
        let guard = get_engine(engine_ptr)?;
        let engine = recover_lock(&guard);

        let toggle_name = get_str(toggle_name_ptr)?;
        let context: Context = get_json(context_ptr)?;
        let custom_strategy_results =
            get_json::<CustomStrategyResults>(custom_strategy_results_ptr)?;
        let enriched_context =
            EnrichedContext::from(context, toggle_name.into(), Some(custom_strategy_results));

        Ok(engine.check_enabled(&enriched_context))
    })();

    result_to_json_ptr(result)
}

/// Checks the toggle variant for a given context. Returns a JSON encoded response of type `VariantResponse`.
///
/// # Safety
///
/// The caller is responsible for ensuring all arguments are valid pointers.
/// Null pointers will result in an error message being returned to the caller,
/// but any invalid pointers will result in undefined behavior.
/// These pointers should not be dropped for the lifetime of this function call.
///
/// The caller is responsible for freeing the allocated memory. This can be done by calling
/// `free_response` and passing in the pointer returned by this method. Failure to do so will result in a leak.
#[no_mangle]
pub unsafe extern "C" fn check_variant(
    engine_ptr: *mut c_void,
    toggle_name_ptr: *const c_char,
    context_ptr: *const c_char,
    custom_strategy_results_ptr: *const c_char,
) -> *const c_char {
    let result: Result<Option<ExtendedVariantDef>, FFIError> = (|| {
        let guard = get_engine(engine_ptr)?;
        let engine = recover_lock(&guard);

        let toggle_name = get_str(toggle_name_ptr)?;
        let context: Context = get_json(context_ptr)?;
        let custom_strategy_results =
            get_json::<CustomStrategyResults>(custom_strategy_results_ptr)?;
        let enriched_context =
            EnrichedContext::from(context, toggle_name.into(), Some(custom_strategy_results));

        let base_variant = engine.check_variant(&enriched_context);
        let toggle_enabled = engine.check_enabled(&enriched_context).unwrap_or_default();
        Ok(base_variant.map(|variant| variant.to_enriched_response(toggle_enabled)))
    })();

    result_to_json_ptr(result)
}

/// Returns a JSON encoded response with a list of strings representing the built-in strategies Yggdrasil supports.
///
/// # Safety
///
/// The caller is responsible for freeing the allocated memory. This can be done by calling
/// `free_response` and passing in the pointer returned by this method. Failure to do so will result in a leak.
#[no_mangle]
pub unsafe extern "C" fn built_in_strategies() -> *const c_char {
    let strategies = serde_json::to_string(&KNOWN_STRATEGIES).unwrap();
    CString::new(strategies).unwrap().into_raw()
}

/// Returns the version of the Yggdrasil library, in a semantic version format
///
/// # Safety
/// This returns a constant string, you should not call free on the result of this
#[no_mangle]
pub unsafe extern "C" fn get_core_version() -> *const c_char {
    CORE_VERSION_CSTRING.as_ptr() as *const c_char
}

/// Frees the memory allocated for a response message created by `check_enabled` or `check_variant`.
///
/// # Safety
///
/// The caller is responsible for ensuring all arguments are valid pointers.
/// Null pointers will result in an error message being returned to the caller,
/// but any invalid pointers will result in undefined behavior.
/// These pointers should not be dropped for the lifetime of this function call.
///
/// This function must be called correctly in order to deallocate the memory allocated for the response in
/// the `check_enabled`, `check_variant`, `count_toggle`, `count_variant` and `get_metrics` functions.
/// Failure to do so will result in a leak.
#[no_mangle]
pub unsafe extern "C" fn free_response(response_ptr: *mut c_char) {
    if response_ptr.is_null() {
        return;
    }
    drop(CString::from_raw(response_ptr));
}

/// Marks a toggle as being counted for purposes of metrics. This function needs to be paired with a call
/// to `get_metrics` at a later point in time to retrieve the metrics.
///
/// # Safety
///
/// The caller is responsible for ensuring all arguments (except the last one, `enabled`) are valid pointers.
/// Null pointers will result in an error message being returned to the caller,
/// but any invalid pointers will result in undefined behavior.
/// These pointers should not be dropped for the lifetime of this function call.
///
/// The caller is responsible for freeing the allocated memory. This can be done by calling
/// `free_response` and passing in the pointer returned by this method. Failure to do so will result in a leak.
#[no_mangle]
pub unsafe extern "C" fn count_toggle(
    engine_ptr: *mut c_void,
    toggle_name_ptr: *const c_char,
    enabled: u8,
) -> *const c_char {
    // Java/C# may pass other set bits but Rust expects a boolean to only have a single bit set
    // so we need to check exactly that the last bit it 1 or 0 and use the boolean value accordingly

    let enabled = enabled & 1 == 1;

    let result: Result<Option<()>, FFIError> = (|| {
        let guard = get_engine(engine_ptr)?;
        let engine = recover_lock(&guard);

        let toggle_name = get_str(toggle_name_ptr)?;

        engine.count_toggle(toggle_name, enabled);
        Ok(Some(()))
    })();

    result_to_json_ptr(result)
}

/// Marks a variant as being counted for purposes of metrics. This function needs to be paired with a call
/// to `get_metrics` at a later point in time to retrieve the metrics.
///
/// # Safety
///
/// The caller is responsible for ensuring all arguments are valid pointers.
/// Null pointers will result in an error message being returned to the caller,
/// but any invalid pointers will result in undefined behavior.
/// These pointers should not be dropped for the lifetime of this function call.
///
/// The caller is responsible for freeing the allocated memory. This can be done by calling
/// `free_response` and passing in the pointer returned by this method. Failure to do so will result in a leak.
#[no_mangle]
pub unsafe extern "C" fn count_variant(
    engine_ptr: *mut c_void,
    toggle_name_ptr: *const c_char,
    variant_name_ptr: *const c_char,
) -> *const c_char {
    let result: Result<Option<()>, FFIError> = (|| {
        let guard = get_engine(engine_ptr)?;
        let engine = recover_lock(&guard);

        let toggle_name = get_str(toggle_name_ptr)?;
        let variant_name = get_str(variant_name_ptr)?;

        engine.count_variant(toggle_name, variant_name);
        Ok(Some(()))
    })();

    result_to_json_ptr(result)
}

/// Returns a JSON encoded string representing the number of times each toggle and variant has been
/// counted since the last time this function was called.
///
/// # Safety
///
/// The caller is responsible for ensuring all arguments are valid pointers.
/// Null pointers will result in an error message being returned to the caller,
/// but any invalid pointers will result in undefined behavior.
/// These pointers should not be dropped for the lifetime of this function call.
///
/// The caller is responsible for freeing the allocated memory, in case the response is not null. This can be done by calling
/// `free_response` and passing in the pointer returned by this method. Failure to do so will result in a leak.
#[no_mangle]
pub unsafe extern "C" fn get_metrics(engine_ptr: *mut c_void) -> *mut c_char {
    let result: Result<Option<MetricBucket>, FFIError> = (|| {
        let guard = get_engine(engine_ptr)?;
        let mut engine = recover_lock(&guard);

        Ok(engine.get_metrics(Utc::now()))
    })();

    result_to_json_ptr(result)
}

/// Lets you know whether impression events are enabled for this toggle or not.
/// Returns a JSON encoded response of type `Response`.
///
/// # Safety
///
/// The caller is responsible for ensuring the engine_ptr is a valid pointer to an unleash engine.
/// An invalid pointer to unleash engine will result in undefined behaviour.
#[no_mangle]
pub unsafe extern "C" fn should_emit_impression_event(
    engine_ptr: *mut c_void,
    toggle_name_ptr: *const c_char,
) -> *mut c_char {
    let result: Result<Option<bool>, FFIError> = (|| {
        let guard = get_engine(engine_ptr)?;
        let engine = recover_lock(&guard);

        let toggle_name = get_str(toggle_name_ptr)?;

        Ok(Some(engine.should_emit_impression_event(toggle_name)))
    })();

    result_to_json_ptr(result)
}

/// Lists the features currently known by the engine, as set by take_state
/// This is a reduced definition and only includes metadata for the feature,
/// not the properties required to calculate the enabled state of the feature.
/// Returns a JSON encoded response of type `Response`.
///
/// # Safety
///
/// The caller is responsible for ensuring the engine_ptr is a valid pointer to an unleash engine.
/// An invalid pointer to unleash engine will result in undefined behaviour.
/// The caller is responsible for freeing the allocated memory, in case the response is not null. This can be done by calling
/// `free_response` and passing in the pointer returned by this method. Failure to do so will result in a leak.
#[no_mangle]
pub unsafe extern "C" fn list_known_toggles(engine_ptr: *mut c_void) -> *mut c_char {
    let result: Result<Option<Vec<ToggleDefinition>>, FFIError> = (|| {
        let guard = get_engine(engine_ptr)?;
        let engine = recover_lock(&guard);

        Ok(Some(engine.list_known_toggles()))
    })();

    result_to_json_ptr(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use unleash_types::client_features::{
        ClientFeature, ClientFeatures, Strategy, Variant, WeightType,
    };

    #[test]
    fn when_requesting_a_toggle_that_does_not_exist_then_a_response_with_no_error_and_not_found_is_returned(
    ) {
        let engine_ptr = new_engine();

        let c_toggle_name = CString::new("some-toggle").unwrap();
        let c_context = CString::new("{}").unwrap();
        let c_results = CString::new("{}").unwrap();

        let toggle_name_ptr = c_toggle_name.as_ptr();
        let context_ptr = c_context.as_ptr();
        let results_ptr = c_results.as_ptr();

        unsafe {
            let string_response =
                check_enabled(engine_ptr, toggle_name_ptr, context_ptr, results_ptr);
            let response = CStr::from_ptr(string_response).to_str().unwrap();
            let enabled_response: Response<bool> = serde_json::from_str(response).unwrap();

            assert!(enabled_response.status_code == ResponseCode::NotFound);
            assert!(enabled_response.error_message.is_none());
        }
    }

    #[test]
    fn when_requesting_a_toggle_that_does_exist_and_is_enabled_then_a_response_with_no_error_and_enabled_status_is_returned(
    ) {
        let engine_ptr = new_engine();
        let toggle_under_test = "some-toggle";

        let c_toggle_name = CString::new(toggle_under_test).unwrap();
        let c_context = CString::new("{}").unwrap();
        let c_results = CString::new("{}").unwrap();

        let toggle_name_ptr = c_toggle_name.as_ptr();
        let context_ptr = c_context.as_ptr();
        let results_ptr = c_results.as_ptr();

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
            meta: None,
        };

        unsafe {
            let engine_guard = get_engine(engine_ptr).expect("Expected a valid engine pointer");
            let mut engine = engine_guard.lock().expect("Failed to lock engine mutex");
            let warnings = engine.take_state(UpdateMessage::FullResponse(client_features));
            drop(engine);

            let string_response =
                check_enabled(engine_ptr, toggle_name_ptr, context_ptr, results_ptr);
            let response = CStr::from_ptr(string_response).to_str().unwrap();
            let enabled_response: Response<bool> = serde_json::from_str(response).unwrap();

            assert!(enabled_response.status_code == ResponseCode::Ok);
            assert!(enabled_response.error_message.is_none());
            assert!(warnings.is_none());
        }
    }

    #[test]
    fn when_given_a_null_engine_pointer_then_a_error_is_returned() {
        let engine_ptr = std::ptr::null_mut();

        unsafe {
            let c_toggle_name = CString::new("some-toggle").unwrap();
            let c_context = CString::new("{}").unwrap();
            let c_results = CString::new("{}").unwrap();

            let toggle_name_ptr = c_toggle_name.as_ptr();
            let context_ptr = c_context.as_ptr();
            let results_ptr = c_results.as_ptr();

            let string_response =
                check_enabled(engine_ptr, toggle_name_ptr, context_ptr, results_ptr);
            let response = CStr::from_ptr(string_response).to_str().unwrap();
            let enabled_response: Response<bool> = serde_json::from_str(response).unwrap();

            assert!(enabled_response.status_code == ResponseCode::Error);
            assert!(enabled_response.error_message.is_some());
        }
    }

    #[test]
    fn when_given_a_null_toggle_name_pointer_then_a_error_is_returned() {
        let engine_ptr = new_engine();

        unsafe {
            let c_context = CString::new("{}").unwrap();
            let c_results = CString::new("{}").unwrap();

            let toggle_name_ptr = std::ptr::null();
            let context_ptr = c_context.as_ptr();
            let results_ptr = c_results.as_ptr();

            let string_response =
                check_enabled(engine_ptr, toggle_name_ptr, context_ptr, results_ptr);
            let response = CStr::from_ptr(string_response).to_str().unwrap();
            let enabled_response: Response<bool> = serde_json::from_str(response).unwrap();

            assert!(enabled_response.status_code == ResponseCode::Error);
            assert!(enabled_response.error_message.is_some());
        }
    }

    #[test]
    fn when_given_a_null_context_pointer_then_a_error_is_returned() {
        let engine_ptr = new_engine();

        unsafe {
            let c_toggle_name = CString::new("some-toggle").unwrap();
            let c_results = CString::new("{}").unwrap();

            let toggle_name_ptr = c_toggle_name.as_ptr();
            let context_ptr = std::ptr::null();
            let results_ptr = c_results.as_ptr();

            let string_response =
                check_enabled(engine_ptr, toggle_name_ptr, context_ptr, results_ptr);
            let response = CStr::from_ptr(string_response).to_str().unwrap();
            let enabled_response: Response<bool> = serde_json::from_str(response).unwrap();

            assert!(enabled_response.status_code == ResponseCode::Error);
            assert!(enabled_response.error_message.is_some());
        }
    }

    #[test]
    fn variant_response_is_enriched_with_toggle_enabled_status() {
        let engine_ptr = new_engine();
        let toggle_under_test = "some-toggle";

        let c_toggle_name = CString::new(toggle_under_test).unwrap();
        let c_context = CString::new("{}").unwrap();
        let c_results = CString::new("{}").unwrap();

        let toggle_name_ptr = c_toggle_name.as_ptr();
        let context_ptr = c_context.as_ptr();
        let results_ptr = c_results.as_ptr();

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
                variants: Some(vec![Variant {
                    name: "variant".into(),
                    weight: 100,
                    payload: None,
                    overrides: None,
                    stickiness: Some("default".into()),
                    weight_type: Some(WeightType::Fix),
                }]),
                ..Default::default()
            }],
            query: None,
            segments: None,
            meta: None,
            version: 2,
        };

        unsafe {
            let engine_guard = get_engine(engine_ptr).expect("Expected a valid engine pointer");
            let mut engine = engine_guard.lock().expect("Failed to lock engine mutex");
            let warnings = engine.take_state(UpdateMessage::FullResponse(client_features));
            drop(engine);

            let string_response =
                check_variant(engine_ptr, toggle_name_ptr, context_ptr, results_ptr);
            let response = CStr::from_ptr(string_response).to_str().unwrap();
            let variant_response: Response<ExtendedVariantDef> =
                serde_json::from_str(response).unwrap();

            assert!(variant_response.status_code == ResponseCode::Ok);
            let variant_response = variant_response.value.expect("Expected variant response");

            assert!(variant_response.feature_enabled);
            assert!(warnings.is_none());
        }
    }

    #[test]
    fn listing_known_features_returns_a_list_of_toggle_definitions() {
        let engine_ptr = new_engine();

        let client_features = ClientFeatures {
            features: vec![
                ClientFeature {
                    name: "toggle1".into(),
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
                },
                ClientFeature {
                    name: "toggle2".into(),
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
                },
            ],
            query: None,
            segments: None,
            meta: None,
            version: 2,
        };

        unsafe {
            let engine_guard = get_engine(engine_ptr).expect("Expected a valid engine pointer");
            let mut engine = engine_guard.lock().expect("Failed to lock engine mutex");
            engine.take_state(UpdateMessage::FullResponse(client_features));
            drop(engine);

            let string_response = super::list_known_toggles(engine_ptr);
            let response = CStr::from_ptr(string_response).to_str().unwrap();
            let known_features: Response<Vec<super::ToggleDefinition>> =
                serde_json::from_str(response).unwrap();

            assert!(known_features.status_code == ResponseCode::Ok);
            let known_features = known_features.value.expect("Expected known features");

            assert_eq!(known_features.len(), 2);
            assert!(known_features.iter().any(|t| t.name == "toggle1"));
            assert!(known_features.iter().any(|t| t.name == "toggle2"));
        }
    }
}
