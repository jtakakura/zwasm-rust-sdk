//! # zwasm-sys
//!
//! Low-level Rust bindings for the zwasm C API.
//!
//! This crate exposes the raw C API of zwasm for use in higher-level Rust libraries and tools.
//! All functions are unsafe and require careful use. See the README for important safety notes.
//!
//! Most users should use a safe wrapper or SDK built on top of this crate.
//!
//! ## Safety
//! All functions are unsafe. You must ensure all pointers, memory, and lifetimes are valid.
//!
//! ## Usage
//! This crate is not intended for direct use in applications. Prefer using a safe wrapper if available.

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[cfg(test)]
mod tests {
    use super::*;

    /* Wasm module: export "f" () -> i32 { return 42 } */
    const WASM: &[u8] = &[
        0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x05, 0x01, 0x60, 0x00, 0x01, 0x7f,
        0x03, 0x02, 0x01, 0x00, 0x07, 0x05, 0x01, 0x01, 0x66, 0x00, 0x00, 0x0a, 0x06, 0x01, 0x04,
        0x00, 0x41, 0x2a, 0x0b,
    ];

    #[test]
    fn test_invoke_wasm() {
        unsafe {
            let module = zwasm_module_new(WASM.as_ptr(), WASM.len());
            assert!(!module.is_null(), "module should be created");
            let name = std::ffi::CString::new("f").unwrap();
            let mut results = [0u64; 1];
            let ok = zwasm_module_invoke(
                module,
                name.as_ptr(),
                std::ptr::null(),
                0,
                results.as_mut_ptr(),
                results.len() as u32,
            );
            assert!(ok, "invoke should succeed");
            assert_eq!(results[0], 42);
            zwasm_module_delete(module);
        }
    }

    #[test]
    fn test_invalid_wasm_module() {
        // Invalid/garbage wasm binary
        let invalid = [0u8, 0, 0, 0];
        unsafe {
            let module = zwasm_module_new(invalid.as_ptr(), invalid.len());
            assert!(module.is_null(), "invalid wasm should return null");
            let err_ptr = zwasm_last_error_message();
            let c_str = std::ffi::CStr::from_ptr(err_ptr);
            let msg = c_str.to_str().unwrap();
            assert!(!msg.is_empty(), "error message should be set");
        }
    }

    #[test]
    fn test_invoke_nonexistent_function() {
        unsafe {
            let module = zwasm_module_new(WASM.as_ptr(), WASM.len());
            assert!(!module.is_null());
            let name = std::ffi::CString::new("no_such_func").unwrap();
            let mut results = [0u64; 1];
            let ok = zwasm_module_invoke(
                module,
                name.as_ptr(),
                std::ptr::null(),
                0,
                results.as_mut_ptr(),
                results.len() as u32,
            );
            assert!(!ok, "invoking nonexistent function should fail");
            let err_ptr = zwasm_last_error_message();
            let c_str = std::ffi::CStr::from_ptr(err_ptr);
            let msg = c_str.to_str().unwrap();
            assert!(!msg.is_empty(), "error message should be set");
            zwasm_module_delete(module);
        }
    }

    #[test]
    fn test_null_ptr_and_zero_len() {
        // Should not segfault, should return null
        unsafe {
            let module = zwasm_module_new(std::ptr::null(), 0);
            assert!(module.is_null(), "null ptr and zero len should return null");
            let err_ptr = zwasm_last_error_message();
            let c_str = std::ffi::CStr::from_ptr(err_ptr);
            let msg = c_str.to_str().unwrap();
            assert!(!msg.is_empty(), "error message should be set");
        }
    }
}
