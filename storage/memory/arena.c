#include "../include/minsql_storage.h"
#include <stdlib.h>
#include <string.h>
#include <sys/mman.h>

#define ARENA_CAPACITY (16 * 1024 * 1024)

typedef struct Arena {
    uint8_t* base;
    size_t capacity;
    size_t offset;
} Arena;

Arena* arena_create(size_t capacity) {
    Arena* arena = malloc(sizeof(Arena));
    if (!arena) return NULL;

    if (capacity == 0) {
        capacity = ARENA_CAPACITY;
    }

    arena->base = mmap(NULL, capacity, PROT_READ | PROT_WRITE, MAP_PRIVATE | MAP_ANONYMOUS, -1, 0);
    if (arena->base == MAP_FAILED) {
        free(arena);
        return NULL;
    }

    arena->capacity = capacity;
    arena->offset = 0;

    return arena;
}

void arena_destroy(Arena* arena) {
    if (!arena) return;
    munmap(arena->base, arena->capacity);
    free(arena);
}

void* arena_alloc(Arena* arena, size_t size) {
    size_t aligned_size = (size + 7) & ~7;

    if (arena->offset + aligned_size > arena->capacity) {
        return NULL;
    }

    void* ptr = arena->base + arena->offset;
    arena->offset += aligned_size;

    return ptr;
}

void arena_reset(Arena* arena) {
    arena->offset = 0;
}

void* storage_arena_alloc(StorageHandle* handle, size_t size) {
    return arena_alloc(handle->arena, size);
}

void storage_arena_reset(StorageHandle* handle) {
    arena_reset(handle->arena);
}
