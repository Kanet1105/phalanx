use crate::driver::Driver;
use crate::error::{Error, ErrorKind};
use std::ffi::OsString;
use std::fs::{self, DirEntry};
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

pub const DEVICE_PATH: &str = "/sys/bus/pci/devices";

#[derive(Debug)]
pub struct EthernetDevice {
    bdf: OsString,
    path: PathBuf,
    default_driver: Option<Driver>,
}

impl AsRef<[u8]> for EthernetDevice {
    fn as_ref(&self) -> &[u8] {
        self.bdf.as_bytes()
    }
}

impl AsRef<str> for EthernetDevice {
    fn as_ref(&self) -> &str {
        self.bdf
            .to_str()
            .expect("Invalid Unicode in the device name")
    }
}

impl From<DirEntry> for EthernetDevice {
    fn from(value: DirEntry) -> Self {
        let mut device = Self {
            bdf: value.file_name(),
            path: value.path(),
            default_driver: None,
        };
        device.default_driver = Driver::find_by_bound_device(&device).ok();
        device
    }
}

impl EthernetDevice {
    pub fn list_devices() -> Result<Vec<Self>, Error> {
        let device_list: Vec<Self> = fs::read_dir(DEVICE_PATH)
            .map_err(|error| Error::new(ErrorKind::InvalidDevicePath, error))?
            .filter_map(|entry| entry.ok())
            .filter_map(|entry| match entry.path().join("net").exists() {
                true => Some(Self::from(entry)),
                false => None,
            })
            .collect();
        Ok(device_list)
    }

    pub fn find_by_bdf_notation(bdf: impl AsRef<str>) -> Result<Self, Error> {
        Self::list_devices()?
            .into_iter()
            .find_map(
                |device| match AsRef::<str>::as_ref(&device) == bdf.as_ref() {
                    true => Some(device),
                    false => None,
                },
            )
            .ok_or(Error::from(ErrorKind::DeviceNotFound))
    }

    pub fn bind_driver(&self, driver: &Driver) -> Result<(), Error> {
        if let Some(default_driver) = &self.default_driver {
            default_driver.unbind(self)?;
        }
        self.override_driver(driver)?;
        driver.bind(self)?;
        Ok(())
    }

    pub fn override_driver(&self, driver: &Driver) -> Result<(), Error> {
        let override_path = self.path.join("driver_override");
        fs::write(override_path, driver)
            .map_err(|error| Error::new(ErrorKind::DriverOverride, error))?;
        Ok(())
    }
}
