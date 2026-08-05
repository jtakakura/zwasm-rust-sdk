use zwasm_sys::{self as sys};

use crate::{
    error::{non_null, Error},
    store::Store,
};

pub struct Memory {
    pub(crate) ptr: *mut sys::wasm_memory_t,
}

impl Memory {
    pub fn new(store: &Store, min: u32, max: Option<u32>) -> Result<Self, Error> {
        let limits = sys::wasm_limits_t {
            min,
            max: max.unwrap_or(sys::wasm_limits_max_default),
        };
        let memorytype = non_null(
            unsafe { sys::wasm_memorytype_new(&limits) },
            "failed to create memory type",
        )?;
        let ptr = non_null(
            unsafe { sys::wasm_memory_new(store.ptr, memorytype) },
            "failed to create memory",
        )?;
        unsafe { sys::wasm_memorytype_delete(memorytype) };
        Ok(Memory { ptr })
    }

    pub fn data(&self) -> &[u8] {
        let data_ptr = unsafe { sys::wasm_memory_data(self.ptr) };
        let data_size = unsafe { sys::wasm_memory_data_size(self.ptr) };
        unsafe { std::slice::from_raw_parts(data_ptr as *const u8, data_size) }
    }

    pub fn data_mut(&mut self) -> &mut [u8] {
        let data_ptr = unsafe { sys::wasm_memory_data(self.ptr) };
        let data_size = unsafe { sys::wasm_memory_data_size(self.ptr) };
        unsafe { std::slice::from_raw_parts_mut(data_ptr as *mut u8, data_size) }
    }

    pub fn grow(&self, delta: u32) -> Result<(), Error> {
        let result = unsafe { sys::wasm_memory_grow(self.ptr, delta) };
        if result {
            Ok(())
        } else {
            Err(Error::Message("failed to grow memory".to_string()))
        }
    }

    pub fn size(&self) -> u32 {
        unsafe { sys::wasm_memory_size(self.ptr) }
    }
}

impl Drop for Memory {
    fn drop(&mut self) {
        unsafe {
            sys::wasm_memory_delete(self.ptr);
        }
    }
}
