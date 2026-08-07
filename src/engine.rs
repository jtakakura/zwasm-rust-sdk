use zwasm_sys as sys;

use crate::error::{non_null, Error};

pub struct Engine {
    pub(crate) ptr: *mut sys::wasm_engine_t,
}

impl Engine {
    pub fn new() -> Result<Self, Error> {
        let ptr = non_null(unsafe { sys::wasm_engine_new() }, "failed to create engine")?;
        Ok(Engine { ptr })
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self::new().expect("failed to create default Engine")
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        unsafe {
            sys::wasm_engine_delete(self.ptr);
        }
    }
}

unsafe impl Send for Engine {}
unsafe impl Sync for Engine {}
