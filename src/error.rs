use thiserror::Error;
use zwasm_sys as sys;

#[derive(Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Message(String),

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
    let len = if message.size > 0 {
        message.size - 1
    } else {
        0
    };
    let msg = String::from_utf8_lossy(std::slice::from_raw_parts(message.data as *const u8, len))
        .to_string();
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
