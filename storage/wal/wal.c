#include "../include/minsql_storage.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <fcntl.h>
#include <pthread.h>
#include <errno.h>

struct WAL {
    int fd;
    char* buffer;
    size_t buffer_pos;
    size_t buffer_capacity;
    uint64_t next_lsn;
    pthread_mutex_t lock;
    char filepath[256];
};

WAL* wal_create(const char* data_dir) {
    WAL* wal = malloc(sizeof(WAL));
    if (!wal) return NULL;

    snprintf(wal->filepath, sizeof(wal->filepath), "%s/wal.log", data_dir);
    
    wal->fd = open(wal->filepath, O_RDWR | O_CREAT | O_APPEND, 0644);
    if (wal->fd < 0) {
        free(wal);
        return NULL;
    }

    wal->buffer = malloc(WAL_BUFFER_SIZE);
    if (!wal->buffer) {
        close(wal->fd);
        free(wal);
        return NULL;
    }

    wal->buffer_pos = 0;
    wal->buffer_capacity = WAL_BUFFER_SIZE;
    wal->next_lsn = 0;
    pthread_mutex_init(&wal->lock, NULL);

    off_t file_size = lseek(wal->fd, 0, SEEK_END);
    if (file_size > 0) {
        wal->next_lsn = file_size;
    }

    return wal;
}

void wal_destroy(WAL* wal) {
    if (!wal) return;

    wal_flush_internal(wal);
    pthread_mutex_destroy(&wal->lock);
    free(wal->buffer);
    close(wal->fd);
    free(wal);
}

static StorageResult wal_flush_internal(WAL* wal) {
    if (wal->buffer_pos == 0) return STORAGE_OK;

    ssize_t written = write(wal->fd, wal->buffer, wal->buffer_pos);
    if (written < 0) {
        return STORAGE_IO_ERROR;
    }

    if (fsync(wal->fd) < 0) {
        return STORAGE_IO_ERROR;
    }

    wal->buffer_pos = 0;
    return STORAGE_OK;
}

uint64_t storage_wal_append(StorageHandle* handle, const WALEntry* entry) {
    WAL* wal = handle->wal;
    
    pthread_mutex_lock(&wal->lock);

    uint64_t lsn = wal->next_lsn;
    
    size_t entry_size = sizeof(WALEntry) + entry->length;
    
    if (wal->buffer_pos + entry_size > wal->buffer_capacity) {
        StorageResult result = wal_flush_internal(wal);
        if (result != STORAGE_OK) {
            pthread_mutex_unlock(&wal->lock);
            return 0;
        }
    }

    WALEntry* buffered_entry = (WALEntry*)(wal->buffer + wal->buffer_pos);
    memcpy(buffered_entry, entry, sizeof(WALEntry));
    buffered_entry->lsn = lsn;
    memcpy(buffered_entry->data, entry->data, entry->length);

    wal->buffer_pos += entry_size;
    wal->next_lsn += entry_size;

    pthread_mutex_unlock(&wal->lock);
    return lsn;
}

StorageResult storage_wal_flush(StorageHandle* handle) {
    WAL* wal = handle->wal;
    
    pthread_mutex_lock(&wal->lock);
    StorageResult result = wal_flush_internal(wal);
    pthread_mutex_unlock(&wal->lock);
    
    return result;
}

StorageResult storage_wal_replay(StorageHandle* handle) {
    WAL* wal = handle->wal;
    
    off_t file_size = lseek(wal->fd, 0, SEEK_END);
    if (file_size <= 0) return STORAGE_OK;

    lseek(wal->fd, 0, SEEK_SET);

    uint8_t* replay_buffer = malloc(file_size);
    if (!replay_buffer) return STORAGE_OOM;

    ssize_t bytes_read = read(wal->fd, replay_buffer, file_size);
    if (bytes_read != file_size) {
        free(replay_buffer);
        return STORAGE_IO_ERROR;
    }

    size_t offset = 0;
    while (offset < file_size) {
        WALEntry* entry = (WALEntry*)(replay_buffer + offset);
        
        size_t entry_size = sizeof(WALEntry) + entry->length;
        if (offset + entry_size > file_size) {
            break;
        }

        switch (entry->type) {
            case WAL_INSERT:
                break;
            case WAL_UPDATE:
                break;
            case WAL_DELETE:
                break;
            case WAL_COMMIT:
                break;
            case WAL_ABORT:
                break;
            case WAL_CHECKPOINT:
                break;
        }

        offset += entry_size;
    }

    free(replay_buffer);
    return STORAGE_OK;
}
