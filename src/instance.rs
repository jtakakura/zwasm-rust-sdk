use zwasm_sys as sys;

use crate::{
    error::{non_null, trap_into_result, Error},
    func::Func,
    module::Module,
    store::Store,
};

/// An instantiated module, wrapping `wasm_instance_t`.
pub struct Instance {
    pub(crate) ptr: *mut sys::wasm_instance_t,
}

impl Instance {
    /// Instantiates `module`, running its start function if it has one.
    ///
    /// `imports` has to line up with the module's import section, in declaration
    /// order. Only function imports are supported; a module importing a memory,
    /// global or table cannot be instantiated through this API yet.
    ///
    /// Imports of `wasi_snapshot_preview1.*` are resolved by the host installed
    /// with [`Store::set_wasi`](crate::store::Store::set_wasi), not through this
    /// argument.
    ///
    /// A trap in the start function is returned as [`Error::Trap`].
    pub fn new(store: &Store, module: &Module, imports: &[&Func]) -> Result<Self, Error> {
        let import_externs: Vec<*mut sys::wasm_extern_t> = imports
            .iter()
            .map(|f| unsafe { sys::wasm_func_as_extern(f.ptr) })
            .collect();
        let import_extern_vec = sys::wasm_extern_vec_t {
            size: import_externs.len(),
            data: import_externs.as_ptr() as *mut _,
        };
        let mut trap: *mut sys::wasm_trap_t = std::ptr::null_mut();
        let ptr =
            unsafe { sys::wasm_instance_new(store.ptr, module.ptr, &import_extern_vec, &mut trap) };

        trap_into_result(trap)?;
        let ptr = non_null(ptr, "failed to create instance")?;

        Ok(Instance { ptr })
    }

    /// Looks a function up by its index among the module's defined functions.
    ///
    /// Prefer [`Instance::get_func_by_name`]. This wraps
    /// `zwasm_instance_get_func`, which indexes the full function index space but
    /// bounds checks against the defined functions alone, so it resolves the wrong
    /// function on a module that imports any. It is only reliable when the module
    /// has no function imports.
    pub fn get_func(&self, index: u32) -> Result<Func, Error> {
        let ptr = non_null(
            unsafe { sys::zwasm_instance_get_func(self.ptr, index) },
            "function not found",
        )?;
        Ok(Func { ptr })
    }

    /// Looks an exported function up by name.
    ///
    /// `module` is needed because `wasm_instance_exports` returns values without
    /// names; the names come from the module's export section, matched by position.
    /// It has to be the module this instance was created from.
    ///
    /// Returns an error when nothing is exported under `name`, or when the export
    /// is not a function.
    pub fn get_func_by_name(&self, module: &Module, name: &str) -> Result<Func, Error> {
        let mut module_exports = sys::wasm_exporttype_vec_t {
            size: 0,
            data: std::ptr::null_mut(),
        };
        unsafe { sys::wasm_module_exports(module.ptr, &mut module_exports) };
        let found_index = (0..module_exports.size).position(|i| {
            let exporttype = unsafe { *module_exports.data.add(i) };
            let name_ptr = unsafe { sys::wasm_exporttype_name(exporttype) };
            let name_bytes = unsafe {
                std::slice::from_raw_parts((*name_ptr).data as *const u8, (*name_ptr).size)
            };
            name_bytes == name.as_bytes()
        });
        unsafe { sys::wasm_exporttype_vec_delete(&mut module_exports) };

        let index =
            found_index.ok_or_else(|| Error::Message(format!("export '{}' not found", name)))?;

        let mut instance_exports = sys::wasm_extern_vec_t {
            size: 0,
            data: std::ptr::null_mut(),
        };
        unsafe { sys::wasm_instance_exports(self.ptr, &mut instance_exports) };
        let ext = unsafe { *instance_exports.data.add(index) };
        let ptr = unsafe { sys::wasm_func_copy(sys::wasm_extern_as_func(ext)) };
        unsafe { sys::wasm_extern_vec_delete(&mut instance_exports) };

        if ptr.is_null() {
            return Err(Error::Message("export is not a function".to_string()));
        }

        Ok(Func { ptr })
    }
}

impl Drop for Instance {
    fn drop(&mut self) {
        unsafe {
            sys::wasm_instance_delete(self.ptr);
        }
    }
}
