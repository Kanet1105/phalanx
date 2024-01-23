use device_util::device::*;
use device_util::driver::*;
use std::fs;

#[test]
fn works() {
    get_device_list();
    get_driver_list();
}
