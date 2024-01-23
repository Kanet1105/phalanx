#[derive(Debug, Eq, PartialEq)]
pub enum ErrorKind {
    InvalidDevicePath,
    DeviceNotFound,
    InvalidDriverPath,
    DriverNotFound,
    DriverBind,
    DriverUnbind,
    DriverOverride,
}

pub struct Error {
    kind: ErrorKind,
    source: Option<Box<dyn std::error::Error>>,
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Self::fmt_kind(&self, f)?;
        Self::fmt_source(&self, f)?;
        Ok(())
    }
}

impl std::error::Error for Error {}

impl From<ErrorKind> for Error {
    fn from(value: ErrorKind) -> Self {
        Self {
            kind: value,
            source: None,
        }
    }
}

impl Error {
    pub fn new<T>(error_kind: ErrorKind, error: T) -> Self
    where
        T: std::error::Error + 'static,
    {
        Self {
            kind: error_kind,
            source: Some(Box::new(error)),
        }
    }

    fn fmt_kind(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error: {:?}\n", self.kind)
    }

    fn fmt_source(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.source {
            Some(source) => write!(f, "Source: {}", source),
            None => Ok(()),
        }
    }
}
