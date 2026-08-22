use zwasm_sys as sys;

use crate::{
    engine::Engine,
    error::{non_null, Error},
    wasi::WasiConfig,
};

/// Runtime state for one thread, wrapping `wasm_store_t`.
///
/// A store owns everything instantiated through it. It is deliberately neither
/// `Send` nor `Sync`, because the C side is single threaded per store.
///
/// There is no `Default`, because an [`Engine`] created inside it would be dropped
/// while the store still referred to it.
pub struct Store {
    pub(crate) ptr: *mut sys::wasm_store_t,
}

impl Store {
    /// Creates a store bound to `engine`, which has to outlive it.
    pub fn new(engine: &Engine) -> Result<Self, Error> {
        let ptr = non_null(
            unsafe { sys::wasm_store_new(engine.ptr) },
            "failed to create store",
        )?;
        Ok(Store { ptr })
    }

    /// Installs a WASI host so that imports of `wasi_snapshot_preview1.*` resolve
    /// against it.
    ///
    /// Call this before instantiating. The config is taken by value because the C
    /// side takes ownership of it. Calling twice replaces the previous host and
    /// frees the old config.
    pub fn set_wasi(&mut self, config: WasiConfig) {
        let config = std::mem::ManuallyDrop::new(config);
        unsafe { sys::zwasm_store_set_wasi(self.ptr, config.ptr) };
    }

    /// Removes the WASI host installed by [`Store::set_wasi`] and frees its config.
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
