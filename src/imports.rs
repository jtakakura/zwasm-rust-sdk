use crate::error;
use crate::ffi;
use zwasm_sys as sys;

/* ================================================================
 * Host function imports
 * ================================================================ */

type HostFn = dyn Fn(&[u64], &mut [u64]) -> Result<(), String> + Send + Sync + 'static;

/// Host function import registry for zwasm modules.
///
/// Allows registering Rust closures as host functions callable from Wasm code.
/// Used to provide custom native functionality to Wasm modules at instantiation.
pub struct Imports {
    pub(crate) ptr: *mut sys::zwasm_imports_t,
    #[allow(clippy::vec_box)]
    _fns: Vec<Box<ImportFn>>,
    _not_send_sync: std::marker::PhantomData<std::rc::Rc<()>>,
}

impl Imports {
    /// Creates a new host imports registry for zwasm modules.
    ///
    /// Use this to register Rust functions as host imports callable from Wasm.
    pub fn new() -> Result<Self, error::ZwasmError> {
        let ptr = unsafe { sys::zwasm_import_new() };
        if ptr.is_null() {
            Err(error::last_error()
                .unwrap_or_else(|| error::ZwasmError("Unknown error".to_string())))
        } else {
            Ok(Imports {
                ptr,
                _fns: Vec::new(),
                _not_send_sync: std::marker::PhantomData,
            })
        }
    }

    /// Registers a Rust function as a host import callable from Wasm code.
    ///
    /// The function will be exposed under the given module and function name, with the specified parameter and result counts (all 64-bit values).
    ///
    /// # Safety
    /// The registered callback is invoked from native runtime code through an FFI trampoline.
    /// The callback must not unwind across FFI boundaries and should avoid re-entrant access
    /// to the same module/runtime state unless such access is externally synchronized.
    pub fn add_fn<F>(
        &mut self,
        module_name: &str,
        func_name: &str,
        param_count: u32,
        result_count: u32,
        func: F,
    ) -> Result<(), error::ZwasmError>
    where
        F: Fn(&[u64], &mut [u64]) -> Result<(), String> + Send + Sync + 'static,
    {
        let c_module_name = std::ffi::CString::new(module_name)
            .map_err(|_| error::ZwasmError("module name contains NUL byte".into()))?;
        let c_func_name = std::ffi::CString::new(func_name)
            .map_err(|_| error::ZwasmError("function name contains NUL byte".into()))?;

        self._fns.push(Box::new(ImportFn {
            module_name: c_module_name,
            func_name: c_func_name,
            param_count,
            result_count,
            func: Box::new(func),
        }));

        let entry = &*self._fns[self._fns.len() - 1];
        let env = entry as *const ImportFn as *mut std::ffi::c_void;

        unsafe {
            sys::zwasm_import_add_fn(
                self.ptr,
                entry.module_name.as_ptr(),
                entry.func_name.as_ptr(),
                Some(ffi::import_fn_trampoline),
                env,
                param_count,
                result_count,
            );
        }

        Ok(())
    }
}

impl Drop for Imports {
    fn drop(&mut self) {
        unsafe {
            sys::zwasm_import_delete(self.ptr);
        }
    }
}

pub(crate) struct ImportFn {
    module_name: std::ffi::CString,
    func_name: std::ffi::CString,
    pub(crate) param_count: u32,
    pub(crate) result_count: u32,
    pub(crate) func: Box<HostFn>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::module;
    use crate::test_fixtures;

    #[test]
    fn test_host_imports() {
        let mut imports = Imports::new().expect("Failed to create imports");
        imports
            .add_fn("env", "add", 2, 1, Box::new(host_add))
            .expect("Failed to add host function");

        let module = module::Module::new_with_imports(test_fixtures::IMPORT_WASM, &imports)
            .expect("Failed to create module with imports");
        let results = module.invoke("call_add", &[]).expect("invoke call_add");
        assert_eq!(results[0], 7, "call_add == 3+4=7");
    }

    fn host_add(args: &[u64], results: &mut [u64]) -> Result<(), String> {
        if args.len() != 2 || results.len() != 1 {
            return Err("invalid arg/result count".to_string());
        }
        let a = args[0] as i32;
        let b = args[1] as i32;
        results[0] = (a + b) as u64;
        Ok(())
    }
}
