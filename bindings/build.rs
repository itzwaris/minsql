use std::env;
use std::path::PathBuf;

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let storage_dir = manifest_dir.join("storage");

    cc::Build::new()
        .file(storage_dir.join("entry.c"))
        .file(storage_dir.join("wal/wal.c"))
        .file(storage_dir.join("pages/page_manager.c"))
        .file(storage_dir.join("buffer/buffer_pool.c"))
        .file(storage_dir.join("memory/arena.c"))
        .include(storage_dir.join("include"))
        .warnings(false)
        .compile("minsql_storage_c");

    cc::Build::new()
        .cpp(true)
        .file(storage_dir.join("indexes/btree.cpp"))
        .file(storage_dir.join("indexes/hash.cpp"))
        .file(storage_dir.join("indexes/bloom.cpp"))
        .include(storage_dir.join("include"))
        .cpp_set_stdlib("stdc++")
        .std("c++20")
        .warnings(false)
        .compile("minsql_storage_cpp");

    println!("cargo:rerun-if-changed=storage/");
    println!("cargo:rustc-link-lib=pthread");
}
