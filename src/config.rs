use crate::error;
use crate::ffi;
use zwasm_sys as sys;

/* ================================================================
 * Configuration (optional)
 * ================================================================ */

/// Runtime configuration for zwasm modules.
///
/// Allows fine-grained control over memory allocation, fuel, timeouts, and resource limits.
/// Used to customize the execution environment for a [`Module`](crate::Module).
pub struct Config {
    pub(crate) ptr: *mut sys::zwasm_config_t,
    allocator: Option<Box<ConfigAllocator>>,
    _not_send_sync: std::marker::PhantomData<std::rc::Rc<()>>,
}

impl Config {
    /// Creates a new runtime configuration for zwasm modules.
    ///
    /// Use this to control memory allocation, fuel, timeouts, and other resource limits.
    pub fn new() -> Result<Self, error::ZwasmError> {
        let ptr = unsafe { sys::zwasm_config_new() };

        if ptr.is_null() {
            Err(error::last_error()
                .unwrap_or_else(|| error::ZwasmError("Unknown error".to_string())))
        } else {
            Ok(Config {
                ptr,
                allocator: None,
                _not_send_sync: std::marker::PhantomData,
            })
        }
    }

    /// Sets custom memory allocation hooks for the zwasm runtime.
    ///
    /// Allows you to override all memory allocation for Wasm execution. Useful for sandboxing or tracking allocations.
    ///
    /// # Safety
    /// The provided callbacks must obey allocator semantics for all `(size, align)` pairs
    /// that the runtime may request. Returning invalid pointers or violating alignment/
    /// deallocation contracts can cause undefined behavior in native code.
    pub fn set_allocator<F, G>(&mut self, alloc_fn: F, free_fn: G)
    where
        F: Fn(usize, usize) -> *mut std::ffi::c_void + Send + Sync + 'static,
        G: Fn(*mut std::ffi::c_void, usize, usize) + Send + Sync + 'static,
    {
        let mut allocator = Box::new(ConfigAllocator {
            alloc: Box::new(alloc_fn),
            free: Box::new(free_fn),
        });

        let env = &mut *allocator as *mut ConfigAllocator as *mut std::ffi::c_void;

        unsafe {
            sys::zwasm_config_set_allocator(
                self.ptr,
                Some(ffi::config_allocator_alloc_trampoline),
                Some(ffi::config_allocator_free_trampoline),
                env,
            );
        }

        self.allocator = Some(allocator);
    }

    /// Sets the execution fuel limit for the module.
    ///
    /// Fuel limits restrict the number of instructions a module can execute before trapping.
    pub fn set_fuel(&mut self, fuel: u64) {
        unsafe {
            sys::zwasm_config_set_fuel(self.ptr, fuel);
        }
    }

    /// Sets the execution timeout (in milliseconds) for the module.
    ///
    /// Execution will be interrupted if the timeout is exceeded.
    pub fn set_timeout(&mut self, timeout_ms: u64) {
        unsafe {
            sys::zwasm_config_set_timeout(self.ptr, timeout_ms);
        }
    }

    /// Sets the maximum linear memory (in bytes) for the module.
    ///
    /// Prevents Wasm modules from growing memory beyond this limit.
    pub fn set_max_memory(&mut self, max_memory: u64) {
        unsafe {
            sys::zwasm_config_set_max_memory(self.ptr, max_memory);
        }
    }

    /// Forces interpreter mode (disables JIT) when `true`.
    ///
    /// Useful for debugging or running on unsupported platforms.
    pub fn set_force_interpreter(&mut self, force: bool) {
        unsafe {
            sys::zwasm_config_set_force_interpreter(self.ptr, force);
        }
    }

    /// Enables or disables cancellation support for the module.
    ///
    /// When enabled, execution can be cancelled via [`Module::cancel`](crate::Module::cancel).
    pub fn set_cancelable(&mut self, cancelable: bool) {
        unsafe {
            sys::zwasm_config_set_cancellable(self.ptr, cancelable);
        }
    }
}

impl Drop for Config {
    fn drop(&mut self) {
        unsafe {
            sys::zwasm_config_delete(self.ptr);
        }
    }
}

pub(crate) struct ConfigAllocator {
    pub(crate) alloc: Box<dyn Fn(usize, usize) -> *mut std::ffi::c_void + Send + Sync + 'static>,
    pub(crate) free: Box<dyn Fn(*mut std::ffi::c_void, usize, usize) + Send + Sync + 'static>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::module;

    /* ------------------------------------------------------------------ */
    /* Wasm test modules (hand-coded binary)                              */
    /* ------------------------------------------------------------------ */

    /* (func (export "f") (result i32) (i32.const 42)) */
    const RETURN42_WASM: &[u8] = &[
        0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x05, 0x01, 0x60, 0x00, 0x01, 0x7f,
        0x03, 0x02, 0x01, 0x00, 0x07, 0x05, 0x01, 0x01, 0x66, 0x00, 0x00, 0x0a, 0x06, 0x01, 0x04,
        0x00, 0x41, 0x2a, 0x0b,
    ];

    #[test]
    fn test_config_lifecycle() {
        let config = Config::new().expect("Failed to create config");
        let module = module::Module::new_configured(RETURN42_WASM, &config)
            .expect("Failed to create module with config");
        let results = module.invoke("f", &[]).expect("Failed to invoke function");
        assert_eq!(results[0], 42, "f() == 42 via configured module");
    }
}
