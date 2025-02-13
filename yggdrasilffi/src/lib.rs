use std::{
    ffi::c_void,
    fmt::{self, Display, Formatter},
    str::Utf8Error,
};

use serde::{Deserialize, Serialize};
use unleash_yggdrasil::{EngineState, EvalWarning};

pub mod json_api;
pub mod packed_message_api;

enum FFIError {
    Utf8Error,
    NullError,
    InvalidMessageFormat,
    InvalidJson(String),
    PartialUpdate(Vec<EvalWarning>),
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
            FFIError::InvalidMessageFormat => write!(f, "Invalid message format detected, this is a serious issue and you should report this as a bug."),
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

unsafe fn get_engine<'a>(engine_ptr: *mut c_void) -> Result<&'a mut EngineState, FFIError> {
    if engine_ptr.is_null() {
        Err(FFIError::NullError)
    } else {
        Ok(unsafe { &mut *(engine_ptr as *mut EngineState) })
    }
}

#[derive(Serialize, Deserialize)]
struct Response<T> {
    status_code: ResponseCode,
    value: Option<T>,
    error_message: Option<String>,
}

#[derive(Serialize, Deserialize, PartialEq, Eq)]
enum ResponseCode {
    Error = -2,
    NotFound = -1,
    Ok = 1,
}
