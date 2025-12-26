use std::env;

fn main() {
    cc::Build::new()
        .include("storage/include")
        .file("storage/entry.c")
        .file("storage/buffer/buffer_pool.c")
        .file("storage/pages/page_manager.c")
        .file("storage/wal/wal.c")
        .file("storage/memory/arena.c")
        .warnings(false)
        .flag_if_supported("-g")
        .compile("minsql_storage");

    println!("cargo:rerun-if-changed=storage/");
    println!("cargo:rerun-if-changed=storage/entry.c");
    println!("cargo:rerun-if-changed=storage/buffer/buffer_pool.c");
    println!("cargo:rerun-if-changed=storage/pages/page_manager.c");
    println!("cargo:rerun-if-changed=storage/wal/wal.c");
    println!("cargo:rerun-if-changed=storage/memory/arena.c");
    println!("cargo:rerun-if-changed=storage/include/minsql_storage.h");

    let out_dir = env::var("OUT_DIR").unwrap();
    println!("cargo:rustc-link-search=native={}", out_dir);
    println!("cargo:rustc-link-lib=static=minsql_storage");
}
