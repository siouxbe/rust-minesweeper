use std::env;

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let mut config: cbindgen::Config = Default::default();
    config.language = cbindgen::Language::Cxx;
    config.header = Some("///This code is compiled with the rust compiler".into());
    config.include_guard = Some("_RUST_LIB_H_".into());
    cbindgen::generate_with_config(&crate_dir, config)
        .unwrap()
        .write_to_file("target/rust_lib.hpp");

    println!("cargo:rerun-if-changed=src/lib.cpp");
    cc::Build::new().file("src/lib.cpp").compile("person");
}
