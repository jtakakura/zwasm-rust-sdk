use zwasm_sys as sys;

use crate::error::{trap_into_result, Error};

pub struct Func {
    pub(crate) ptr: *mut sys::wasm_func_t,
}

impl Func {
    pub fn call(&self, args: &[sys::wasm_val_t]) -> Result<Vec<sys::wasm_val_t>, Error> {
        let args_vec = sys::wasm_val_vec_t {
            size: args.len(),
            data: args.as_ptr() as *mut _,
        };
        let nresults = unsafe { sys::wasm_func_result_arity(self.ptr) };
        let mut results = vec![unsafe { std::mem::zeroed::<sys::wasm_val_t>() }; nresults];
        let mut results_vec = sys::wasm_val_vec_t {
            size: nresults,
            data: results.as_mut_ptr(),
        };
        let trap = unsafe { sys::wasm_func_call(self.ptr, &args_vec, &mut results_vec) };
        trap_into_result(trap)?;
        Ok(results)
    }
}

impl Drop for Func {
    fn drop(&mut self) {
        unsafe {
            sys::wasm_func_delete(self.ptr);
        }
    }
}
