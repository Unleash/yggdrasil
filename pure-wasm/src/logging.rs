use core::cell::UnsafeCell;

#[repr(transparent)]
pub struct SyncUnsafeCell<T>(pub UnsafeCell<T>);

unsafe impl<T> Sync for SyncUnsafeCell<T> {}

pub static LOG_BUFFER: SyncUnsafeCell<[u8; 4096]> = SyncUnsafeCell(UnsafeCell::new([0; 4096]));

#[macro_export]
macro_rules! wasm_log {
    ($($arg:tt)*) => {{
        use core::fmt::Write;

        let buf = &mut *$crate::logging::LOG_BUFFER.0.get();

        struct BufWriter<'a> {
            buf: &'a mut [u8],
            pos: usize,
        }

        impl<'a> Write for BufWriter<'a> {
            fn write_str(&mut self, s: &str) -> core::fmt::Result {
                let bytes = s.as_bytes();
                let space = self.buf.len().saturating_sub(self.pos + 1); // leave room for our null terminator, it's better to truncate than to overflow
                let count = bytes.len().min(space);
                self.buf[self.pos..self.pos + count].copy_from_slice(&bytes[..count]);
                self.pos += count;
                Ok(())
            }
        }

        impl<'a> BufWriter<'a> {
            fn finalize(self) -> usize {
                if self.pos < self.buf.len() {
                    self.buf[self.pos] = 0; //always null terminate, we're reading CStrings on the other side so this prevents reading past the end of the buffer
                }
                self.pos
            }
        }

        let mut writer = BufWriter { buf, pos: 0 };
        let _ = write!(writer, $($arg)*);
        writer.finalize()
    }};
}

// SAFETY: the caller should only ever read from this buffer
#[unsafe(no_mangle)]
pub extern "C" fn get_log_buffer_ptr() -> *const u8 {
    LOG_BUFFER.0.get().cast::<u8>()
}
