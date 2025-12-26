#include "../include/minsql_storage.h"
#include "../include/compat.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

typedef struct {
    uint16_t offset;
    uint16_t length;
    uint16_t flags;
} LinePointer;

/* PageManager struct definition */
struct PageManager {
    int fd;
    char filepath[256];
    uint32_t num_pages;
};

PageManager* page_manager_create(const char* data_dir) {
    PageManager* pm = malloc(sizeof(PageManager));
    if (!pm) return NULL;

    snprintf(pm->filepath, sizeof(pm->filepath), "%s/pages.dat", data_dir);
    
    pm->fd = open(pm->filepath, O_RDWR | O_CREAT, 0644);
    if (pm->fd < 0) {
        free(pm);
        return NULL;
    }

    off_t file_size = lseek(pm->fd, 0, SEEK_END);
    pm->num_pages = file_size / PAGE_SIZE;

    return pm;
}

void page_manager_destroy(PageManager* pm) {
    if (!pm) return;
    close(pm->fd);
    free(pm);
}

Page* page_manager_read(PageManager* pm, uint32_t page_id) {
    if (page_id >= pm->num_pages) {
        return NULL;
    }

    Page* page = malloc(sizeof(Page));
    if (!page) return NULL;

    off_t offset = page_id * PAGE_SIZE;
    if (lseek(pm->fd, offset, SEEK_SET) != offset) {
        free(page);
        return NULL;
    }

    ssize_t bytes_read = read(pm->fd, page, PAGE_SIZE);
    if (bytes_read != PAGE_SIZE) {
        free(page);
        return NULL;
    }

    page->dirty = false;
    page->pin_count = 1;

    return page;
}

StorageResult page_manager_write(PageManager* pm, Page* page) {
    uint32_t page_id = page->header.page_id;
    off_t offset = page_id * PAGE_SIZE;

    if (lseek(pm->fd, offset, SEEK_SET) != offset) {
        return STORAGE_IO_ERROR;
    }

    ssize_t written = write(pm->fd, page, PAGE_SIZE);
    if (written != PAGE_SIZE) {
        return STORAGE_IO_ERROR;
    }

    if (fsync(pm->fd) < 0) {
        return STORAGE_IO_ERROR;
    }

    page->dirty = false;
    return STORAGE_OK;
}

Page* page_manager_alloc(PageManager* pm) {
    Page* page = malloc(sizeof(Page));
    if (!page) return NULL;

    memset(page, 0, sizeof(Page));
    
    page->header.page_id = pm->num_pages;
    page->header.lower = sizeof(PageHeader);
    page->header.upper = PAGE_SIZE;
    page->header.flags = 0;
    page->header.lsn = 0;
    page->dirty = true;
    page->pin_count = 1;

    pm->num_pages++;

    if (lseek(pm->fd, page->header.page_id * PAGE_SIZE, SEEK_SET) < 0) {
        free(page);
        return NULL;
    }

    if (write(pm->fd, page, PAGE_SIZE) != PAGE_SIZE) {
        free(page);
        return NULL;
    }

    return page;
}

uint16_t page_get_free_space(Page* page) {
    return page->header.upper - page->header.lower;
}

StorageResult page_add_tuple(Page* page, const void* tuple_data, uint16_t tuple_size) {
    uint16_t free_space = page_get_free_space(page);
    uint16_t required = tuple_size + sizeof(LinePointer);

    if (free_space < required) {
        return STORAGE_ERROR;
    }

    LinePointer* lp = (LinePointer*)(((uint8_t*)page) + page->header.lower);
    lp->offset = page->header.upper - tuple_size;
    lp->length = tuple_size;
    lp->flags = 0;

    memcpy(((uint8_t*)page) + lp->offset, tuple_data, tuple_size);

    page->header.lower += sizeof(LinePointer);
    page->header.upper -= tuple_size;
    page->dirty = true;

    return STORAGE_OK;
}

void* page_get_tuple(Page* page, uint16_t slot) {
    uint16_t num_slots = (page->header.lower - sizeof(PageHeader)) / sizeof(LinePointer);
    
    if (slot >= num_slots) {
        return NULL;
    }

    LinePointer* lp = (LinePointer*)(((uint8_t*)page) + sizeof(PageHeader) + slot * sizeof(LinePointer));
    
    if (lp->flags & 0x01) {
        return NULL;
    }

    return ((uint8_t*)page) + lp->offset;
}

StorageResult page_delete_tuple(Page* page, uint16_t slot) {
    uint16_t num_slots = (page->header.lower - sizeof(PageHeader)) / sizeof(LinePointer);
    
    if (slot >= num_slots) {
        return STORAGE_ERROR;
    }

    LinePointer* lp = (LinePointer*)(((uint8_t*)page) + sizeof(PageHeader) + slot * sizeof(LinePointer));
    lp->flags |= 0x01;
    page->dirty = true;

    return STORAGE_OK;
}
