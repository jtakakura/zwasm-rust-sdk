use zwasm_sys as sys;

use crate::{
    engine::Engine,
    error::{non_null, Error},
    wasi::WasiConfig,
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

    pub fn set_wasi(&mut self, config: WasiConfig) {
        let config = std::mem::ManuallyDrop::new(config);
        unsafe { sys::zwasm_store_set_wasi(self.ptr, config.ptr) };
    }

    pub fn unset_wasi(&mut self) {
        unsafe { sys::zwasm_store_set_wasi(self.ptr, std::ptr::null_mut()) };
    }
}

impl Drop for Store {
    fn drop(&mut self) {
        unsafe {
            sys::wasm_store_delete(self.ptr);
        }
    }
}
