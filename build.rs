fn main() {
    #[cfg(windows)]
    {
        println!("cargo:rerun-if-changed=packaging/windows/crow.rc");
        println!("cargo:rerun-if-changed=packaging/windows/crow.ico");
        embed_resource::compile("packaging/windows/crow.rc", embed_resource::NONE)
            .manifest_optional()
            .expect("compile Windows icon and version resource");
    }
}
