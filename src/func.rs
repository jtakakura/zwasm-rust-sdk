use zwasm_sys as sys;

use crate::{
    error::{trap_into_result, Error},
    val::Val,
};

pub struct Func {
    pub(crate) ptr: *mut sys::wasm_func_t,
}

impl Func {
    pub fn call(&self, args: &[Val]) -> Result<Vec<Val>, Error> {
        let args_sys: Vec<sys::wasm_val_t> = args.iter().map(|a| a.clone().into()).collect();
        let args_vec = sys::wasm_val_vec_t {
            size: args_sys.len(),
            data: args_sys.as_ptr() as *mut _,
        };
        let nresults = unsafe { sys::wasm_func_result_arity(self.ptr) };
        let mut results = vec![unsafe { std::mem::zeroed::<sys::wasm_val_t>() }; nresults];
        let mut results_vec = sys::wasm_val_vec_t {
            size: nresults,
            data: results.as_mut_ptr(),
        };
        let trap = unsafe { sys::wasm_func_call(self.ptr, &args_vec, &mut results_vec) };
        trap_into_result(trap)?;
        Ok(results.into_iter().map(|r| r.into()).collect())
    }
}

impl Drop for Func {
    fn drop(&mut self) {
        unsafe {
            sys::wasm_func_delete(self.ptr);
        }
    }
}
