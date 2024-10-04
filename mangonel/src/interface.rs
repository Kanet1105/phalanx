use std::sync::Arc;

use pnet::datalink::{self, NetworkInterface};

pub struct Port {
    inner: Arc<PortInner>,
}

struct PortInner {
    wan: NetworkInterface,
    lan: NetworkInterface,
}

impl Clone for Port {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl Port {
    pub fn new(
        wan_interface_name: impl AsRef<str>,
        lan_interface_name: impl AsRef<str>,
    ) -> Result<Self, NetworkInterfaceError> {
        let wan = datalink::interfaces()
            .into_iter()
            .find(|interface| interface.name == wan_interface_name.as_ref())
            .ok_or(NetworkInterfaceError::DeviceDoesNotExist(
                wan_interface_name.as_ref().to_owned(),
            ))?;

        let lan = datalink::interfaces()
            .into_iter()
            .find(|interface| interface.name == lan_interface_name.as_ref())
            .ok_or(NetworkInterfaceError::DeviceDoesNotExist(
                wan_interface_name.as_ref().to_owned(),
            ))?;

        let inner = PortInner { wan, lan };

        Ok(Self {
            inner: Arc::new(inner),
        })
    }

    pub fn wan(&self) -> &NetworkInterface {
        &self.inner.wan
    }

    pub fn lan(&self) -> &NetworkInterface {
        &self.inner.lan
    }
}

#[derive(Debug)]
pub enum NetworkInterfaceError {
    DeviceDoesNotExist(String),
}
