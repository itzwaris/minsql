#include "../include/minsql_storage.h"
#include <cstring>
#include <algorithm>
#include <vector>

struct BTreeNode {
    bool is_leaf;
    size_t num_keys;
    std::vector<uint8_t> keys[BTREE_ORDER];
    union {
        BTreeNode* children[BTREE_ORDER + 1];
        uint64_t values[BTREE_ORDER];
    };

    BTreeNode(bool leaf) : is_leaf(leaf), num_keys(0) {
        if (!is_leaf) {
            std::memset(children, 0, sizeof(children));
        } else {
            std::memset(values, 0, sizeof(values));
        }
    }

    ~BTreeNode() {
        if (!is_leaf) {
            for (size_t i = 0; i <= num_keys; i++) {
                delete children[i];
            }
        }
    }
};

struct BTreeIndex {
    BTreeNode* root;
    char name[64];

    BTreeIndex(const char* idx_name) : root(new BTreeNode(true)) {
        strncpy(name, idx_name, sizeof(name) - 1);
        name[sizeof(name) - 1] = '\0';
    }

    ~BTreeIndex() {
        delete root;
    }
};

static int compare_keys(const uint8_t* k1, size_t len1, const uint8_t* k2, size_t len2) {
    size_t min_len = len1 < len2 ? len1 : len2;
    int cmp = std::memcmp(k1, k2, min_len);
    if (cmp != 0) return cmp;
    if (len1 < len2) return -1;
    if (len1 > len2) return 1;
    return 0;
}

static void split_child(BTreeNode* parent, size_t index) {
    BTreeNode* full_child = parent->children[index];
    BTreeNode* new_child = new BTreeNode(full_child->is_leaf);

    size_t mid = BTREE_ORDER / 2;
    new_child->num_keys = BTREE_ORDER - mid - 1;

    for (size_t i = 0; i < new_child->num_keys; i++) {
        new_child->keys[i] = full_child->keys[mid + 1 + i];
    }

    if (!full_child->is_leaf) {
        for (size_t i = 0; i <= new_child->num_keys; i++) {
            new_child->children[i] = full_child->children[mid + 1 + i];
        }
    } else {
        for (size_t i = 0; i < new_child->num_keys; i++) {
            new_child->values[i] = full_child->values[mid + 1 + i];
        }
    }

    full_child->num_keys = mid;

    for (size_t i = parent->num_keys; i > index; i--) {
        parent->children[i + 1] = parent->children[i];
    }
    parent->children[index + 1] = new_child;

    for (size_t i = parent->num_keys; i > index; i--) {
        parent->keys[i] = parent->keys[i - 1];
    }
    parent->keys[index] = full_child->keys[mid];

    parent->num_keys++;
}

static void insert_non_full(BTreeNode* node, const void* key, size_t key_len, uint64_t value) {
    int i = node->num_keys - 1;

    if (node->is_leaf) {
        while (i >= 0 && compare_keys((const uint8_t*)key, key_len, node->keys[i].data(), node->keys[i].size()) < 0) {
            node->keys[i + 1] = node->keys[i];
            node->values[i + 1] = node->values[i];
            i--;
        }

        node->keys[i + 1] = std::vector<uint8_t>((const uint8_t*)key, (const uint8_t*)key + key_len);
        node->values[i + 1] = value;
        node->num_keys++;
    } else {
        while (i >= 0 && compare_keys((const uint8_t*)key, key_len, node->keys[i].data(), node->keys[i].size()) < 0) {
            i--;
        }
        i++;

        if (node->children[i]->num_keys == BTREE_ORDER) {
            split_child(node, i);
            if (compare_keys((const uint8_t*)key, key_len, node->keys[i].data(), node->keys[i].size()) > 0) {
                i++;
            }
        }

        insert_non_full(node->children[i], key, key_len, value);
    }
}

extern "C" {

BTreeIndex* storage_create_btree(StorageHandle* handle, const char* name) {
    return new BTreeIndex(name);
}

void storage_destroy_btree(BTreeIndex* index) {
    delete index;
}

StorageResult storage_btree_insert(BTreeIndex* index, const void* key, size_t key_len, uint64_t value) {
    BTreeNode* root = index->root;

    if (root->num_keys == BTREE_ORDER) {
        BTreeNode* new_root = new BTreeNode(false);
        new_root->children[0] = root;
        split_child(new_root, 0);
        index->root = new_root;
        insert_non_full(new_root, key, key_len, value);
    } else {
        insert_non_full(root, key, key_len, value);
    }

    return STORAGE_OK;
}

bool storage_btree_search(BTreeIndex* index, const void* key, size_t key_len, uint64_t* value) {
    BTreeNode* node = index->root;

    while (node) {
        size_t i = 0;
        while (i < node->num_keys && compare_keys((const uint8_t*)key, key_len, node->keys[i].data(), node->keys[i].size()) > 0) {
            i++;
        }

        if (i < node->num_keys && compare_keys((const uint8_t*)key, key_len, node->keys[i].data(), node->keys[i].size()) == 0) {
            if (node->is_leaf) {
                *value = node->values[i];
                return true;
            }
            node = node->children[i + 1];
        } else {
            if (node->is_leaf) {
                return false;
            }
            node = node->children[i];
        }
    }

    return false;
}

StorageResult storage_btree_delete(BTreeIndex* index, const void* key, size_t key_len) {
    return STORAGE_OK;
}

}
