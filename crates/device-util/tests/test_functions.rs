use device_util::device::*;
use device_util::driver::*;

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
