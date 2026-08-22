use zwasm_sys::{self as sys};

use crate::{
    error::{non_null, Error},
    store::Store,
    val::Val,
};

/// A global variable, wrapping `wasm_global_t`.
pub struct Global {
    pub(crate) ptr: *mut sys::wasm_global_t,
}

impl Global {
    /// Creates a global holding `initial`.
    ///
    /// The value type is taken from `initial`. `mutable` decides whether
    /// [`Global::set`] has any effect.
    pub fn new(store: &Store, initial: Val, mutable: bool) -> Result<Self, Error> {
        let valtype = non_null(
            unsafe { sys::wasm_valtype_new(initial.kind()) },
            "failed to create value type",
        )?;
        let mutability = if mutable {
            sys::wasm_mutability_enum_WASM_VAR
        } else {
            sys::wasm_mutability_enum_WASM_CONST
        };

        // wasm_globaltype_new takes ownership of valtype, so valtype is only ours
        // to release while this call has not succeeded.
        let globaltype = unsafe { sys::wasm_globaltype_new(valtype, mutability as u8) };
        if globaltype.is_null() {
            unsafe { sys::wasm_valtype_delete(valtype) };
            return Err(Error::Message("failed to create global type".to_string()));
        }

        let initial_val: sys::wasm_val_t = initial.into();
        let ptr = unsafe { sys::wasm_global_new(store.ptr, globaltype, &initial_val) };
        unsafe { sys::wasm_globaltype_delete(globaltype) };

        let ptr = non_null(ptr, "failed to create global")?;
        Ok(Global { ptr })
    }

    /// Reads the current value.
    pub fn get(&self) -> Val {
        let mut out: sys::wasm_val_t = unsafe { std::mem::zeroed() };
        unsafe { sys::wasm_global_get(self.ptr, &mut out) };
        Val::from(out)
    }

    /// Writes `value`.
    ///
    /// On an immutable global this is silently ignored, matching the C API, which
    /// rejects the out of band write without reporting it. There is no error to
    /// check, so guard on the mutability you created the global with.
    pub fn set(&mut self, value: Val) {
        let val: sys::wasm_val_t = value.into();
        unsafe { sys::wasm_global_set(self.ptr, &val) }
    }
}

impl Drop for Global {
    fn drop(&mut self) {
        unsafe {
            sys::wasm_global_delete(self.ptr);
        }
    }
}
