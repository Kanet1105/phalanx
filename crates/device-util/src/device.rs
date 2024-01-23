use crate::driver::Driver;
use crate::error::{Error, ErrorKind};
use std::fs;
use std::path::{Path, PathBuf};

pub const DEVICE_PATH: &str = "/sys/bus/pci/devices";

#[derive(Debug)]
pub struct EthernetDevice {
    bdf: String,
    path: PathBuf,
}

impl AsRef<[u8]> for EthernetDevice {
    fn as_ref(&self) -> &[u8] {
        self.bdf.as_bytes()
    }
}

impl AsRef<Path> for EthernetDevice {
    fn as_ref(&self) -> &Path {
        self.path.as_path()
    }
}

impl From<&str> for EthernetDevice {
    fn from(value: &str) -> Self {
        Self {
            bdf: value.to_string(),
            path: PathBuf::from(DEVICE_PATH).join(value),
        }
    }
}

impl EthernetDevice {
    pub fn list_devices() -> Result<Vec<Self>, Error> {
        let device_list: Vec<Self> = fs::read_dir(DEVICE_PATH)
            .map_err(|error| Error::new(ErrorKind::InvalidDevicePath, error))?
            .filter_map(|entry| entry.ok())
            .filter_map(|entry| match entry.path().join("net").exists() {
                true => Some(entry.file_name()),
                false => None,
            })
            .map(|bdf| Self::from(bdf.to_str()))
            .collect();
        Ok(device_list)
    }

    pub fn find_by_bdf(bdf: impl AsRef<str>) -> Result<Self, Error> {
        Self::list_devices()?
            .into_iter()
            .find_map(|device| match device.bdf.as_str() == bdf.as_ref() {
                true => Some(device),
                false => None,
            })
            .ok_or(Error::from(ErrorKind::DeviceNotFound))
    }

    // pub fn find_by_name(name: impl AsRef<str>) -> Result<Self, Error> {
    //     Self::list_devices()?
    //         .into_iter()
    //         .find_map(|device| match device.name() == name.as_ref() {
    //             true => Some(device),
    //             false => None,
    //         })
    //         .ok_or(Error::from(ErrorKind::DeviceNotFound))
    // }

    fn get_device_name(device_path: impl AsRef<Path>) -> Result<String, Error> {
        let device_name_path = device_path.as_ref().join("net");
        let device_name = fs::read_dir(device_name_path)
            .map_err(|error| Error::new(ErrorKind::InvalidDeviceNamePath, error))?
            .filter_map(|entry| entry.ok())
            .find_map(|entry| match entry.file_name().to_str() {
                Some(device_name) => Some(device_name.to_string()),
                None => None,
            })
            .ok_or(Error::from(ErrorKind::DeviceNameNotFound))?;
        Ok(device_name)
    }

    // pub fn bind_driver(&self, driver: &Driver) -> Result<(), Error> {
    //     self.unbind_current_driver()?;
    //     self.driver_override(driver)?;
    //     driver.bind(self)?;
    //     Ok(())
    // }

    // fn unbind_current_driver(&self) -> Result<(), Error> {
    //     let current_driver_path = self.path.join("driver");
    //     if current_driver_path.exists() {
    //         let unbind_path = current_driver_path.join("unbind");
    //         fs::write(unbind_path, self)
    //             .map_err(|error| Error::new(ErrorKind::DriverUnbind, error))?;
    //     }
    //     Ok(())
    // }

    // fn driver_override(&self, driver: &Driver) -> Result<(), Error> {
    //     let override_path = self.path.join("driver_override");
    //     fs::write(override_path, driver)
    //         .map_err(|error| Error::new(ErrorKind::DriverOverride, error))?;
    //     Ok(())
    // }
}
