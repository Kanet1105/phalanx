#[allow(non_upper_case_globals)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
#[allow(clippy::all)]
mod dpdk {
    include!(concat!(env!("OUT_DIR"), "/dpdk.rs"));
}
pub use dpdk::*;
