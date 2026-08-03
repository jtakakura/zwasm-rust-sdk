use zwasm_sys as sys;

use crate::{
    engine::Engine,
    error::{non_null, Error},
};

pub struct Store {
    pub(crate) ptr: *mut sys::wasm_store_t,
}

impl Store {
    pub fn new(engine: &Engine) -> Result<Self, Error> {
        let ptr = non_null(
            unsafe { sys::wasm_store_new(engine.ptr) },
            "failed to create store",
        )?;
        Ok(Store { ptr })
    }
}

impl Drop for Store {
    fn drop(&mut self) {
        unsafe {
            sys::wasm_store_delete(self.ptr);
        }
    }
}
