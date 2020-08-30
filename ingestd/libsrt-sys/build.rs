fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=wrapper.h");

    let libsrt = pkg_config::Config::new().probe("srt")?;

    let mut builder = bindgen::Builder::default();
    for path in libsrt.include_paths {
        builder = builder.clang_arg(format!("-I{}", path.to_str().unwrap()));
    }
    let bindings = builder
        .header("wrapper.h")
        .whitelist_recursively(false)
        .whitelist_function("srt_.*")
        .whitelist_var("SRT_.*")
        .whitelist_type("srt_.*")
        .whitelist_type("SRT_.*")
        .whitelist_type("(SRT|SYS|UDP)SOCKET")
        .whitelist_type("CBytePerfMon")
        .size_t_is_usize(true)
        .prepend_enum_name(false)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
    Ok(())
}
