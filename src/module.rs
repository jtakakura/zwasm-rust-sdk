use crate::config;
use crate::error;
use crate::imports;
use crate::utils;
use crate::wasi;

/// A WebAssembly module instance backed by the zwasm runtime.
///
/// This type provides safe, idiomatic Rust bindings to the zwasm engine, supporting
/// fast JIT execution, full Wasm 3.0 spec coverage, and advanced features like SIMD,
/// threads, GC, and exception handling. Module instances are not thread-safe and must
/// not be shared between threads.
///
/// See the [zwasm project](https://github.com/clojurewasm/zwasm) for details on the underlying engine.
use zwasm_sys as sys;

pub struct Module {
    ptr: *mut sys::zwasm_module_t,
    _not_send_sync: std::marker::PhantomData<std::rc::Rc<()>>,
}

impl Module {
    /* ================================================================
     * Module lifecycle
     * ================================================================ */

    /// Creates a new WebAssembly module instance from raw Wasm bytes using the zwasm runtime.
    ///
    /// This is the most basic way to instantiate a Wasm module. For WASI or custom configuration, see [`Self::new_wasi`] and [`Self::new_configured`].
    pub fn new(wasm_bytes: &[u8]) -> Result<Self, error::ZwasmError> {
        let ptr = unsafe { sys::zwasm_module_new(wasm_bytes.as_ptr(), wasm_bytes.len()) };

        if ptr.is_null() {
            Err(error::last_error()
                .unwrap_or_else(|| error::ZwasmError("Unknown error".to_string())))
        } else {
            Ok(Module {
                ptr,
                _not_send_sync: std::marker::PhantomData,
            })
        }
    }

    /// Creates a new WASI-enabled module instance from raw Wasm bytes using the zwasm runtime.
    ///
    /// WASI syscalls are enabled for this module. For custom WASI configuration, use [`Self::new_wasi_configured`].
    pub fn new_wasi(wasm_bytes: &[u8]) -> Result<Self, error::ZwasmError> {
        let ptr = unsafe { sys::zwasm_module_new_wasi(wasm_bytes.as_ptr(), wasm_bytes.len()) };

        if ptr.is_null() {
            Err(error::last_error()
                .unwrap_or_else(|| error::ZwasmError("Unknown error".to_string())))
        } else {
            Ok(Module {
                ptr,
                _not_send_sync: std::marker::PhantomData,
            })
        }
    }

    /// Creates a module with an explicit runtime configuration (memory, fuel, limits, etc).
    ///
    /// Allows fine-grained control over the execution environment. See [`Config`] for options.
    pub fn new_configured(
        wasm_bytes: &[u8],
        config: &config::Config,
    ) -> Result<Self, error::ZwasmError> {
        let ptr = unsafe {
            sys::zwasm_module_new_configured(wasm_bytes.as_ptr(), wasm_bytes.len(), config.ptr)
        };

        if ptr.is_null() {
            Err(error::last_error()
                .unwrap_or_else(|| error::ZwasmError("Unknown error".to_string())))
        } else {
            Ok(Module {
                ptr,
                _not_send_sync: std::marker::PhantomData,
            })
        }
    }

    /// Creates a WASI-enabled module with an explicit WASI configuration.
    ///
    /// Use this to provide argv, env, preopens, and other WASI settings. See [`WasiConfig`].
    pub fn new_wasi_configured(
        wasm_bytes: &[u8],
        wasi_config: &wasi::WasiConfig,
    ) -> Result<Self, error::ZwasmError> {
        let ptr = unsafe {
            sys::zwasm_module_new_wasi_configured(
                wasm_bytes.as_ptr(),
                wasm_bytes.len(),
                wasi_config.ptr,
            )
        };

        if ptr.is_null() {
            Err(error::last_error()
                .unwrap_or_else(|| error::ZwasmError("Unknown error".to_string())))
        } else {
            Ok(Module {
                ptr,
                _not_send_sync: std::marker::PhantomData,
            })
        }
    }

    /// Creates a WASI-enabled module with both WASI and runtime configuration.
    ///
    /// Combines all options from [`Config`] and [`WasiConfig`].
    pub fn new_wasi_configured2(
        wasm_bytes: &[u8],
        wasi_config: &wasi::WasiConfig,
        config: &config::Config,
    ) -> Result<Self, error::ZwasmError> {
        let ptr = unsafe {
            sys::zwasm_module_new_wasi_configured2(
                wasm_bytes.as_ptr(),
                wasm_bytes.len(),
                wasi_config.ptr,
                config.ptr,
            )
        };

        if ptr.is_null() {
            Err(error::last_error()
                .unwrap_or_else(|| error::ZwasmError("Unknown error".to_string())))
        } else {
            Ok(Module {
                ptr,
                _not_send_sync: std::marker::PhantomData,
            })
        }
    }

    /// Creates a module with host functions registered via [`Imports`].
    ///
    /// Use this to expose custom native functions to Wasm code.
    pub fn new_with_imports(
        wasm_bytes: &[u8],
        imports: &imports::Imports,
    ) -> Result<Self, error::ZwasmError> {
        let ptr = unsafe {
            sys::zwasm_module_new_with_imports(wasm_bytes.as_ptr(), wasm_bytes.len(), imports.ptr)
        };

        if ptr.is_null() {
            Err(error::last_error()
                .unwrap_or_else(|| error::ZwasmError("Unknown error".to_string())))
        } else {
            Ok(Module {
                ptr,
                _not_send_sync: std::marker::PhantomData,
            })
        }
    }

    /// Validates raw Wasm bytes for correctness according to the Wasm spec, without instantiating a module.
    ///
    /// Returns `Ok(())` if the bytes are valid Wasm, or an error otherwise.
    pub fn validate(wasm_bytes: &[u8]) -> Result<(), error::ZwasmError> {
        let ok = unsafe { sys::zwasm_module_validate(wasm_bytes.as_ptr(), wasm_bytes.len()) };

        if !ok {
            Err(error::last_error()
                .unwrap_or_else(|| error::ZwasmError("Unknown error".to_string())))
        } else {
            Ok(())
        }
    }

    /* ================================================================
     * Function invocation
     * ================================================================ */

    /// Invokes an exported function by name in the instantiated Wasm module.
    ///
    /// Arguments and return values are always 64-bit (Wasm i32/i64/f32/f64 are bitcast as needed).
    /// This is the main entrypoint for calling Wasm code from Rust.
    ///
    /// # Safety
    /// This method is safe to call from Rust, but it ultimately delegates to the native
    /// runtime. Callers must ensure no concurrent native access is performed on the same
    /// module instance through other aliases (for example via raw pointers in foreign code).
    pub fn invoke(&self, name: &str, args: &[u64]) -> Result<Vec<u64>, error::ZwasmError> {
        let c_name = std::ffi::CString::new(name)
            .map_err(|_| error::ZwasmError("function name contains NUL byte".into()))?;
        let nresults = self.export_result_count_by_name(name)? as usize;
        let mut results = vec![0u64; nresults];

        let ok = unsafe {
            sys::zwasm_module_invoke(
                self.ptr,
                c_name.as_ptr(),
                if args.is_empty() {
                    std::ptr::null_mut()
                } else {
                    args.as_ptr() as *mut u64
                },
                args.len() as u32,
                if results.is_empty() {
                    std::ptr::null_mut()
                } else {
                    results.as_mut_ptr()
                },
                results.len() as u32,
            )
        };

        if !ok {
            Err(error::last_error()
                .unwrap_or_else(|| error::ZwasmError("Unknown error".to_string())))
        } else {
            Ok(results)
        }
    }

    /// Invokes the module start function, if present.
    ///
    /// This is typically used for modules with a `_start` or similar entrypoint.
    ///
    /// # Safety
    /// This method is safe to call from Rust, but it executes in the native runtime.
    /// The same aliasing/concurrency requirements as [`Self::invoke`] apply.
    pub fn invoke_start(&self) -> Result<(), error::ZwasmError> {
        let ok = unsafe { sys::zwasm_module_invoke_start(self.ptr) };

        if !ok {
            Err(error::last_error()
                .unwrap_or_else(|| error::ZwasmError("Unknown error".to_string())))
        } else {
            Ok(())
        }
    }

    /* ================================================================
     * Export introspection
     * ================================================================ */

    /// Returns the number of exported functions in the module.
    pub fn export_count(&self) -> u32 {
        unsafe { sys::zwasm_module_export_count(self.ptr) }
    }

    /// Returns the export name at `index`, if present.
    ///
    /// Returns `None` if the index is out of bounds.
    pub fn export_name(&self, index: u32) -> Option<String> {
        let ptr = unsafe { sys::zwasm_module_export_name(self.ptr, index) };

        if ptr.is_null() {
            None
        } else {
            Some(
                unsafe { std::ffi::CStr::from_ptr(ptr) }
                    .to_string_lossy()
                    .into_owned(),
            )
        }
    }

    /// Returns the number of parameters for the export at `index`.
    ///
    /// Parameters are always 64-bit values.
    pub fn export_param_count(&self, index: u32) -> u32 {
        unsafe { sys::zwasm_module_export_param_count(self.ptr, index) }
    }

    /// Returns the number of results for the export at `index`.
    ///
    /// Results are always 64-bit values.
    pub fn export_result_count(&self, index: u32) -> u32 {
        unsafe { sys::zwasm_module_export_result_count(self.ptr, index) }
    }

    /// Requests cancellation of currently running Wasm execution in this module.
    ///
    /// # Safety
    /// Cancellation behavior is implemented by the native runtime. Callers should only use
    /// this as an asynchronous signal and must not assume immediate termination semantics.
    pub fn cancel(&self) {
        unsafe { sys::zwasm_module_cancel(self.ptr) };
    }

    /* ================================================================
     * Memory access
     * ================================================================ */

    /// Returns a direct view of the module's linear memory (if present).
    ///
    /// This is a zero-copy view into the Wasm memory. Use with care.
    ///
    /// # Safety
    /// The returned slice points to memory owned by the runtime. The caller must ensure
    /// the memory is not mutated or invalidated (for example by invocation or memory growth)
    /// while this slice is alive.
    pub unsafe fn memory_data(&self) -> Option<&[u8]> {
        let ptr = unsafe { sys::zwasm_module_memory_data(self.ptr) };

        if ptr.is_null() {
            None
        } else {
            Some(unsafe { std::slice::from_raw_parts(ptr, self.memory_size()) })
        }
    }

    /// Returns a copy of the current linear memory snapshot.
    ///
    /// This is the safest way to access Wasm memory from Rust.
    ///
    /// # Safety
    /// This method avoids exposing a borrowed native memory view and is therefore preferred
    /// over [`Self::memory_data`] when possible.
    pub fn memory_data_copy(&self) -> Option<Vec<u8>> {
        unsafe { self.memory_data().map(|s| s.to_vec()) }
    }

    /// Returns the current linear memory size in bytes.
    pub fn memory_size(&self) -> usize {
        unsafe { sys::zwasm_module_memory_size(self.ptr) }
    }

    /// Reads bytes from the module's linear memory into `buf`.
    ///
    /// # Safety
    /// This method is safe to call from Rust, but reads from native-managed memory. The
    /// same aliasing/concurrency requirements as [`Self::invoke`] apply.
    pub fn memory_read(&self, offset: u32, buf: &mut [u8]) -> Result<(), error::ZwasmError> {
        let len = utils::to_u32_len(buf.len())?;
        let ok = unsafe { sys::zwasm_module_memory_read(self.ptr, offset, len, buf.as_mut_ptr()) };

        if !ok {
            Err(error::last_error()
                .unwrap_or_else(|| error::ZwasmError("Unknown error".to_string())))
        } else {
            Ok(())
        }
    }

    /// Writes bytes from `data` into the module's linear memory.
    ///
    /// # Safety
    /// This method is safe to call from Rust, but writes to native-managed memory. The
    /// same aliasing/concurrency requirements as [`Self::invoke`] apply.
    pub fn memory_write(&self, offset: u32, data: &[u8]) -> Result<(), error::ZwasmError> {
        let len = utils::to_u32_len(data.len())?;
        let ok = unsafe { sys::zwasm_module_memory_write(self.ptr, offset, data.as_ptr(), len) };

        if !ok {
            Err(error::last_error()
                .unwrap_or_else(|| error::ZwasmError("Unknown error".to_string())))
        } else {
            Ok(())
        }
    }

    fn export_result_count_by_name(&self, name: &str) -> Result<u32, error::ZwasmError> {
        let index = self.export_index_by_name(name)?;
        Ok(self.export_result_count(index))
    }

    fn export_index_by_name(&self, name: &str) -> Result<u32, error::ZwasmError> {
        let count = self.export_count();
        for index in 0..count {
            if let Some(export_name) = self.export_name(index) {
                if export_name == name {
                    return Ok(index);
                }
            }
        }
        Err(error::ZwasmError(format!("export not found: {name}")))
    }
}

impl Drop for Module {
    fn drop(&mut self) {
        unsafe {
            sys::zwasm_module_delete(self.ptr);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_fixtures;

    #[test]
    fn test_module_lifecycle() {
        let module = Module::new(test_fixtures::MINIMAL_WASM);
        assert!(module.is_ok(), "module_new minimal");

        let module = Module::new(&[0x00, 0x00, 0x00, 0x00]);
        assert!(module.is_err(), "module_new invalid returns Err");

        let err = module.err();
        assert!(
            !err.unwrap().0.is_empty(),
            "last_error non-empty after failure"
        );
    }

    #[test]
    fn test_validate() {
        let result = Module::validate(test_fixtures::RETURN42_WASM);
        assert!(result.is_ok(), "validate valid module");

        let result = Module::validate(&[0x00, 0x00, 0x00, 0x00]);
        assert!(result.is_err(), "validate invalid module returns Err");

        let err = result.err();
        assert!(
            !err.unwrap().0.is_empty(),
            "last_error non-empty after failure"
        );
    }

    #[test]
    fn test_invoke_no_args() {
        let module = Module::new(test_fixtures::RETURN42_WASM).expect("Failed to create module");
        let results = module.invoke("f", &[]).expect("Failed to invoke function");
        println!("f() = {}", results[0]);
        assert_eq!(results[0], 42);

        // Invoke again to test reusability
        let results = module
            .invoke("f", &[])
            .expect("Failed to invoke function second time");
        println!("f() again = {}", results[0]);
        assert_eq!(results[0], 42);
    }

    #[test]
    fn test_invoke_with_args() {
        let module = Module::new(test_fixtures::ADD_WASM).expect("Failed to create module");
        let args = [10, 32];
        let results = module
            .invoke("add", &args)
            .expect("Failed to invoke function");
        println!("add(10, 32) = {}", results[0]);
        assert_eq!(results[0], 42);

        // Edge: zero + zero
        let args = [0, 0];
        let results = module
            .invoke("add", &args)
            .expect("Failed to invoke function");
        println!("add(0, 0) = {}", results[0]);
        assert_eq!(results[0], 0);

        // Edge: large i32 values (wrapping)
        let args = [0xFFFFFFFF, 1];
        let results = module
            .invoke("add", &args)
            .expect("Failed to invoke function");
        println!("add(MAX, 1) = {}", results[0]);
        assert_eq!(results[0] & 0xFFFFFFFF, 0);
    }

    #[test]
    fn test_invoke_nonexistent() {
        let module = Module::new(test_fixtures::RETURN42_WASM).expect("Failed to create module");
        let result = module.invoke("nonexistent", &[]);
        assert!(result.is_err(), "invoke nonexistent returns Err");

        let err = result.err();
        assert!(
            !err.unwrap().0.is_empty(),
            "last_error non-empty after failure"
        );
    }

    #[test]
    fn test_export_introspection() {
        let module = Module::new(test_fixtures::RETURN42_WASM).expect("Failed to create module");
        assert_eq!(module.export_count(), 1, "1 export");

        let name = module.export_name(0).expect("export name not null");
        assert_eq!(name, "f", "export name == 'f'");
        assert_eq!(module.export_param_count(0), 0, "0 params");
        assert_eq!(module.export_result_count(0), 1, "1 result");
        assert!(module.export_name(99).is_none(), "export_name(99) == None");
    }

    #[test]
    fn test_memory_access() {
        let module = Module::new(test_fixtures::MEMORY_WASM).expect("Failed to create module");
        let data = unsafe { module.memory_data() }.expect("memory_data not null");
        let size = module.memory_size();
        assert!(size >= 65536, "memory >= 1 page");
        let write_buf = [0xDE, 0xAD, 0xBE, 0xEF];
        module.memory_write(0, &write_buf).expect("memory_write");
        let mut read_buf = [0u8; 4];
        module.memory_read(0, &mut read_buf).expect("memory_read");
        assert_eq!(write_buf, read_buf, "read == write");
        assert!(data[0] == 0xDE && data[1] == 0xAD, "data ptr matches write");
        let oob_write_result = module.memory_write(size as u32, &write_buf);
        assert!(oob_write_result.is_err(), "OOB write returns Err");
        let oob_read_result = module.memory_read(size as u32, &mut read_buf);
        assert!(oob_read_result.is_err(), "OOB read returns Err");
        module.invoke("f", &[]).expect("invoke store fn");
        let data = unsafe { module.memory_data() }.expect("memory_data not null after invoke");
        let val = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        assert_eq!(val, 42, "store fn wrote 42 at offset 0");
    }

    #[test]
    fn test_no_memory_module() {
        let module = Module::new(test_fixtures::RETURN42_WASM).expect("Failed to create module");
        assert!(
            unsafe { module.memory_data() }.is_none(),
            "memory_data == None for no-memory"
        );
        assert_eq!(module.memory_size(), 0, "memory_size == 0 for no-memory");
    }
}
