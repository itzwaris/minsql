use anyhow::Result;
use std::ffi::CString;
use std::os::raw::c_char;

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
    fn storage_create_table(
        handle: *mut std::ffi::c_void,
        table_name: *const c_char,
        schema_json: *const c_char,
    ) -> i32;
    fn storage_insert_row(
        handle: *mut std::ffi::c_void,
        table_name: *const c_char,
        data: *const u8,
        data_len: usize,
        row_id_out: *mut u64,
    ) -> i32;
    fn storage_update_rows(
        handle: *mut std::ffi::c_void,
        table_name: *const c_char,
        predicate: *const c_char,
        data: *const u8,
        data_len: usize,
        count_out: *mut usize,
    ) -> i32;
    fn storage_delete_rows(
        handle: *mut std::ffi::c_void,
        table_name: *const c_char,
        predicate: *const c_char,
        count_out: *mut usize,
    ) -> i32;
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

    pub fn create_table(&self, table_name: &str, schema: &str) -> Result<()> {
        tracing::debug!("Creating table '{}' with schema: {}", table_name, schema);

        let c_table_name = CString::new(table_name)?;
        let c_schema = CString::new(schema)?;

        let result =
            unsafe { storage_create_table(self.handle, c_table_name.as_ptr(), c_schema.as_ptr()) };

        if result != 0 {
            anyhow::bail!(
                "Failed to create table '{}': error code {}",
                table_name,
                result
            );
        }
        self.checkpoint()?;

        tracing::info!(
            "Successfully created table '{}' in system catalog",
            table_name
        );
        Ok(())
    }

    pub fn insert_row(&self, table_name: &str, data: &[u8]) -> Result<u64> {
        tracing::debug!(
            "Inserting row into table '{}', data size: {} bytes",
            table_name,
            data.len()
        );

        let c_table_name = CString::new(table_name)?;
        let mut row_id: u64 = 0;

        let result = unsafe {
            storage_insert_row(
                self.handle,
                c_table_name.as_ptr(),
                data.as_ptr(),
                data.len(),
                &mut row_id,
            )
        };

        if result != 0 {
            anyhow::bail!(
                "Failed to insert row into '{}': error code {}",
                table_name,
                result
            );
        }
        self.wal_flush()?;

        tracing::debug!(
            "Successfully inserted row with ID {} into '{}'",
            row_id,
            table_name
        );
        Ok(row_id)
    }

    pub fn update_rows(&self, table_name: &str, predicate: &str, data: &[u8]) -> Result<usize> {
        tracing::debug!(
            "Updating rows in table '{}' matching: {}",
            table_name,
            predicate
        );

        let c_table_name = CString::new(table_name)?;
        let c_predicate = CString::new(predicate)?;
        let mut count: usize = 0;

        let result = unsafe {
            storage_update_rows(
                self.handle,
                c_table_name.as_ptr(),
                c_predicate.as_ptr(),
                data.as_ptr(),
                data.len(),
                &mut count,
            )
        };

        if result != 0 {
            anyhow::bail!(
                "Failed to update rows in '{}': error code {}",
                table_name,
                result
            );
        }
        self.wal_flush()?;

        tracing::info!("Successfully updated {} rows in '{}'", count, table_name);
        Ok(count)
    }

    pub fn delete_rows(&self, table_name: &str, predicate: &str) -> Result<usize> {
        tracing::debug!(
            "Deleting rows from table '{}' matching: {}",
            table_name,
            predicate
        );

        let c_table_name = CString::new(table_name)?;
        let c_predicate = CString::new(predicate)?;
        let mut count: usize = 0;

        let result = unsafe {
            storage_delete_rows(
                self.handle,
                c_table_name.as_ptr(),
                c_predicate.as_ptr(),
                &mut count,
            )
        };

        if result != 0 {
            anyhow::bail!(
                "Failed to delete rows from '{}': error code {}",
                table_name,
                result
            );
        }

        self.wal_flush()?;

        tracing::info!("Successfully deleted {} rows from '{}'", count, table_name);
        Ok(count)
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
