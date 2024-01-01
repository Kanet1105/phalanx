use std::{env, path::PathBuf};

const LIBRARY_NAME: &str = "libdpdk";
const LIBRARY_VERSION: &str = env!("CARGO_PKG_VERSION");
const WRAPPER: &str = "wrapper.c";

fn main() {
    check_os();

    link_library(LIBRARY_NAME, LIBRARY_VERSION);

    // Build the crate again when the content of WRAPPER changes.
    println!("cargo:rerun-if-changed={}", WRAPPER);

    let builder = bindgen::builder()
        .header(WRAPPER)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .derive_debug(true)
        .opaque_type("rte_.*_hdr")
        .opaque_type("rte_arp_ipv4")
        .generate()
        .expect(&format!(
            "Unable to generate bindings for {} v{}",
            LIBRARY_NAME, LIBRARY_VERSION
        ));

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    builder
        .write_to_file(out_path.join("bindings_dpdk.rs"))
        .expect(&format!(
            "Unable to write bindings to the file at {:?}",
            out_path.as_os_str(),
        ));
}

/// Check the target OS that we will build against. Currently supports Linux only.
fn check_os() {
    if !cfg!(target_os = "linux") {
        panic!("Currently supports linux only");
    }
}

/// Link the library by the name and its version the version has to match exactly to the version
/// of the installed libarary.
fn link_library(name: &str, version: &str) {
    pkg_config::Config::new()
        .exactly_version(version)
        .probe(name)
        .expect("Unable to link the library");
}
