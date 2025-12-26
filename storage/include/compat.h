/* Windows compatibility layer for POSIX functions */
#ifndef MINSQL_COMPAT_H
#define MINSQL_COMPAT_H

#ifdef _WIN32

#include <windows.h>
#include <io.h>
#include <fcntl.h>
#include <sys/types.h>
#include <sys/stat.h>
#include <direct.h>

/* File operations */
#define O_RDWR _O_RDWR
#define O_CREAT _O_CREAT
#define O_APPEND _O_APPEND

#ifndef S_IRWXU
#define S_IRWXU 0700
#endif

#define open(path, flags, ...) _open(path, flags | _O_BINARY, _S_IREAD | _S_IWRITE)
#define close(fd) _close(fd)
#define read(fd, buf, count) _read(fd, buf, (unsigned int)(count))
#define write(fd, buf, count) _write(fd, buf, (unsigned int)(count))
#define lseek(fd, offset, whence) _lseek(fd, (long)(offset), whence)
#define fsync(fd) _commit(fd)
#define mkdir(path, mode) _mkdir(path)

typedef long off_t;
typedef int ssize_t;

/* pthread compatibility using Windows Critical Sections */
typedef CRITICAL_SECTION pthread_mutex_t;

#define pthread_mutex_init(mutex, attr) (InitializeCriticalSection(mutex), 0)
#define pthread_mutex_destroy(mutex) DeleteCriticalSection(mutex)
#define pthread_mutex_lock(mutex) EnterCriticalSection(mutex)
#define pthread_mutex_unlock(mutex) LeaveCriticalSection(mutex)

/* Memory mapping - use VirtualAlloc instead of mmap */
#define PROT_READ  0x1
#define PROT_WRITE 0x2
#define MAP_PRIVATE   0x02
#define MAP_ANONYMOUS 0x20
#define MAP_FAILED ((void*)-1)

static inline void* mmap(void* addr, size_t length, int prot, int flags, int fd, off_t offset) {
    (void)addr; (void)prot; (void)flags; (void)fd; (void)offset;
    void* ptr = VirtualAlloc(NULL, length, MEM_COMMIT | MEM_RESERVE, PAGE_READWRITE);
    return ptr ? ptr : MAP_FAILED;
}

static inline int munmap(void* addr, size_t length) {
    (void)length;
    return VirtualFree(addr, 0, MEM_RELEASE) ? 0 : -1;
}

#else
/* POSIX systems */
#include <unistd.h>
#include <fcntl.h>
#include <sys/mman.h>
#include <pthread.h>
#include <sys/stat.h>
#include <sys/types.h>
#endif

#endif /* MINSQL_COMPAT_H */
