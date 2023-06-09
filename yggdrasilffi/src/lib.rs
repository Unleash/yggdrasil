use std::{
    collections::HashMap,
    ffi::{c_char, CStr, CString},
};

use unleash_types::{client_features::ClientFeatures, client_metrics::MetricBucket};
use unleash_yggdrasil::{Context, EngineState};

#[repr(C)]
pub struct FFIContext {
    pub user_id: *const c_char,
    pub session_id: *const c_char,
    pub environment: *const c_char,
    pub app_name: *const c_char,
    pub current_time: *const c_char,
    pub remote_address: *const c_char,
    pub properties_keys: *const *const c_char,
    pub properties_values: *const *const c_char,
    pub properties_len: usize,
}

#[repr(C)]
pub struct FFIPayload {
    pub payload_type: *const c_char,
    pub value: *const c_char,
}

#[repr(C)]
pub struct FFIVariantDef {
    pub name: *const c_char,
    pub payload: Option<*mut FFIPayload>,
    pub enabled: bool,
}

#[no_mangle]
pub extern "C" fn engine_new() -> *mut libc::c_void {
    let state = EngineState::default();
    Box::into_raw(Box::new(state)) as *mut libc::c_void
}

#[no_mangle]
pub extern "C" fn engine_free(ptr: *mut libc::c_void) {
    if ptr.is_null() {
        return;
    }
    unsafe {
        drop(Box::from_raw(ptr as *mut EngineState));
    }
}

#[no_mangle]
pub extern "C" fn engine_take_state(ptr: *mut libc::c_void, json: *const c_char) -> *const c_char {
    let state = unsafe {
        assert!(!ptr.is_null());
        &mut *(ptr as *mut EngineState)
    };

    let c_str = unsafe {
        assert!(!json.is_null());
        CStr::from_ptr(json)
    };

    let str_slice: &str = c_str.to_str().unwrap();
    let toggles: ClientFeatures = serde_json::from_str(str_slice).unwrap();

    let metric_bucket: Option<MetricBucket> = state.take_state(toggles);
    let json_str: String = serde_json::to_string(&metric_bucket).unwrap();
    let c_string = CString::new(json_str).unwrap();
    c_string.into_raw()
}

#[no_mangle]
pub extern "C" fn engine_is_enabled(
    ptr: *mut libc::c_void,
    toggle_name: *const c_char,
    context: *const FFIContext,
) -> bool {
    let state = unsafe {
        assert!(!ptr.is_null());
        &mut *(ptr as *mut EngineState)
    };

    let c_str = unsafe {
        assert!(!toggle_name.is_null());
        CStr::from_ptr(toggle_name)
    };

    let toggle_name: &str = c_str.to_str().unwrap();

    let context = unsafe { &*context };
    let context = context.into();

    state.is_enabled(&toggle_name, &context)
}

#[no_mangle]
pub unsafe extern "C" fn engine_get_variant(
    ptr: *mut libc::c_void,
    toggle_name: *const c_char,
    context: *mut FFIContext,
) -> *mut FFIVariantDef {
    let state = unsafe {
        assert!(!ptr.is_null());
        &mut *(ptr as *mut EngineState)
    };

    let toggle_name = CStr::from_ptr(toggle_name).to_str().unwrap();
    let context = unsafe { &*context };
    let context = context.into();

    let variant_def = state.get_variant(toggle_name, &context);
    let ffi_variant_def = FFIVariantDef {
        name: CString::new(variant_def.name).unwrap().into_raw(),
        payload: match variant_def.payload {
            Some(payload) => Some(Box::into_raw(Box::new(FFIPayload {
                payload_type: CString::new(payload.payload_type).unwrap().into_raw(),
                value: CString::new(payload.value).unwrap().into_raw(),
            }))),
            None => None,
        },
        enabled: variant_def.enabled,
    };
    Box::into_raw(Box::new(ffi_variant_def))
}

#[no_mangle]
pub extern "C" fn engine_free_variant_def(ptr: *mut FFIVariantDef) {
    if !ptr.is_null() {
        unsafe {
            let variant_def = Box::from_raw(ptr);
            if !variant_def.name.is_null() {
                let name_ptr = variant_def.name as *mut c_char;
                drop(CString::from_raw(name_ptr));
            }

            if let Some(payload_ptr) = variant_def.payload {
                let payload = Box::from_raw(payload_ptr);
                if !payload.payload_type.is_null() {
                    let payload_type_ptr = payload.payload_type as *mut c_char;
                    drop(CString::from_raw(payload_type_ptr));
                }
                if !payload.value.is_null() {
                    let value_ptr = payload.value as *mut c_char;
                    drop(CString::from_raw(value_ptr));
                }
            }
        }
    }
}

// This is about as safe as running into a TNT factory while you're on fire
// but without tanking performance this is the most effective way to do this.
// We're trusting the caller to behave in a sensible way here. So long as this is
// wrapped by something we trust it should be okay.
//
// This conversation impl makes a few assumptions:
// 1) The caller sends us a valid pointer for each context value
// 2) The caller restructures the properties hashmap into two separate arrays and a length
// 2.1) Those arrays are equal in length
// 2.2) Those arrays contain only valid UTF-8 strings, this means valid keys with null properties are not allowed
// 2.3) The len property is correct i.e. matches the length of the original hashmap
// 3) The caller will not free any of this memory while we are still using it
// 4) The caller is responsible for controlling concurrency primitives; this cannot be made thread safe here
// Violating any of these assumptions will result in a segfault, or, in the case of 4, unpredictable and bad things
//
// This also makes the assumption that the caller will free the allocated memory of the
// underlying strings after returning. If the caller does not do that, this will result in a memory leak
//
// The caller in this context is the host language FFI wrapper, not the Rust code that consumes this
impl<'a> Into<Context> for &'a FFIContext {
    fn into(self) -> Context {
        let properties = unsafe {
            let keys = std::slice::from_raw_parts(self.properties_keys, self.properties_len);
            let values = std::slice::from_raw_parts(self.properties_values, self.properties_len);

            keys.iter()
                .zip(values.iter())
                .filter_map(|(&k, &v)| {
                    if k.is_null() || v.is_null() {
                        None
                    } else {
                        let key = CStr::from_ptr(k).to_string_lossy().into_owned();
                        let value = CStr::from_ptr(v).to_string_lossy().into_owned();
                        Some((key, value))
                    }
                })
                .collect::<HashMap<_, _>>()
        };

        Context {
            user_id: if self.user_id.is_null() {
                None
            } else {
                Some(unsafe { CStr::from_ptr(self.user_id).to_string_lossy().into_owned() })
            },
            session_id: if self.session_id.is_null() {
                None
            } else {
                Some(unsafe {
                    CStr::from_ptr(self.session_id)
                        .to_string_lossy()
                        .into_owned()
                })
            },
            environment: if self.environment.is_null() {
                None
            } else {
                Some(unsafe {
                    CStr::from_ptr(self.environment)
                        .to_string_lossy()
                        .into_owned()
                })
            },
            app_name: if self.app_name.is_null() {
                None
            } else {
                Some(unsafe { CStr::from_ptr(self.app_name).to_string_lossy().into_owned() })
            },
            current_time: if self.current_time.is_null() {
                None
            } else {
                Some(unsafe {
                    CStr::from_ptr(self.current_time)
                        .to_string_lossy()
                        .into_owned()
                })
            },
            remote_address: if self.remote_address.is_null() {
                None
            } else {
                Some(unsafe {
                    CStr::from_ptr(self.remote_address)
                        .to_string_lossy()
                        .into_owned()
                })
            },
            properties: if self.properties_keys.is_null() || self.properties_values.is_null() {
                None
            } else {
                Some(properties)
            },
        }
    }
}
