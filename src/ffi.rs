use crate::config;
use crate::imports;

pub(crate) unsafe extern "C" fn config_allocator_alloc_trampoline(
    env: *mut std::ffi::c_void,
    size: usize,
    align: usize,
) -> *mut std::ffi::c_void {
    if env.is_null() {
        return std::ptr::null_mut();
    }

    let allocator = unsafe { &*(env as *const config::ConfigAllocator) };

    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        (allocator.alloc)(size, align)
    })) {
        Ok(ptr) => ptr,
        Err(_) => std::ptr::null_mut(),
    }
}

pub(crate) unsafe extern "C" fn config_allocator_free_trampoline(
    env: *mut std::ffi::c_void,
    ptr: *mut std::ffi::c_void,
    size: usize,
    align: usize,
) {
    if env.is_null() {
        return;
    }

    let allocator = unsafe { &*(env as *const config::ConfigAllocator) };
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        (allocator.free)(ptr, size, align)
    }));
}

pub(crate) unsafe extern "C" fn import_fn_trampoline(
    env: *mut std::ffi::c_void,
    args_ptr: *const u64,
    results_ptr: *mut u64,
) -> bool {
    if env.is_null() {
        return false;
    }

    let func = unsafe { &*(env as *const imports::ImportFn) };
    let args = if func.param_count == 0 {
        &[]
    } else {
        if args_ptr.is_null() {
            return false;
        }

        unsafe { std::slice::from_raw_parts(args_ptr, func.param_count as usize) }
    };
    let results = if func.result_count == 0 {
        &mut []
    } else {
        if results_ptr.is_null() {
            return false;
        }

        unsafe { std::slice::from_raw_parts_mut(results_ptr, func.result_count as usize) }
    };

    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| (func.func)(args, results))) {
        Ok(Ok(())) => true,
        Ok(Err(_)) => false,
        Err(_) => false,
    }
}
