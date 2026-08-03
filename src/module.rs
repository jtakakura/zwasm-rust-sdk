use zwasm_sys as sys;

use crate::{
    error::{non_null, Error},
    store::Store,
};

pub struct Module {
    pub(crate) ptr: *mut sys::wasm_module_t,
}

impl Module {
    pub fn new(store: &Store, wasm_bytes: &[u8]) -> Result<Self, Error> {
        let binary = sys::wasm_byte_vec_t {
            size: wasm_bytes.len(),
            data: wasm_bytes.as_ptr() as *mut _,
        };
        let ptr = non_null(
            unsafe { sys::wasm_module_new(store.ptr, &binary) },
            "failed to create module",
        )?;
        Ok(Module { ptr })
    }
}

impl Drop for Module {
    fn drop(&mut self) {
        unsafe {
            sys::wasm_module_delete(self.ptr);
        }
    }
}
