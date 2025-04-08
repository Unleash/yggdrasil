use unleash_yggdrasil::EngineState;

use getrandom::register_custom_getrandom;

register_custom_getrandom!(my_random_fn);

#[unsafe(no_mangle)]
pub fn get_random() -> i32 {
    rand::random::<i32>()
}

//This is expected to be defined by the caller and passed to the WASM layer
unsafe extern "C" {
    fn fill_random(ptr: *mut u8, len: usize) -> i32;
}

fn my_random_fn(buf: &mut [u8]) -> Result<(), getrandom::Error> {
    let result = unsafe { fill_random(buf.as_mut_ptr(), buf.len()) };
    if result == 0 {
        Ok(())
    } else {
        //probably the wrong error code here
        Err(getrandom::Error::UNEXPECTED)
    }
}


#[unsafe(no_mangle)]
pub fn add(a: i32, b: i32) -> i32 {
    let engine = EngineState::initial_state("2022-01-25T12:00:00.000Z".parse().unwrap());
    let context = unleash_yggdrasil::Context::default();
    engine.is_enabled("test", &context, &None);
    11
}