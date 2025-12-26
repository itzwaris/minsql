#ifndef MINSQL_STORAGE_H
#define MINSQL_STORAGE_H

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

#define PAGE_SIZE 8192
#define WAL_BUFFER_SIZE 65536
#define BTREE_ORDER 128

// Forward declarations
typedef struct StorageHandle StorageHandle;
typedef struct BufferPool BufferPool;
typedef struct WAL WAL;
typedef struct BTreeIndex BTreeIndex;
typedef struct HashIndex HashIndex;
typedef struct BloomFilter BloomFilter;
typedef struct PageManager PageManager;
typedef struct Arena Arena;

typedef enum {
    WAL_INSERT = 1,
    WAL_UPDATE = 2,
    WAL_DELETE = 3,
    WAL_COMMIT = 4,
    WAL_ABORT = 5,
    WAL_CHECKPOINT = 6
} WALEntryType;

typedef enum {
    STORAGE_OK = 0,
    STORAGE_ERROR = 1,
    STORAGE_OOM = 2,
    STORAGE_IO_ERROR = 3,
    STORAGE_CORRUPTION = 4
} StorageResult;

// PageHeader must be defined first since Page uses it
typedef struct {
    uint32_t page_id;
    uint32_t checksum;
    uint16_t lower;
    uint16_t upper;
    uint16_t special;
    uint16_t flags;
    uint64_t lsn;
} PageHeader;

// Page struct with full definition
typedef struct Page {
    PageHeader header;
    bool dirty;
    uint16_t pin_count;
    uint8_t data[PAGE_SIZE - sizeof(PageHeader) - sizeof(bool) - sizeof(uint16_t)];
} Page;

typedef struct {
    uint64_t lsn;
    uint32_t transaction_id;
    uint64_t logical_time;
    uint16_t type;
    uint16_t length;
    uint8_t data[];
} WALEntry;

/* StorageHandle struct - full definition for cross-file access */
struct StorageHandle {
    char data_dir[256];
    BufferPool* buffer_pool;
    PageManager* page_manager;
    WAL* wal;
    Arena* arena;
};

StorageHandle* storage_init(const char* data_dir);
void storage_shutdown(StorageHandle* handle);

Page* storage_get_page(StorageHandle* handle, uint32_t page_id);
StorageResult storage_put_page(StorageHandle* handle, Page* page);
StorageResult storage_flush_page(StorageHandle* handle, Page* page);
void storage_release_page(StorageHandle* handle, Page* page);

uint64_t storage_wal_append(StorageHandle* handle, const WALEntry* entry);
StorageResult storage_wal_flush(StorageHandle* handle);
StorageResult storage_wal_replay(StorageHandle* handle);

BTreeIndex* storage_create_btree(StorageHandle* handle, const char* name);
void storage_destroy_btree(BTreeIndex* index);
StorageResult storage_btree_insert(BTreeIndex* index, const void* key, size_t key_len, uint64_t value);
bool storage_btree_search(BTreeIndex* index, const void* key, size_t key_len, uint64_t* value);
StorageResult storage_btree_delete(BTreeIndex* index, const void* key, size_t key_len);

HashIndex* storage_create_hash(StorageHandle* handle, const char* name, size_t num_buckets);
void storage_destroy_hash(HashIndex* index);
StorageResult storage_hash_insert(HashIndex* index, const void* key, size_t key_len, uint64_t value);
bool storage_hash_search(HashIndex* index, const void* key, size_t key_len, uint64_t* value);
StorageResult storage_hash_delete(HashIndex* index, const void* key, size_t key_len);

BloomFilter* storage_create_bloom(size_t num_bits, size_t num_hashes);
void storage_destroy_bloom(BloomFilter* filter);
void storage_bloom_insert(BloomFilter* filter, const void* key, size_t key_len);
bool storage_bloom_might_contain(BloomFilter* filter, const void* key, size_t key_len);

StorageResult storage_checkpoint(StorageHandle* handle);
StorageResult storage_recover(StorageHandle* handle);

void* storage_arena_alloc(StorageHandle* handle, size_t size);
void storage_arena_reset(StorageHandle* handle);

#ifdef __cplusplus
}
#endif

#endif
