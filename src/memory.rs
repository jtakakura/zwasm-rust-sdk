use zwasm_sys::{self as sys};

use crate::{
    error::{non_null, Error},
    store::Store,
};

/// A linear memory, wrapping `wasm_memory_t`.
pub struct Memory {
    pub(crate) ptr: *mut sys::wasm_memory_t,
}

impl Memory {
    /// Creates a memory of `min` pages, growable to `max` pages.
    ///
    /// Sizes are in 64 KiB pages. `None` for `max` means no maximum.
    pub fn new(store: &Store, min: u32, max: Option<u32>) -> Result<Self, Error> {
        let limits = sys::wasm_limits_t {
            min,
            max: max.unwrap_or(sys::wasm_limits_max_default),
        };
        let memorytype = non_null(
            unsafe { sys::wasm_memorytype_new(&limits) },
            "failed to create memory type",
        )?;
        let ptr = unsafe { sys::wasm_memory_new(store.ptr, memorytype) };
        unsafe { sys::wasm_memorytype_delete(memorytype) };

        let ptr = non_null(ptr, "failed to create memory")?;
        Ok(Memory { ptr })
    }

    /// Borrows the memory's bytes.
    ///
    /// Growing the memory can move the backing buffer. [`Memory::grow`] takes
    /// `&mut self`, so the borrow checker rejects holding this slice across one.
    /// Guest code that grows the memory is outside that check.
    pub fn data(&self) -> &[u8] {
        let data_ptr = unsafe { sys::wasm_memory_data(self.ptr) };
        let data_size = unsafe { sys::wasm_memory_data_size(self.ptr) };
        unsafe { std::slice::from_raw_parts(data_ptr as *const u8, data_size) }
    }

    /// Borrows the memory's bytes mutably.
    ///
    /// The same invalidation rule as [`Memory::data`] applies.
    pub fn data_mut(&mut self) -> &mut [u8] {
        let data_ptr = unsafe { sys::wasm_memory_data(self.ptr) };
        let data_size = unsafe { sys::wasm_memory_data_size(self.ptr) };
        unsafe { std::slice::from_raw_parts_mut(data_ptr as *mut u8, data_size) }
    }

    /// Grows the memory by `delta` pages.
    ///
    /// Fails when the result would exceed the maximum the memory was created with.
    /// Takes `&mut self` because growing can move the backing buffer, invalidating
    /// any slice from [`Memory::data`].
    pub fn grow(&mut self, delta: u32) -> Result<(), Error> {
        let result = unsafe { sys::wasm_memory_grow(self.ptr, delta) };
        if result {
            Ok(())
        } else {
            Err(Error::Message("failed to grow memory".to_string()))
        }
    }

    /// Returns the current size in 64 KiB pages.
    ///
    /// For a byte count, take the length of [`Memory::data`].
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
