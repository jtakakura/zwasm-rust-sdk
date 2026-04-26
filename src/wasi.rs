use crate::error;
use crate::utils;
use zwasm_sys as sys;

/* ================================================================
 * WASI configuration
 * ================================================================ */

/// WASI (WebAssembly System Interface) configuration for zwasm modules.
///
/// Supports WASI Preview 1 and 2, full syscalls, stdio overrides, preopened directories,
/// and environment/argv configuration. Used to provide system interface capabilities to
/// Wasm modules running in the zwasm runtime.
pub struct WasiConfig {
    pub(crate) ptr: *mut sys::zwasm_wasi_config_t,
    argv: Vec<std::ffi::CString>,
    argv_ptrs: Vec<*const std::os::raw::c_char>,
    env_keys: Vec<std::ffi::CString>,
    env_key_lens: Vec<usize>,
    env_key_ptrs: Vec<*const std::os::raw::c_char>,
    env_vals: Vec<std::ffi::CString>,
    env_val_lens: Vec<usize>,
    env_val_ptrs: Vec<*const std::os::raw::c_char>,
    preopens: Vec<(std::ffi::CString, std::ffi::CString)>,
    preopen_fd_guest_paths: Vec<std::ffi::CString>,
    _not_send_sync: std::marker::PhantomData<std::rc::Rc<()>>,
}

impl WasiConfig {
    /// Creates a new WASI configuration for zwasm modules.
    ///
    /// Supports both WASI Preview 1 and 2. Use this to provide system interface capabilities to Wasm code.
    pub fn new() -> Result<Self, error::ZwasmError> {
        let ptr = unsafe { sys::zwasm_wasi_config_new() };

        if ptr.is_null() {
            Err(error::last_error()
                .unwrap_or_else(|| error::ZwasmError("Unknown error".to_string())))
        } else {
            Ok(WasiConfig {
                ptr,
                argv: Vec::new(),
                argv_ptrs: Vec::new(),
                env_keys: Vec::new(),
                env_key_lens: Vec::new(),
                env_key_ptrs: Vec::new(),
                env_vals: Vec::new(),
                env_val_lens: Vec::new(),
                env_val_ptrs: Vec::new(),
                preopens: Vec::new(),
                preopen_fd_guest_paths: Vec::new(),
                _not_send_sync: std::marker::PhantomData,
            })
        }
    }

    /// Sets the argv (command-line arguments) for the guest process.
    ///
    /// These will be visible to the Wasm module via WASI syscalls.
    pub fn set_argv(&mut self, argv: &[&str]) -> Result<(), error::ZwasmError> {
        self.argv = argv
            .iter()
            .map(|s| {
                std::ffi::CString::new(*s)
                    .map_err(|_| error::ZwasmError("argument contains NUL byte".into()))
            })
            .collect::<Result<Vec<_>, _>>()?;

        self.argv_ptrs = self.argv.iter().map(|s| s.as_ptr()).collect::<Vec<_>>();
        let argc = utils::to_u32_len(self.argv_ptrs.len())?;

        unsafe {
            sys::zwasm_wasi_config_set_argv(
                self.ptr,
                argc,
                if self.argv_ptrs.is_empty() {
                    std::ptr::null()
                } else {
                    self.argv_ptrs.as_ptr()
                },
            )
        };

        Ok(())
    }

    /// Sets environment variables for the guest process.
    ///
    /// These will be visible to the Wasm module via WASI syscalls.
    pub fn set_env(&mut self, env: &[(&str, &str)]) -> Result<(), error::ZwasmError> {
        let c_keys = env
            .iter()
            .map(|(key, _)| {
                std::ffi::CString::new(*key).map_err(|_| {
                    error::ZwasmError("environment variable key contains NUL byte".into())
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let c_key_lens = c_keys
            .iter()
            .map(|s| s.as_bytes().len())
            .collect::<Vec<_>>();
        let c_key_ptrs = c_keys.iter().map(|s| s.as_ptr()).collect::<Vec<_>>();
        let c_vals = env
            .iter()
            .map(|(_, val)| {
                std::ffi::CString::new(*val).map_err(|_| {
                    error::ZwasmError("environment variable value contains NUL byte".into())
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let c_val_lens = c_vals
            .iter()
            .map(|s| s.as_bytes().len())
            .collect::<Vec<_>>();
        let c_val_ptrs = c_vals.iter().map(|s| s.as_ptr()).collect::<Vec<_>>();
        let count = utils::to_u32_len(c_key_ptrs.len())?;

        self.env_keys = c_keys;
        self.env_key_lens = c_key_lens;
        self.env_key_ptrs = c_key_ptrs;
        self.env_vals = c_vals;
        self.env_val_lens = c_val_lens;
        self.env_val_ptrs = c_val_ptrs;

        unsafe {
            sys::zwasm_wasi_config_set_env(
                self.ptr,
                count,
                if self.env_key_ptrs.is_empty() {
                    std::ptr::null()
                } else {
                    self.env_key_ptrs.as_ptr()
                },
                if self.env_key_lens.is_empty() {
                    std::ptr::null()
                } else {
                    self.env_key_lens.as_ptr()
                },
                if self.env_val_ptrs.is_empty() {
                    std::ptr::null()
                } else {
                    self.env_val_ptrs.as_ptr()
                },
                if self.env_val_lens.is_empty() {
                    std::ptr::null()
                } else {
                    self.env_val_lens.as_ptr()
                },
            )
        };

        Ok(())
    }

    /// Preopens a host directory at a guest-visible path for the Wasm module.
    ///
    /// Grants the guest access to the specified host directory under the given guest path.
    ///
    /// # Safety
    /// This method grants the guest access to host filesystem resources. Callers must ensure
    /// the mapped host path is intended to be exposed to untrusted Wasm code.
    pub fn preopen_dir(
        &mut self,
        host_path: &str,
        guest_path: &str,
    ) -> Result<(), error::ZwasmError> {
        let c_host_path = std::ffi::CString::new(host_path)
            .map_err(|_| error::ZwasmError("host path contains NUL byte".into()))?;
        let c_guest_path = std::ffi::CString::new(guest_path)
            .map_err(|_| error::ZwasmError("guest path contains NUL byte".into()))?;

        self.preopens.push((c_host_path, c_guest_path));
        let (c_host_path, c_guest_path) = &self.preopens[self.preopens.len() - 1];
        let c_host_path_len = c_host_path.as_bytes().len();
        let c_guest_path_len = c_guest_path.as_bytes().len();

        unsafe {
            sys::zwasm_wasi_config_preopen_dir(
                self.ptr,
                c_host_path.as_ptr(),
                c_host_path_len,
                c_guest_path.as_ptr(),
                c_guest_path_len,
            )
        };

        Ok(())
    }

    /// Preopens an existing host file descriptor at a guest-visible path for the Wasm module.
    ///
    /// Useful for passing already-opened files or sockets into the guest.
    ///
    /// # Safety
    /// This method transfers or borrows host FD capabilities into the guest, depending on
    /// `ownership`. Callers must ensure FD lifetime/ownership policy matches the supplied flag
    /// and that exposing the FD to guest code is acceptable.
    pub fn preopen_fd(
        &mut self,
        host_fd: isize,
        guest_path: &str,
        kind: u8,
        ownership: u8,
    ) -> Result<(), error::ZwasmError> {
        let c_guest_path = std::ffi::CString::new(guest_path)
            .map_err(|_| error::ZwasmError("guest path contains NUL byte".into()))?;

        // Keep the guest path alive for the lifetime of the WASI config.
        self.preopen_fd_guest_paths.push(c_guest_path);
        let c_guest_path = self.preopen_fd_guest_paths.last().ok_or_else(|| {
            error::ZwasmError("internal error: failed to retain preopen fd guest path".into())
        })?;
        let c_guest_path_len = c_guest_path.as_bytes().len();

        unsafe {
            sys::zwasm_wasi_config_preopen_fd(
                self.ptr,
                host_fd,
                c_guest_path.as_ptr(),
                c_guest_path_len,
                kind,
                ownership,
            )
        };

        Ok(())
    }

    /// Overrides WASI stdio file descriptor mapping (stdin, stdout, stderr).
    ///
    /// Allows redirecting guest stdio to custom host file descriptors.
    ///
    /// # Safety
    /// The runtime may close or retain the supplied `host_fd` based on `ownership`. Callers
    /// must ensure ownership mode is correct to avoid double-close or leaked descriptors.
    pub fn set_stdio_fd(
        &mut self,
        wasi_fd: u32,
        host_fd: isize,
        ownership: u8,
    ) -> Result<(), error::ZwasmError> {
        unsafe {
            sys::zwasm_wasi_config_set_stdio_fd(self.ptr, wasi_fd, host_fd, ownership);
        }
        Ok(())
    }
}

impl Drop for WasiConfig {
    fn drop(&mut self) {
        unsafe {
            sys::zwasm_wasi_config_delete(self.ptr);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::os::fd::AsRawFd;

    use super::*;

    #[test]
    fn test_wasi_fd_api() {
        let mut wc = WasiConfig::new().expect("Failed to create WasiConfig");

        /* Set stdio overrides (use pipe fds) */
        let (stdout_read, stdout_write) =
            nix::unistd::pipe().expect("Failed to create pipe for stdout");

        /* Override stdout (fd 1) with write end of pipe, borrow mode */
        wc.set_stdio_fd(1, stdout_write.as_raw_fd() as isize, 0 /* borrow */)
            .expect("Failed to set stdio fd for stdout");

        /* Override stderr (fd 2) with write end as well, borrow mode */
        wc.set_stdio_fd(2, stdout_write.as_raw_fd() as isize, 0 /* borrow */)
            .expect("Failed to set stdio fd for stderr");

        /* Invalid fd index (>=3) should be silently ignored */
        wc.set_stdio_fd(5, stdout_read.as_raw_fd() as isize, 0)
            .expect("Failed to set stdio fd for invalid index");

        /* Add an FD-based preopen (borrow mode) */
        let dir_fd = nix::fcntl::open(
            ".",
            nix::fcntl::OFlag::O_RDONLY,
            nix::sys::stat::Mode::empty(),
        )
        .expect("Failed to open current directory for preopen fd");
        wc.preopen_fd(
            dir_fd.as_raw_fd() as isize,
            "/sandbox",
            1, /* dir */
            0, /* borrow */
        )
        .expect("Failed to add preopen fd");

        drop(wc);

        let written = nix::unistd::write(&stdout_write, b"ok")
            .expect("Failed to write to borrowed stdout pipe");
        assert_eq!(written, 2, "borrowed stdout pipe still writable");

        let _stat = nix::sys::stat::fstat(&dir_fd)
            .expect("borrowed dir fd still valid after WasiConfig drop");
    }
}
