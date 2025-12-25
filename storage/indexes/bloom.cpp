#include "../include/minsql_storage.h"
#include <vector>
#include <cstring>
#include <functional>

struct BloomFilter {
    std::vector<uint8_t> bits;
    size_t num_bits;
    size_t num_hashes;

    BloomFilter(size_t n_bits, size_t n_hashes) 
        : num_bits(n_bits), num_hashes(n_hashes) {
        bits.resize((num_bits + 7) / 8, 0);
    }

    size_t hash(const void* key, size_t key_len, size_t seed) const {
        size_t h = seed;
        const uint8_t* bytes = static_cast<const uint8_t*>(key);
        for (size_t i = 0; i < key_len; i++) {
            h = h * 31 + bytes[i];
        }
        return h % num_bits;
    }

    void set_bit(size_t bit_idx) {
        size_t byte_idx = bit_idx / 8;
        size_t bit_offset = bit_idx % 8;
        bits[byte_idx] |= (1 << bit_offset);
    }

    bool get_bit(size_t bit_idx) const {
        size_t byte_idx = bit_idx / 8;
        size_t bit_offset = bit_idx % 8;
        return (bits[byte_idx] & (1 << bit_offset)) != 0;
    }
};

extern "C" {

BloomFilter* storage_create_bloom(size_t num_bits, size_t num_hashes) {
    if (num_bits == 0) num_bits = 10000;
    if (num_hashes == 0) num_hashes = 3;
    return new BloomFilter(num_bits, num_hashes);
}

void storage_destroy_bloom(BloomFilter* filter) {
    delete filter;
}

void storage_bloom_insert(BloomFilter* filter, const void* key, size_t key_len) {
    for (size_t i = 0; i < filter->num_hashes; i++) {
        size_t bit_idx = filter->hash(key, key_len, i);
        filter->set_bit(bit_idx);
    }
}

bool storage_bloom_might_contain(BloomFilter* filter, const void* key, size_t key_len) {
    for (size_t i = 0; i < filter->num_hashes; i++) {
        size_t bit_idx = filter->hash(key, key_len, i);
        if (!filter->get_bit(bit_idx)) {
            return false;
        }
    }
    return true;
}

}
