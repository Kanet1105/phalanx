use device_util::device::*;
use device_util::driver::*;
use std::fs;

fn get_device_list() -> Vec<Device> {
    fs::read_dir(DEVICE_PATH)
        .unwrap()
        .filter_map(|entry| entry.ok())
        .map(|entry| {
            let device_name = entry.file_name().to_str().unwrap().to_string();
            Device::find_by_bdf(&device_name).unwrap()
        })
        .collect()
}

fn get_driver_list() -> Vec<Driver> {
    fs::read_dir(DRIVER_PATH)
        .unwrap()
        .filter_map(|entry| entry.ok())
        .map(|entry| {
            let driver_name = entry.file_name().to_str().unwrap().to_string();
            Driver::find_by_name(&driver_name).unwrap()
        })
        .collect()
}

#[test]
fn works() {
    get_device_list();
    get_driver_list();
}
