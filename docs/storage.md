# Storage Engine

## Overview

minsql uses a page-oriented storage engine implemented in C (for predictability) and C++ (for complex data structures). The storage layer is accessed from Rust via FFI.

## Architecture

### Components

```
┌─────────────────────────────────────┐
│         Rust Execution Layer        │
└─────────────────┬───────────────────┘
                  │ FFI
┌─────────────────▼───────────────────┐
│           Buffer Pool               │
├─────────────────────────────────────┤
│          Page Manager               │
├─────────────────────────────────────┤
│          WAL Writer/Reader          │
├─────────────────────────────────────┤
│          Index Structures           │
│        (B-Tree, Hash, Bloom)        │
├─────────────────────────────────────┤
│           Compression               │
└─────────────────┬───────────────────┘
                  │
┌─────────────────▼───────────────────┐
│          Disk/File System           │
└─────────────────────────────────────┘
```

## Page Layout

### Page Structure

Pages are fixed-size units of storage (default 8KB):

```
┌────────────────────────────────────┐
│         Page Header (24 bytes)     │
├────────────────────────────────────┤
│         Line Pointers (variable)   │
├────────────────────────────────────┤
│         Free Space                 │
├────────────────────────────────────┤
│         Tuples (variable)          │
└────────────────────────────────────┘
```

### Page Header

```c
typedef struct PageHeader {
    uint32_t page_id;
    uint32_t checksum;
    uint16_t lower;      // End of line pointers
    uint16_t upper;      // Start of tuples
    uint16_t special;    // Special space offset
    uint16_t flags;
    uint64_t lsn;        // Log sequence number
} PageHeader;
```

### Line Pointers

Indirection layer for tuple locations:

```c
typedef struct LinePointer {
    uint16_t offset;
    uint16_t length;
    uint16_t flags;
} LinePointer;
```

Flags:
- `LP_NORMAL`: Normal tuple
- `LP_REDIRECT`: Redirects to another line pointer
- `LP_DEAD`: Tuple is deleted
- `LP_UNUSED`: Line pointer is unused

### Tuple Format

```c
typedef struct TupleHeader {
    uint32_t t_xmin;     // Insert transaction ID
    uint32_t t_xmax;     // Delete transaction ID
    uint16_t t_infomask; // Flags
    uint8_t t_hoff;      // Header size
    uint8_t t_bits[FLEXIBLE_ARRAY_MEMBER]; // Null bitmap
} TupleHeader;
```

After the header comes the actual column data.

## Buffer Pool

### Production Storage Integration

minsql now has **full production-level storage integration** with the following operations:

#### Table Management

```rust
pub fn create_table(&self, table_name: &str, schema: &str) -> Result<()>
```

Creates a new table with schema stored in the system catalog:
- Parses schema JSON containing column definitions
- Allocates initial storage pages
- Creates system catalog entries
- Sets up indexes for primary keys
- Returns success after checkpoint

**Example Schema**:
```json
{
  "id": {
    "name": "id",
    "type": "Integer",
    "nullable": false,
    "primary_key": true
  },
  "name": {
    "name": "name",
    "type": "Text",
    "nullable": false,
    "primary_key": false
  }
}
```

#### Row Operations

**Insert**:
```rust
pub fn insert_row(&self, table_name: &str, data: &[u8]) -> Result<u64>
```

- Finds free space in table pages
- Writes serialized tuple data
- Updates indexes automatically
- Logs to WAL for durability
- Returns unique row ID

**Update**:
```rust
pub fn update_rows(&self, table_name: &str, predicate: &str, data: &[u8]) -> Result<usize>
```

- Scans table for matching rows based on predicate
- Applies updates to each matched row
- Maintains index consistency
- Logs all changes to WAL
- Returns count of updated rows

**Delete**:
```rust
pub fn delete_rows(&self, table_name: &str, predicate: &str) -> Result<usize>
```

- Scans table for matching rows
- Marks rows as deleted or removes them
- Updates free space map
- Maintains index consistency
- Logs to WAL for crash recovery
- Returns count of deleted rows

### Write-Ahead Logging (WAL)

**Production Features**:
- All write operations logged before page modification
- Flush operations guarantee durability
- Checkpoint creates consistent state on disk
- Recovery replays WAL after crash

```rust
pub fn wal_flush(&self) -> Result<()>  // Flush WAL to disk
pub fn checkpoint(&self) -> Result<()>  // Create consistent checkpoint
pub fn recover(&self) -> Result<()>    // Replay WAL after crash
```

### Data Durability Guarantees

**INSERT Operations**:
1. WAL flush before insert
2. Write data to storage pages
3. WAL flush after batch insert
4. Result: Crash-safe with zero data loss

**UPDATE/DELETE Operations**:
1. Scan and identify target rows
2. Log changes to WAL
3. Apply modifications
4. WAL flush for durability
5. Result: Transactional consistency

**CREATE TABLE Operations**:
1. Build schema metadata
2. Log schema to WAL
3. Create system catalog entry
4. Checkpoint for durability
5. Result: Schema persists across crashes

### Buffer Pool Manager

Caches pages in memory:

```c
typedef struct BufferPool {
    Page* pages;
    size_t capacity;
    HashTable* page_table;
    LRUCache* lru;
    pthread_mutex_t lock;
} BufferPool;
```

### Buffer Replacement

LRU-based eviction:

```c
Page* buffer_pool_get_page(BufferPool* pool, uint32_t page_id) {
    Page* page = hash_table_lookup(pool->page_table, page_id);
    
    if (page != NULL) {
        lru_cache_touch(pool->lru, page);
        return page;
    }
    
    // Page not in cache, need to fetch
    if (buffer_pool_is_full(pool)) {
        Page* victim = lru_cache_evict(pool->lru);
        if (victim->dirty) {
            flush_page(victim);
        }
    }
    
    page = read_page_from_disk(page_id);
    buffer_pool_insert(pool, page);
    return page;
}
```

### Page Pinning

Pages can be pinned to prevent eviction:

```c
void buffer_pool_pin_page(BufferPool* pool, Page* page) {
    page->pin_count++;
}

void buffer_pool_unpin_page(BufferPool* pool, Page* page) {
    page->pin_count--;
}
```

Pinned pages (pin_count > 0) cannot be evicted.

## Write-Ahead Log (WAL)

### WAL Entry Format

```c
typedef struct WALEntry {
    uint64_t lsn;
    uint32_t transaction_id;
    uint64_t logical_time;
    uint16_t type;
    uint16_t length;
    uint8_t data[FLEXIBLE_ARRAY_MEMBER];
} WALEntry;
```

### WAL Entry Types

- `WAL_INSERT`: Insert tuple
- `WAL_UPDATE`: Update tuple
- `WAL_DELETE`: Delete tuple
- `WAL_COMMIT`: Transaction commit
- `WAL_ABORT`: Transaction abort
- `WAL_CHECKPOINT`: Checkpoint marker

### WAL Writer

```c
uint64_t wal_append(WAL* wal, WALEntry* entry) {
    pthread_mutex_lock(&wal->lock);
    
    uint64_t lsn = wal->next_lsn;
    entry->lsn = lsn;
    
    // Write to buffer
    memcpy(wal->buffer + wal->buffer_pos, entry, entry->length);
    wal->buffer_pos += entry->length;
    wal->next_lsn += entry->length;
    
    // Flush if buffer is full
    if (wal->buffer_pos >= WAL_BUFFER_SIZE) {
        wal_flush(wal);
    }
    
    pthread_mutex_unlock(&wal->lock);
    return lsn;
}
```

### WAL Flushing

```c
void wal_flush(WAL* wal) {
    write(wal->fd, wal->buffer, wal->buffer_pos);
    fsync(wal->fd);
    wal->buffer_pos = 0;
}
```

Ensures durability with `fsync`.

### WAL Replay

```c
void wal_replay(WAL* wal, BufferPool* pool) {
    WALEntry* entry;
    
    while ((entry = wal_read_next(wal)) != NULL) {
        switch (entry->type) {
            case WAL_INSERT:
                replay_insert(pool, entry);
                break;
            case WAL_UPDATE:
                replay_update(pool, entry);
                break;
            case WAL_DELETE:
                replay_delete(pool, entry);
                break;
            case WAL_COMMIT:
                replay_commit(entry);
                break;
            case WAL_ABORT:
                replay_abort(entry);
                break;
        }
    }
}
```

Replay is idempotent: replaying the same WAL multiple times produces identical state.

## Indexes

### B-Tree Index (C++)

```cpp
template<typename Key, typename Value>
class BTree {
private:
    struct Node {
        bool is_leaf;
        size_t num_keys;
        Key keys[BTREE_ORDER];
        union {
            Node* children[BTREE_ORDER + 1];
            Value values[BTREE_ORDER];
        };
    };
    
    Node* root;
    
public:
    void insert(const Key& key, const Value& value);
    bool search(const Key& key, Value& result);
    void remove(const Key& key);
    
private:
    void split_child(Node* parent, size_t index);
    void merge_children(Node* parent, size_t index);
};
```

### Hash Index (C++)

```cpp
template<typename Key, typename Value>
class HashIndex {
private:
    struct Bucket {
        std::vector<std::pair<Key, Value>> entries;
    };
    
    std::vector<Bucket> buckets;
    size_t num_buckets;
    
    size_t hash(const Key& key) const {
        return std::hash<Key>{}(key) % num_buckets;
    }
    
public:
    void insert(const Key& key, const Value& value) {
        size_t bucket_idx = hash(key);
        buckets[bucket_idx].entries.push_back({key, value});
    }
    
    bool search(const Key& key, Value& result) {
        size_t bucket_idx = hash(key);
        for (auto& [k, v] : buckets[bucket_idx].entries) {
            if (k == key) {
                result = v;
                return true;
            }
        }
        return false;
    }
};
```

### Bloom Filter (C++)

```cpp
class BloomFilter {
private:
    std::vector<uint8_t> bits;
    size_t num_bits;
    size_t num_hashes;
    
    size_t hash(const std::string& key, size_t seed) const {
        return std::hash<std::string>{}(key + std::to_string(seed)) % num_bits;
    }
    
public:
    void insert(const std::string& key) {
        for (size_t i = 0; i < num_hashes; i++) {
            size_t bit_idx = hash(key, i);
            bits[bit_idx / 8] |= (1 << (bit_idx % 8));
        }
    }
    
    bool might_contain(const std::string& key) const {
        for (size_t i = 0; i < num_hashes; i++) {
            size_t bit_idx = hash(key, i);
            if ((bits[bit_idx / 8] & (1 << (bit_idx % 8))) == 0) {
                return false;
            }
        }
        return true;
    }
};
```

## Compression

### LZ4 Compression

Tuples can be compressed:

```cpp
std::vector<uint8_t> compress_tuple(const uint8_t* data, size_t size) {
    size_t max_compressed_size = LZ4_compressBound(size);
    std::vector<uint8_t> compressed(max_compressed_size);
    
    int compressed_size = LZ4_compress_default(
        reinterpret_cast<const char*>(data),
        reinterpret_cast<char*>(compressed.data()),
        size,
        max_compressed_size
    );
    
    compressed.resize(compressed_size);
    return compressed;
}

std::vector<uint8_t> decompress_tuple(const uint8_t* data, size_t compressed_size, size_t original_size) {
    std::vector<uint8_t> decompressed(original_size);
    
    LZ4_decompress_safe(
        reinterpret_cast<const char*>(data),
        reinterpret_cast<char*>(decompressed.data()),
        compressed_size,
        original_size
    );
    
    return decompressed;
}
```

## Free Space Management

### Free Space Map (FSM)

Tracks available space in pages:

```c
typedef struct FSM {
    uint8_t* pages;
    size_t num_pages;
} FSM;

void fsm_set_free_space(FSM* fsm, uint32_t page_id, uint16_t free_space) {
    fsm->pages[page_id] = free_space / 32;  // Store in units of 32 bytes
}

uint32_t fsm_find_page_with_space(FSM* fsm, uint16_t required_space) {
    uint8_t required_units = (required_space + 31) / 32;
    
    for (size_t i = 0; i < fsm->num_pages; i++) {
        if (fsm->pages[i] >= required_units) {
            return i;
        }
    }
    
    return INVALID_PAGE_ID;
}
```

## Crash Recovery

### Checkpointing

Periodic checkpoints reduce recovery time:

```c
void create_checkpoint(BufferPool* pool, WAL* wal) {
    // Flush all dirty pages
    for (size_t i = 0; i < pool->capacity; i++) {
        Page* page = &pool->pages[i];
        if (page->dirty) {
            write_page_to_disk(page);
            page->dirty = false;
        }
    }
    
    // Write checkpoint marker to WAL
    WALEntry checkpoint = {
        .type = WAL_CHECKPOINT,
        .lsn = wal->next_lsn,
    };
    wal_append(wal, &checkpoint);
    wal_flush(wal);
}
```

### Recovery Process

```c
void recover(BufferPool* pool, WAL* wal) {
    // Find last checkpoint
    uint64_t checkpoint_lsn = wal_find_last_checkpoint(wal);
    
    // Replay WAL from checkpoint
    wal_seek(wal, checkpoint_lsn);
    wal_replay(wal, pool);
    
    // Flush all dirty pages
    buffer_pool_flush_all(pool);
}
```

## Memory Management

### Arena Allocator

Deterministic memory allocation:

```c
typedef struct Arena {
    uint8_t* base;
    size_t capacity;
    size_t offset;
} Arena;

void* arena_alloc(Arena* arena, size_t size) {
    if (arena->offset + size > arena->capacity) {
        return NULL;
    }
    
    void* ptr = arena->base + arena->offset;
    arena->offset += size;
    return ptr;
}

void arena_reset(Arena* arena) {
    arena->offset = 0;
}
```

## FFI Boundary

### C API Header

```c
// Storage engine initialization
void* storage_init(const char* data_dir);
void storage_shutdown(void* handle);

// Page operations
void* storage_get_page(void* handle, uint32_t page_id);
void storage_put_page(void* handle, void* page);
void storage_flush_page(void* handle, void* page);

// WAL operations
uint64_t storage_wal_append(void* handle, const void* data, size_t length);
void storage_wal_flush(void* handle);

// Index operations
void* storage_create_btree_index(void* handle, const char* name);
void storage_btree_insert(void* index, const void* key, size_t key_len, uint64_t value);
bool storage_btree_search(void* index, const void* key, size_t key_len, uint64_t* value);
```

### Rust FFI Bindings

```rust
#[repr(C)]
pub struct StorageHandle {
    ptr: *mut c_void,
}

extern "C" {
    fn storage_init(data_dir: *const c_char) -> *mut c_void;
    fn storage_shutdown(handle: *mut c_void);
    fn storage_get_page(handle: *mut c_void, page_id: u32) -> *mut c_void;
    fn storage_put_page(handle: *mut c_void, page: *mut c_void);
}
```

## Performance Characteristics

### Buffer Pool Hit Rate

Target: >95% for typical workloads

### WAL Throughput

Target: >50MB/s sequential writes

### B-Tree Lookup

Target: O(log n) with 3-4 levels for millions of keys

### Page Scan

Target: ~100k pages/second sequential scan

## Monitoring

### Storage Metrics

```c
typedef struct StorageMetrics {
    uint64_t pages_read;
    uint64_t pages_written;
    uint64_t buffer_hits;
    uint64_t buffer_misses;
    uint64_t wal_writes;
    uint64_t checkpoints;
} StorageMetrics;
```

## Future Enhancements

### Compression

- Columnar compression for OLAP workloads
- Dictionary encoding
- Run-length encoding

### Advanced Indexes

- GiST (Generalized Search Tree)
- GIN (Generalized Inverted Index)
- Spatial indexes

### Parallel I/O

- Multi-threaded page reads
- Asynchronous I/O
- Direct I/O bypassing page cache
