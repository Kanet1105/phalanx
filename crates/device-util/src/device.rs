use crate::driver::Driver;
use crate::error::{Error, ErrorKind};
use std::fs;
use std::path::PathBuf;

pub const DEVICE_PATH: &str = "/sys/bus/pci/devices";

#[derive(Debug)]
pub struct Device {
    name: String,
    path: PathBuf,
}

impl AsRef<[u8]> for Device {
    fn as_ref(&self) -> &[u8] {
        self.name.as_bytes()
    }
}

impl Device {
    pub fn find_by_bdf(bdf: &str) -> Result<Self, Error> {
        let device_path = PathBuf::from(DEVICE_PATH).join(bdf);
        match device_path.exists() {
            true => {
                let device_name = device_path
                    .file_name()
                    .ok_or(Error::from(ErrorKind::InvalidDevicePath))?
                    .to_str()
                    .ok_or(Error::from(ErrorKind::InvalidUtf8))?
                    .to_string();

                Ok(Self {
                    name: device_name,
                    path: device_path,
                })
            }
            false => Err(Error::from(ErrorKind::DeviceDoesNotExist)),
        }
    }

    pub fn bind_driver(&self, driver: &Driver) -> Result<(), Error> {
        self.unbind_current_driver()?;
        self.driver_override(driver)?;
        driver.bind(self)?;
        Ok(())
    }

    fn unbind_current_driver(&self) -> Result<(), Error> {
        let current_driver_path = self.path.join("driver");
        if current_driver_path.exists() {
            let unbind_path = current_driver_path.join("unbind");
            fs::write(unbind_path, self)
                .map_err(|error| Error::new(ErrorKind::DriverUnbind, error))?;
        }
        Ok(())
    }

    fn driver_override(&self, driver: &Driver) -> Result<(), Error> {
        let override_path = self.path.join("driver_override");
        fs::write(override_path, driver)
            .map_err(|error| Error::new(ErrorKind::DriverOverride, error))?;
        Ok(())
    }
}
