use std::{ffi::OsString, io};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "node:os")]
extern "C" {
    #[wasm_bindgen(catch, js_name = hostname)]
    fn node_hostname() -> Result<String, JsValue>;
}

pub fn get() -> io::Result<OsString> {
    node_hostname()
        .map(OsString::from)
        .map_err(|err| io::Error::other(format!("Failed to get hostname: {err:?}")))
}
