#include "include/minsql_storage.h"
#include "include/compat.h"
#include <stdlib.h>
#include <string.h>

extern BufferPool* buffer_pool_create(size_t capacity);
extern void buffer_pool_destroy(BufferPool* pool);
extern Page* buffer_pool_get_page(BufferPool* pool, PageManager* pm, uint32_t page_id);
extern void buffer_pool_unpin_page(BufferPool* pool, Page* page);
extern StorageResult buffer_pool_flush_all(BufferPool* pool, PageManager* pm);
extern StorageResult buffer_pool_flush_page(BufferPool* pool, PageManager* pm, Page* page);

extern PageManager* page_manager_create(const char* data_dir);
extern void page_manager_destroy(PageManager* pm);
extern Page* page_manager_read(PageManager* pm, uint32_t page_id);
extern StorageResult page_manager_write(PageManager* pm, Page* page);
extern Page* page_manager_alloc(PageManager* pm);

extern WAL* wal_create(const char* data_dir);
extern void wal_destroy(WAL* wal);

extern Arena* arena_create(size_t capacity);
extern void arena_destroy(Arena* arena);

StorageHandle* storage_init(const char* data_dir) {
    StorageHandle* handle = malloc(sizeof(StorageHandle));
    if (!handle) return NULL;

    strncpy(handle->data_dir, data_dir, sizeof(handle->data_dir) - 1);
    handle->data_dir[sizeof(handle->data_dir) - 1] = '\0';

    mkdir(data_dir, 0755);

    handle->page_manager = page_manager_create(data_dir);
    if (!handle->page_manager) {
        free(handle);
        return NULL;
    }

    handle->buffer_pool = buffer_pool_create(1024);
    if (!handle->buffer_pool) {
        page_manager_destroy(handle->page_manager);
        free(handle);
        return NULL;
    }

    handle->wal = wal_create(data_dir);
    if (!handle->wal) {
        buffer_pool_destroy(handle->buffer_pool);
        page_manager_destroy(handle->page_manager);
        free(handle);
        return NULL;
    }

    handle->arena = arena_create(0);
    if (!handle->arena) {
        wal_destroy(handle->wal);
        buffer_pool_destroy(handle->buffer_pool);
        page_manager_destroy(handle->page_manager);
        free(handle);
        return NULL;
    }

    return handle;
}

void storage_shutdown(StorageHandle* handle) {
    if (!handle) return;

    buffer_pool_flush_all(handle->buffer_pool, handle->page_manager);
    storage_wal_flush(handle);

    arena_destroy(handle->arena);
    wal_destroy(handle->wal);
    buffer_pool_destroy(handle->buffer_pool);
    page_manager_destroy(handle->page_manager);
    free(handle);
}

Page* storage_get_page(StorageHandle* handle, uint32_t page_id) {
    return buffer_pool_get_page(handle->buffer_pool, handle->page_manager, page_id);
}

StorageResult storage_put_page(StorageHandle* handle, Page* page) {
    page->dirty = true;
    return STORAGE_OK;
}

StorageResult storage_flush_page(StorageHandle* handle, Page* page) {
    return buffer_pool_flush_page(handle->buffer_pool, handle->page_manager, page);
}

void storage_release_page(StorageHandle* handle, Page* page) {
    buffer_pool_unpin_page(handle->buffer_pool, page);
}

StorageResult storage_checkpoint(StorageHandle* handle) {
    StorageResult result = buffer_pool_flush_all(handle->buffer_pool, handle->page_manager);
    if (result != STORAGE_OK) {
        return result;
    }

    WALEntry checkpoint_entry;
    checkpoint_entry.type = WAL_CHECKPOINT;
    checkpoint_entry.transaction_id = 0;
    checkpoint_entry.logical_time = 0;
    checkpoint_entry.length = 0;

    storage_wal_append(handle, &checkpoint_entry);
    return storage_wal_flush(handle);
}

StorageResult storage_recover(StorageHandle* handle) {
    return storage_wal_replay(handle);
}
