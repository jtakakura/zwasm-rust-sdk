use zwasm_sys as sys;

use crate::{
    error::{non_null, trap_into_result, Error},
    store::Store,
    val::Val,
};

pub struct Func {
    pub(crate) ptr: *mut sys::wasm_func_t,
    pub(crate) owner: Option<sys::wasm_extern_vec_t>,
}

impl Func {
    pub fn new_host(
        store: &Store,
        functype: *const sys::wasm_functype_t,
        callback: sys::wasm_func_callback_t,
    ) -> Result<Self, Error> {
        let func = unsafe { sys::wasm_func_new(store.ptr, functype, callback) };
        let func = non_null(func, "failed to create host function")?;
        Ok(Func {
            ptr: func,
            owner: None,
        })
    }

    pub fn call(&self, args: &[Val]) -> Result<Vec<Val>, Error> {
        let args_vals: Vec<sys::wasm_val_t> = args.iter().map(|a| a.clone().into()).collect();
        let args_vec = sys::wasm_val_vec_t {
            size: args_vals.len(),
            data: args_vals.as_ptr() as *mut _,
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
        match self.owner.take() {
            Some(mut exports) => unsafe { sys::wasm_extern_vec_delete(&mut exports) },
            None => unsafe {
                sys::wasm_func_delete(self.ptr);
            },
        }
    }
}
