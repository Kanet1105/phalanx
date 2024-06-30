use std::panic::Location;

pub trait WrapError {
    type Output;

    fn wrap(self, kind: ErrorKind) -> Self::Output;
}

impl<T, E> WrapError for Result<T, E>
where
    E: Into<ErrorSource>,
{
    type Output = Result<T, Error>;

    fn wrap(self, kind: ErrorKind) -> Self::Output {
        match self {
            Ok(value) => Ok(value),
            Err(error) => Err((kind, error).into()),
        }
    }
}

impl WrapError for i32 {
    type Output = Result<(), Error>;

    fn wrap(self, kind: ErrorKind) -> Self::Output {
        match self.is_negative() {
            true => Err((kind, -self).into()),
            false => Ok(()),
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum ErrorKind {
    Mmap,
    Munmap,
}

pub struct Error {
    kind: ErrorKind,
    source: ErrorSource,
    location: Location<'static>,
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}: {} at {}", self.kind, self.source, self.location)
    }
}

impl std::error::Error for Error {}

impl<E> From<(ErrorKind, E)> for Error
where
    E: Into<ErrorSource>,
{
    #[track_caller]
    fn from(value: (ErrorKind, E)) -> Self {
        Self {
            kind: value.0,
            source: value.1.into(),
            location: *Location::caller(),
        }
    }
}

enum ErrorSource {
    IO(std::io::Error),
    FFI(i32),
}

impl std::fmt::Display for ErrorSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IO(error) => write!(f, "{}", error),
            Self::FFI(error_code) => {
                write!(f, "{}", std::io::Error::from_raw_os_error(*error_code))
            }
        }
    }
}

impl From<std::io::Error> for ErrorSource {
    fn from(value: std::io::Error) -> Self {
        Self::IO(value)
    }
}

impl From<i32> for ErrorSource {
    fn from(value: i32) -> Self {
        Self::FFI(value)
    }
}
