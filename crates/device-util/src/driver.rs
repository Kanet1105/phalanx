use crate::device::Device;
use crate::error::{Error, ErrorKind};
use std::fs;
use std::path::PathBuf;

pub const DRIVER_PATH: &str = "/sys/bus/pci/drivers";

#[derive(Debug)]
pub struct Driver {
    name: String,
    path: PathBuf,
}

impl AsRef<[u8]> for Driver {
    fn as_ref(&self) -> &[u8] {
        self.name.as_bytes()
    }
}

impl Driver {
    pub fn find_by_name(name: &str) -> Result<Self, Error> {
        let driver_path = PathBuf::from(DRIVER_PATH).join(name);
        match driver_path.exists() {
            true => {
                let driver_name = driver_path
                    .file_name()
                    .ok_or(Error::from(ErrorKind::InvalidDriverPath))?
                    .to_str()
                    .ok_or(Error::from(ErrorKind::InvalidUtf8))?
                    .to_string();

                Ok(Self {
                    name: driver_name,
                    path: driver_path,
                })
            }
            false => Err(Error::from(ErrorKind::DriverDoesNotExist)),
        }
    }

    pub fn bind(&self, device: &Device) -> Result<(), Error> {
        fs::write(self.path.join("bind"), device)
            .map_err(|error| Error::new(ErrorKind::DriverBind, error))?;
        Ok(())
    }
}
