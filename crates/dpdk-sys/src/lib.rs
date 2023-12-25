#[allow(non_upper_case_globals)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
mod dpdk {
    include!(concat!(env!("OUT_DIR"), "/bindings-dpdk.rs"));
}
pub use dpdk::*;
