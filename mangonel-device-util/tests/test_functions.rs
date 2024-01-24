use device_util::{device::*, driver::*, error::*};
use std::fs;

#[test]
fn test_list_devices() {
    let device_list = EthernetDevice::list_devices().unwrap();
    println!("{:?}", device_list);
}

#[test]
fn test_list_drivers() {
    let driver_list = Driver::list_drivers().unwrap();
    println!("{:?}", driver_list);
}

#[test]
fn test_error_format() {
    let _ = fs::read_to_string("no_file")
        .map_err(|error| Error::new(ErrorKind::DeviceNotFound, error))
        .map_err(|error| println!("{:?}", error));
}
