#include "../include/minsql_storage.h"
#include <cstring>
#include <vector>
#include <functional>

struct HashBucket {
    std::vector<std::pair<std::vector<uint8_t>, uint64_t>> entries;
};

struct HashIndex {
    HashBucket* buckets;
    size_t num_buckets;
    char name[64];

    HashIndex(const char* idx_name, size_t num_bkt) : num_buckets(num_bkt) {
        buckets = new HashBucket[num_buckets];
        strncpy(name, idx_name, sizeof(name) - 1);
        name[sizeof(name) - 1] = '\0';
    }

    ~HashIndex() {
        delete[] buckets;
    }

    size_t hash(const void* key, size_t key_len) const {
        size_t h = 0;
        const uint8_t* bytes = static_cast<const uint8_t*>(key);
        for (size_t i = 0; i < key_len; i++) {
            h = h * 31 + bytes[i];
        }
        return h % num_buckets;
    }
};

extern "C" {

HashIndex* storage_create_hash(StorageHandle* handle, const char* name, size_t num_buckets) {
    if (num_buckets == 0) num_buckets = 1024;
    return new HashIndex(name, num_buckets);
}

void storage_destroy_hash(HashIndex* index) {
    delete index;
}

StorageResult storage_hash_insert(HashIndex* index, const void* key, size_t key_len, uint64_t value) {
    size_t bucket_idx = index->hash(key, key_len);
    HashBucket* bucket = &index->buckets[bucket_idx];

    std::vector<uint8_t> key_vec(static_cast<const uint8_t*>(key), 
                                  static_cast<const uint8_t*>(key) + key_len);

    for (auto& entry : bucket->entries) {
        if (entry.first == key_vec) {
            entry.second = value;
            return STORAGE_OK;
        }
    }

    bucket->entries.push_back({key_vec, value});
    return STORAGE_OK;
}

bool storage_hash_search(HashIndex* index, const void* key, size_t key_len, uint64_t* value) {
    size_t bucket_idx = index->hash(key, key_len);
    HashBucket* bucket = &index->buckets[bucket_idx];

    std::vector<uint8_t> key_vec(static_cast<const uint8_t*>(key), 
                                  static_cast<const uint8_t*>(key) + key_len);

    for (const auto& entry : bucket->entries) {
        if (entry.first == key_vec) {
            *value = entry.second;
            return true;
        }
    }

    return false;
}

StorageResult storage_hash_delete(HashIndex* index, const void* key, size_t key_len) {
    size_t bucket_idx = index->hash(key, key_len);
    HashBucket* bucket = &index->buckets[bucket_idx];

    std::vector<uint8_t> key_vec(static_cast<const uint8_t*>(key), 
                                  static_cast<const uint8_t*>(key) + key_len);

    for (auto it = bucket->entries.begin(); it != bucket->entries.end(); ++it) {
        if (it->first == key_vec) {
            bucket->entries.erase(it);
            return STORAGE_OK;
        }
    }

    return STORAGE_ERROR;
}

}
