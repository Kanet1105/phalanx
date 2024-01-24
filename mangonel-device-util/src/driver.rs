use crate::{
    device::EthernetDevice,
    error::{Error, ErrorKind},
};
use std::{
    ffi::OsString,
    fs::{self, DirEntry},
    os::unix::ffi::OsStrExt,
    path::{Path, PathBuf},
};

pub const DRIVER_PATH: &str = "/sys/bus/pci/drivers";

#[derive(Debug)]
pub struct Driver {
    name: OsString,
    path: PathBuf,
}

impl AsRef<[u8]> for Driver {
    fn as_ref(&self) -> &[u8] {
        self.name.as_bytes()
    }
}

impl AsRef<str> for Driver {
    fn as_ref(&self) -> &str {
        self.name
            .to_str()
            .expect("Invalid Unicode in the driver name")
    }
}

impl AsRef<Path> for Driver {
    fn as_ref(&self) -> &Path {
        self.path.as_path()
    }
}

impl From<DirEntry> for Driver {
    fn from(value: DirEntry) -> Self {
        Self {
            name: value.file_name(),
            path: value.path(),
        }
    }
}

impl From<PathBuf> for Driver {
    fn from(value: PathBuf) -> Self {
        Self {
            name: value.clone().into_os_string(),
            path: value,
        }
    }
}

impl Driver {
    pub fn list_drivers() -> Result<Vec<Self>, Error> {
        let driver_list: Vec<Self> = fs::read_dir(DRIVER_PATH)
            .map_err(|error| Error::new(ErrorKind::InvalidDriverPath, error))?
            .filter_map(|entry| entry.ok())
            .map(Self::from)
            .collect();
        Ok(driver_list)
    }

    pub fn find_by_name(name: impl AsRef<str>) -> Result<Self, Error> {
        Self::list_drivers()?
            .into_iter()
            .find_map(
                |driver| match AsRef::<str>::as_ref(&driver) == name.as_ref() {
                    true => Some(driver),
                    false => None,
                },
            )
            .ok_or(Error::from(ErrorKind::DriverNotFound))
    }

    pub fn find_by_bound_device(device: &EthernetDevice) -> Result<Self, Error> {
        Self::list_drivers()?
            .into_iter()
            .find_map(|driver| {
                let driver_path: &Path = driver.as_ref();
                let device_name: &str = device.as_ref();
                match driver_path.join(device_name).exists() {
                    true => Some(driver),
                    false => None,
                }
            })
            .ok_or(Error::from(ErrorKind::DriverNotFound))
    }

    pub fn bind(&self, device: &EthernetDevice) -> Result<(), Error> {
        fs::write(self.path.join("bind"), device)
            .map_err(|error| Error::new(ErrorKind::DriverBind, error))?;
        Ok(())
    }

    pub fn unbind(&self, device: &EthernetDevice) -> Result<(), Error> {
        fs::write(self.path.join("unbind"), device)
            .map_err(|error| Error::new(ErrorKind::DriverUnbind, error))?;
        Ok(())
    }
}
