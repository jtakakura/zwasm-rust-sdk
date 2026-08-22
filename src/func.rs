use zwasm_sys as sys;

use crate::{
    error::{non_null, trap_into_result, Error},
    store::Store,
    val::Val,
};

/// A callable function, wrapping `wasm_func_t`.
///
/// Obtained from an [`Instance`](crate::instance::Instance) export, or created from
/// a Rust callback with [`Func::new_host`].
pub struct Func {
    pub(crate) ptr: *mut sys::wasm_func_t,
    // Set when `ptr` is borrowed out of an exports vector, which owns it. Dropping
    // the vector is what releases the function, so the two travel together.
    pub(crate) owner: Option<sys::wasm_extern_vec_t>,
}

impl Func {
    /// Creates a host function the guest can call.
    ///
    /// # Safety
    ///
    /// `functype` must point to a live `wasm_functype_t`. Ownership stays with the
    /// caller, who must release it with `wasm_functype_delete` once this call
    /// returns; the arity is copied here.
    ///
    /// `callback` must accept the argument and result arities that `functype`
    /// declares, and must write every result before returning null. Returning a
    /// non-null trap transfers ownership of that trap to the runtime.
    pub unsafe fn new_host(
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

    /// Calls the function.
    ///
    /// `args` has to match the function's declared parameters in count and type;
    /// a mismatch traps rather than failing at compile time. The result vector is
    /// sized from the function's own result arity.
    ///
    /// A guest trap is returned as [`Error::Trap`] carrying the trap message.
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
