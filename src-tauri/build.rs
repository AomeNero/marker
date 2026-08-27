fn main() {
    // GNU ld on Windows exports every symbol from linked rlibs into the cdylib,
    // exceeding the PE/COFF 65535 export ordinal limit on large Tauri apps.
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows")
        && std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("gnu")
    {
        println!("cargo::rustc-link-arg=-Wl,--exclude-libs=ALL,--exclude-all-symbols");
    }
    tauri_build::build()
}
