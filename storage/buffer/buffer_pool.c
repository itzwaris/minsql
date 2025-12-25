#include "../include/minsql_storage.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <pthread.h>

#define DEFAULT_BUFFER_POOL_SIZE 1024

typedef struct BufferEntry {
    Page* page;
    uint32_t page_id;
    uint64_t last_access;
    bool valid;
} BufferEntry;

struct BufferPool {
    BufferEntry* entries;
    size_t capacity;
    size_t num_entries;
    pthread_mutex_t lock;
    uint64_t access_counter;
};

BufferPool* buffer_pool_create(size_t capacity) {
    BufferPool* pool = malloc(sizeof(BufferPool));
    if (!pool) return NULL;

    pool->entries = malloc(sizeof(BufferEntry) * capacity);
    if (!pool->entries) {
        free(pool);
        return NULL;
    }

    for (size_t i = 0; i < capacity; i++) {
        pool->entries[i].page = NULL;
        pool->entries[i].valid = false;
        pool->entries[i].last_access = 0;
    }

    pool->capacity = capacity;
    pool->num_entries = 0;
    pool->access_counter = 0;
    pthread_mutex_init(&pool->lock, NULL);

    return pool;
}

void buffer_pool_destroy(BufferPool* pool) {
    if (!pool) return;

    for (size_t i = 0; i < pool->capacity; i++) {
        if (pool->entries[i].valid && pool->entries[i].page) {
            free(pool->entries[i].page);
        }
    }

    free(pool->entries);
    pthread_mutex_destroy(&pool->lock);
    free(pool);
}

static int buffer_pool_find_slot(BufferPool* pool, uint32_t page_id) {
    for (size_t i = 0; i < pool->capacity; i++) {
        if (pool->entries[i].valid && pool->entries[i].page_id == page_id) {
            return i;
        }
    }
    return -1;
}

static int buffer_pool_find_victim(BufferPool* pool) {
    uint64_t min_access = UINT64_MAX;
    int victim_idx = -1;

    for (size_t i = 0; i < pool->capacity; i++) {
        if (!pool->entries[i].valid) {
            return i;
        }

        if (pool->entries[i].page->pin_count == 0 && 
            pool->entries[i].last_access < min_access) {
            min_access = pool->entries[i].last_access;
            victim_idx = i;
        }
    }

    return victim_idx;
}

Page* buffer_pool_get_page(BufferPool* pool, PageManager* pm, uint32_t page_id) {
    pthread_mutex_lock(&pool->lock);

    int slot = buffer_pool_find_slot(pool, page_id);
    
    if (slot >= 0) {
        pool->entries[slot].last_access = pool->access_counter++;
        pool->entries[slot].page->pin_count++;
        pthread_mutex_unlock(&pool->lock);
        return pool->entries[slot].page;
    }

    if (pool->num_entries >= pool->capacity) {
        int victim = buffer_pool_find_victim(pool);
        
        if (victim < 0) {
            pthread_mutex_unlock(&pool->lock);
            return NULL;
        }

        BufferEntry* entry = &pool->entries[victim];
        
        if (entry->page->dirty) {
            page_manager_write(pm, entry->page);
        }

        free(entry->page);
        entry->valid = false;
        pool->num_entries--;
    }

    Page* page = page_manager_read(pm, page_id);
    if (!page) {
        pthread_mutex_unlock(&pool->lock);
        return NULL;
    }

    int free_slot = buffer_pool_find_victim(pool);
    if (free_slot < 0) {
        free(page);
        pthread_mutex_unlock(&pool->lock);
        return NULL;
    }

    pool->entries[free_slot].page = page;
    pool->entries[free_slot].page_id = page_id;
    pool->entries[free_slot].valid = true;
    pool->entries[free_slot].last_access = pool->access_counter++;
    pool->num_entries++;

    page->pin_count = 1;

    pthread_mutex_unlock(&pool->lock);
    return page;
}

void buffer_pool_unpin_page(BufferPool* pool, Page* page) {
    pthread_mutex_lock(&pool->lock);

    for (size_t i = 0; i < pool->capacity; i++) {
        if (pool->entries[i].valid && pool->entries[i].page == page) {
            if (page->pin_count > 0) {
                page->pin_count--;
            }
            break;
        }
    }

    pthread_mutex_unlock(&pool->lock);
}

StorageResult buffer_pool_flush_page(BufferPool* pool, PageManager* pm, Page* page) {
    pthread_mutex_lock(&pool->lock);

    StorageResult result = page_manager_write(pm, page);
    page->dirty = false;

    pthread_mutex_unlock(&pool->lock);
    return result;
}

StorageResult buffer_pool_flush_all(BufferPool* pool, PageManager* pm) {
    pthread_mutex_lock(&pool->lock);

    for (size_t i = 0; i < pool->capacity; i++) {
        if (pool->entries[i].valid && pool->entries[i].page->dirty) {
            StorageResult result = page_manager_write(pm, pool->entries[i].page);
            if (result != STORAGE_OK) {
                pthread_mutex_unlock(&pool->lock);
                return result;
            }
        }
    }

    pthread_mutex_unlock(&pool->lock);
    return STORAGE_OK;
}
