use thiserror::Error;
use zwasm_sys as sys;

#[derive(Error, Debug)]
#[error("ZwasmError: {0}")]
pub struct ZwasmError(pub String);

pub fn last_error() -> Option<ZwasmError> {
    let err_ptr = unsafe { sys::zwasm_last_error_message() };
    if err_ptr.is_null() {
        None
    } else {
        let c_str = unsafe { std::ffi::CStr::from_ptr(err_ptr) };
        let str_slice = c_str.to_str().unwrap_or("Invalid UTF-8");
        Some(ZwasmError(str_slice.to_string()))
    }
}
