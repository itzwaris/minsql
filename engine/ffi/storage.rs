use anyhow::Result;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::ptr;

#[repr(C)]
pub struct StorageHandle {
    ptr: *mut std::ffi::c_void,
}

extern "C" {
    fn storage_init(data_dir: *const c_char) -> *mut std::ffi::c_void;
    fn storage_shutdown(handle: *mut std::ffi::c_void);
    fn storage_checkpoint(handle: *mut std::ffi::c_void) -> i32;
    fn storage_recover(handle: *mut std::ffi::c_void) -> i32;
    fn storage_wal_flush(handle: *mut std::ffi::c_void) -> i32;
}

pub struct StorageEngine {
    handle: *mut std::ffi::c_void,
}

unsafe impl Send for StorageEngine {}
unsafe impl Sync for StorageEngine {}

impl StorageEngine {
    pub fn new(data_dir: &str) -> Result<Self> {
        let c_dir = CString::new(data_dir)?;
        let handle = unsafe { storage_init(c_dir.as_ptr()) };

        if handle.is_null() {
            anyhow::bail!("Failed to initialize storage engine");
        }

        Ok(Self { handle })
    }

    pub fn checkpoint(&self) -> Result<()> {
        let result = unsafe { storage_checkpoint(self.handle) };
        if result != 0 {
            anyhow::bail!("Checkpoint failed");
        }
        Ok(())
    }

    pub fn recover(&self) -> Result<()> {
        let result = unsafe { storage_recover(self.handle) };
        if result != 0 {
            anyhow::bail!("Recovery failed");
        }
        Ok(())
    }

    pub fn wal_flush(&self) -> Result<()> {
        let result = unsafe { storage_wal_flush(self.handle) };
        if result != 0 {
            anyhow::bail!("WAL flush failed");
        }
        Ok(())
    }

    pub fn wal_replay(&self) -> Result<()> {
        self.recover()
    }

    pub fn shutdown(&self) {
        unsafe { storage_shutdown(self.handle) };
    }
}

impl Drop for StorageEngine {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            unsafe { storage_shutdown(self.handle) };
        }
    }
}
