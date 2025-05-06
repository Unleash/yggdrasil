#[unsafe(no_mangle)]
pub fn get_random() -> i32 {
    rand::random::<i32>()
}

//This is expected to be defined by the caller and passed to the WASM layer
unsafe extern "C" {
    fn fill_random(ptr: *mut u8, len: usize) -> i32;
}

pub(crate) fn get_random_source(buf: &mut [u8]) -> Result<(), getrandom::Error> {
    let result = unsafe { fill_random(buf.as_mut_ptr(), buf.len()) };
    if result == 0 {
        Ok(())
    } else {
        // probably the wrong error code here, this may need a custom definition
        // good enough for a spike
        Err(getrandom::Error::NO_RDRAND)
    }
}
