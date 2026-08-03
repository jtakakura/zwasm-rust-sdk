use zwasm_sys as sys;

use crate::{
    error::{non_null, trap_into_result, Error},
    func::Func,
    module::Module,
    store::Store,
};

pub struct Instance {
    pub(crate) ptr: *mut sys::wasm_instance_t,
}

impl Instance {
    pub fn new(store: &Store, module: &Module) -> Result<Self, Error> {
        let imports = sys::wasm_extern_vec_t {
            size: 0,
            data: std::ptr::null_mut(),
        };
        let mut trap: *mut sys::wasm_trap_t = std::ptr::null_mut();
        let ptr = unsafe { sys::wasm_instance_new(store.ptr, module.ptr, &imports, &mut trap) };

        trap_into_result(trap)?;
        let ptr = non_null(ptr, "failed to create instance")?;

        Ok(Instance { ptr })
    }

    pub fn get_func(&self, index: u32) -> Result<Func, Error> {
        let ptr = non_null(
            unsafe { sys::zwasm_instance_get_func(self.ptr, index) },
            "function not found",
        )?;
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
