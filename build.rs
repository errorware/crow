fn main() {
    #[cfg(windows)]
    {
        // Match Linux/macOS's 8 MB main-thread stack (MSVC defaults to 1 MB,
        // which GPUI's element/render trees overflow in debug builds).
        println!("cargo:rustc-link-arg=/STACK:8388608");
        println!("cargo:rerun-if-changed=packaging/windows/crow.rc");
        println!("cargo:rerun-if-changed=packaging/windows/crow.ico");
        embed_resource::compile("packaging/windows/crow.rc", embed_resource::NONE)
            .manifest_optional()
            .expect("compile Windows icon and version resource");
    }
}
