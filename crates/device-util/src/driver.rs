use crate::device::EthernetDevice;
use crate::error::{Error, ErrorKind};
use std::fs;
use std::path::PathBuf;

pub const DRIVER_PATH: &str = "/sys/bus/pci/drivers";

#[derive(Debug)]
pub struct Driver {
    name: String,
    path: PathBuf,
}

impl std::cmp::PartialEq<&str> for Driver {
    fn eq(&self, other: &&str) -> bool {
        self.name.as_str() == *other
    }
}

impl AsRef<[u8]> for Driver {
    fn as_ref(&self) -> &[u8] {
        self.name.as_bytes()
    }
}

impl From<&str> for Driver {
    fn from(value: &str) -> Self {
        Self {
            name: value.to_string(),
            path: PathBuf::from(DRIVER_PATH).join(value),
        }
    }
}

impl Driver {
    pub fn list_drivers() -> Result<Vec<Self>, Error> {
        let driver_list: Vec<Self> = fs::read_dir(DRIVER_PATH)
            .map_err(|error| Error::new(ErrorKind::InvalidDriverPath, error))?
            .filter_map(|entry| entry.ok())
            .filter_map(|entry| entry.file_name().to_str())
            .map(|driver_name| Self::from(driver_name))
            .collect();
        Ok(driver_list)
    }

    pub fn find_by_name(name: &str) -> Result<Self, Error> {
        Self::list_drivers()?
            .into_iter()
            .find_map(|driver| match driver == name {
                true => Some(driver),
                false => None,
            })
            .ok_or(Error::from(ErrorKind::DriverNotFound))
    }

    pub fn find_by_bound_device(device: &EthernetDevice) -> Result<Self, Error> {
        Self::list_drivers()?
            .into_iter()
            .find_map(|driver| match driver.path.join(device).exists() {
                true => Some(driver),
                false => None,
            })
            .ok_or(Error::from(ErrorKind::DriverNotFound))
    }

    pub fn bind(&self, device: &EthernetDevice) -> Result<(), Error> {
        fs::write(self.path.join("bind"), device)
            .map_err(|error| Error::new(ErrorKind::DriverBind, error))?;
        Ok(())
    }
}
