use std::{ffi::CString, os::raw::c_char};

use zwasm_sys::{self as sys};

use crate::error::{non_null, Error};

pub struct WasiConfig {
    pub(crate) ptr: *mut sys::zwasm_wasi_config_t,
}

impl WasiConfig {
    pub fn new() -> Result<Self, Error> {
        let ptr = non_null(
            unsafe { sys::zwasm_wasi_config_new() },
            "failed to create WASI config",
        )?;
        Ok(WasiConfig { ptr })
    }

    pub fn inherit_stdio(&mut self) {
        unsafe { sys::zwasm_wasi_config_inherit_stdio(self.ptr) };
    }

    pub fn inherit_env(&mut self) -> Result<(), Error> {
        let result = unsafe { sys::zwasm_wasi_config_inherit_env(self.ptr) };
        if result {
            Ok(())
        } else {
            Err(Error::Message("failed to inherit env".to_string()))
        }
    }

    pub fn set_args(&mut self, args: &[&str]) -> Result<(), Error> {
        let c_args = to_cstrings(args)?;
        let c_arg_ptrs: Vec<*const c_char> = c_args.iter().map(|s| s.as_ptr()).collect();
        unsafe { sys::zwasm_wasi_config_set_args(self.ptr, c_arg_ptrs.len(), c_arg_ptrs.as_ptr()) };
        Ok(())
    }

    pub fn set_envs(&mut self, envs: &[(&str, &str)]) -> Result<(), Error> {
        let (keys, vals): (Vec<&str>, Vec<&str>) = envs.iter().cloned().unzip();
        let c_keys = to_cstrings(&keys)?;
        let c_vals = to_cstrings(&vals)?;
        let c_key_ptrs: Vec<*const c_char> = c_keys.iter().map(|s| s.as_ptr()).collect();
        let c_val_ptrs: Vec<*const c_char> = c_vals.iter().map(|s| s.as_ptr()).collect();
        unsafe {
            sys::zwasm_wasi_config_set_envs(
                self.ptr,
                c_key_ptrs.len(),
                c_key_ptrs.as_ptr(),
                c_val_ptrs.as_ptr(),
            )
        };
        Ok(())
    }

    pub fn preopen_dir(&mut self, host_path: &str, guest_path: &str) -> Result<(), Error> {
        let c_host_path = CString::new(host_path)
            .map_err(|_| Error::Message("host path contains an interior null byte".to_string()))?;
        let c_guest_path = CString::new(guest_path)
            .map_err(|_| Error::Message("guest path contains an interior null byte".to_string()))?;
        let result = unsafe {
            sys::zwasm_wasi_config_preopen_dir(
                self.ptr,
                c_host_path.as_ptr(),
                c_guest_path.as_ptr(),
            )
        };
        if result {
            Ok(())
        } else {
            Err(Error::Message("failed to preopen directory".to_string()))
        }
    }
}

impl Default for WasiConfig {
    fn default() -> Self {
        Self::new().expect("failed to create default WASI config")
    }
}

impl Drop for WasiConfig {
    fn drop(&mut self) {
        unsafe { sys::zwasm_wasi_config_delete(self.ptr) };
    }
}

fn to_cstrings(strs: &[&str]) -> Result<Vec<CString>, Error> {
    strs.iter()
        .map(|s| CString::new(*s))
        .collect::<Result<_, _>>()
        .map_err(|_| Error::Message("string contains an interior null byte".to_string()))
}
