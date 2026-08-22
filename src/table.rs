use zwasm_sys::{self as sys};

use crate::{
    error::{non_null, Error},
    store::Store,
};

/// A table of references, wrapping `wasm_table_t`.
///
/// Element access (`wasm_table_get` and `wasm_table_set`) is not wrapped, because
/// it works in terms of `wasm_ref_t`, which has no safe representation here yet.
pub struct Table {
    pub(crate) ptr: *mut sys::wasm_table_t,
}

impl Table {
    /// Creates a `funcref` table of `min` slots, growable to `max`.
    ///
    /// `None` for `max` means no maximum. Every slot starts null. The element type
    /// is always `funcref`.
    pub fn new(store: &Store, min: u32, max: Option<u32>) -> Result<Self, Error> {
        let valtype = non_null(
            unsafe { sys::wasm_valtype_new(sys::wasm_valkind_enum_WASM_FUNCREF as u8) },
            "failed to create value type",
        )?;
        let limits = sys::wasm_limits_t {
            min,
            max: max.unwrap_or(sys::wasm_limits_max_default),
        };

        // wasm_tabletype_new takes ownership of valtype, so valtype is only ours to
        // release while this call has not succeeded.
        let tabletype = unsafe { sys::wasm_tabletype_new(valtype, &limits) };
        if tabletype.is_null() {
            unsafe { sys::wasm_valtype_delete(valtype) };
            return Err(Error::Message("failed to create table type".to_string()));
        }

        let ptr = unsafe { sys::wasm_table_new(store.ptr, tabletype, std::ptr::null_mut()) };
        unsafe { sys::wasm_tabletype_delete(tabletype) };

        let ptr = non_null(ptr, "failed to create table")?;
        Ok(Table { ptr })
    }

    /// Grows the table by `delta` null slots.
    ///
    /// Fails when the result would exceed the maximum the table was created with.
    pub fn grow(&mut self, delta: u32) -> Result<(), Error> {
        let result = unsafe { sys::wasm_table_grow(self.ptr, delta, std::ptr::null_mut()) };
        if result {
            Ok(())
        } else {
            Err(Error::Message("failed to grow table".to_string()))
        }
    }

    /// Returns the current number of slots.
    pub fn size(&self) -> u32 {
        unsafe { sys::wasm_table_size(self.ptr) }
    }
}

impl Drop for Table {
    fn drop(&mut self) {
        unsafe {
            sys::wasm_table_delete(self.ptr);
        }
    }
}
