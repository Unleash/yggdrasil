// use std::ffi::c_void;
use unleash_yggdrasil::EngineState;

// #[unsafe(no_mangle)]
// pub fn add(a: i32, b: i32) -> i32 {
//     return a + b + 1;
// }

#[unsafe(no_mangle)]
pub fn take_state() -> i32 {
    2
}

#[unsafe(no_mangle)]
pub fn add(a: i32, b: i32) -> i32 {
    let engine = EngineState::default();
    let context = unleash_yggdrasil::Context::default();
    engine.is_enabled("test", &context, &None);
    11
}

// #[unsafe(no_mangle)]
// pub fn new_engine() -> *mut c_void {
//     let engine = EngineState::default();
//     Box::into_raw(Box::new(engine)) as *mut c_void
// }
