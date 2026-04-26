use thiserror::Error;
use zwasm_sys as sys;

#[derive(Error, Debug)]
#[error("ZwasmError: {0}")]
pub struct ZwasmError(pub String);

impl ZwasmError {
    /// Returns `true` when execution was interrupted by host cancellation or timeout.
    pub fn is_interrupted(&self) -> bool {
        self.is_canceled() || self.is_timeout_exceeded()
    }

    /// Returns `true` when execution was canceled by the host.
    pub fn is_canceled(&self) -> bool {
        contains_case_insensitive(&self.0, "execution canceled")
            || contains_case_insensitive(&self.0, "canceled")
    }

    /// Returns `true` when execution stopped because the configured timeout elapsed.
    pub fn is_timeout_exceeded(&self) -> bool {
        contains_case_insensitive(&self.0, "execution timed out")
            || contains_case_insensitive(&self.0, "timeout exceeded")
            || contains_case_insensitive(&self.0, "timed out")
    }
}

fn contains_case_insensitive(haystack: &str, needle: &str) -> bool {
    haystack
        .to_ascii_lowercase()
        .contains(&needle.to_ascii_lowercase())
}

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
