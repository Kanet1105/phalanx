use std::env;
use std::path::PathBuf;

const LIB_NAME: &str = "libxdp";
const LIB_VERSION: &str = "1.4";
const WRAPPER: &str = "wrapper.h";

/// # Panics
///
/// Causes the program to panic if run on other than linux. Which is
/// a natural course of event because AF_XDP, unlike its strongest
/// alternative, DPDK, supports only linux.
fn check_os() {
    if !cfg!(target_os = "linux") {
        panic!("Currently supports linux only.");
    }
}

/// Link a library by its name and version.
/// 
/// # Panics
/// 
/// Panics either when the program fails to find the library in the
/// default system library path or when the installed library version
/// does not satisfy the minimum version given by "version" parameter.
fn link_lib(name: &str, version: &str) {
    pkg_config::Config::new()
        .atleast_version(version)
        .probe(name)
        .unwrap_or_else(|error| panic!("Error linking library: {}", error));
}

fn main() {
    check_os();
    link_lib(LIB_NAME, LIB_VERSION);

    let bindings = bindgen::Builder::default()
        .header(WRAPPER)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
