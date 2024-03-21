use std::{io, panic::Location};

pub trait WrapError {
    type Output;

    fn wrap<C>(self, context: C) -> Self::Output
    where
        C: std::fmt::Debug + 'static;
}

impl<T, E> WrapError for Result<T, E>
where
    E: std::error::Error + 'static,
{
    type Output = Result<T, Error>;

    #[track_caller]
    fn wrap<C>(self, context: C) -> Self::Output
    where
        C: std::fmt::Debug + 'static,
    {
        match self {
            Ok(value) => Ok(value),
            Err(error) => Err(Error::boxed(error, context)),
        }
    }
}

impl WrapError for i32 {
    type Output = Result<(), Error>;

    fn wrap<C>(self, context: C) -> Self::Output
    where
        C: std::fmt::Debug + 'static,
    {
        match self.is_negative() {
            true => Err(Error::ffi(-self, context)),
            false => Ok(()),
        }
    }
}

pub struct Error {
    location: Location<'static>,
    context: Box<dyn std::fmt::Debug>,
    source: ErrorKind,
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "[{}:{}]: {:?}",
            self.location.file(),
            self.location.line(),
            self.context
        )?;
        write!(f, "{}", self.source)?;
        Ok(())
    }
}

impl std::error::Error for Error {}

impl Error {
    #[track_caller]
    pub fn boxed<E, C>(error: E, context: C) -> Self
    where
        E: std::error::Error + 'static,
        C: std::fmt::Debug + 'static,
    {
        Self {
            location: *Location::caller(),
            context: Box::new(context),
            source: ErrorKind::Boxed(Box::new(error)),
        }
    }

    #[track_caller]
    pub fn ffi<C>(error_code: i32, context: C) -> Self
    where
        C: std::fmt::Debug + 'static,
    {
        Self {
            location: *Location::caller(),
            context: Box::new(context),
            source: ErrorKind::FFI(error_code),
        }
    }
}

pub enum ErrorKind {
    Boxed(Box<dyn std::error::Error>),
    FFI(i32),
    StaticStr(&'static str),
}

impl std::fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Self::Boxed(error) => write!(f, "{}", error),
            Self::FFI(error_code) => write!(f, "{}", io::Error::from_raw_os_error(*error_code)),
            Self::StaticStr(error_str) => write!(f, "{}", error_str),
        }
    }
}
