use thiserror::Error;
use zwasm_sys as sys;

/// Anything that can go wrong in this crate.
#[derive(Error, Debug)]
pub enum Error {
    /// An operation failed without producing a trap.
    ///
    /// Most of the C API reports failure as a null pointer or `false` and carries
    /// no reason, so these messages are written here rather than taken from zwasm.
    #[error("{0}")]
    Message(String),

    /// Guest execution trapped. Carries the message from `wasm_trap_message`.
    #[error("{0}")]
    Trap(String),
}

pub(crate) fn non_null<T>(ptr: *mut T, msg: &str) -> Result<*mut T, Error> {
    if ptr.is_null() {
        Err(Error::Message(msg.to_string()))
    } else {
        Ok(ptr)
    }
}

pub(crate) unsafe fn trap_to_error(trap: *mut sys::wasm_trap_t) -> Error {
    let mut message = sys::wasm_message_t {
        size: 0,
        data: std::ptr::null_mut(),
    };
    sys::wasm_trap_message(trap, &mut message);
    let msg = if message.data.is_null() {
        "trap with no message".to_string()
    } else {
        String::from_utf8_lossy(std::slice::from_raw_parts(
            message.data as *const u8,
            message.size,
        ))
        .to_string()
    };
    sys::wasm_byte_vec_delete(&mut message);
    sys::wasm_trap_delete(trap);
    Error::Trap(msg)
}

pub(crate) fn trap_into_result(trap: *mut sys::wasm_trap_t) -> Result<(), Error> {
    if trap.is_null() {
        Ok(())
    } else {
        Err(unsafe { trap_to_error(trap) })
    }
}
