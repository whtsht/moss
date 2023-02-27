use core::{fmt, num::TryFromIntError, panic::Location};
use x86_64::structures::paging::{mapper::MapToError, page::AddressNotAligned, Size4KiB};

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
    location: &'static Location<'static>,
}

impl Error {
    #[track_caller]
    pub(crate) fn new(kind: ErrorKind) -> Self {
        let location = Location::caller();
        Self { kind, location }
    }
}

impl From<ErrorKind> for Error {
    #[track_caller]
    fn from(err: ErrorKind) -> Self {
        Error::new(err)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:?}: {}, {}:{}:{}",
            self.kind,
            self.kind,
            self.location.file(),
            self.location.line(),
            self.location.column()
        )?;
        Ok(())
    }
}

#[derive(Debug)]
pub enum ErrorKind {
    AddressNotAligned(AddressNotAligned),
    MapTo(MapToError<Size4KiB>),
    TryFromInt(TryFromIntError),
    PhysicalMemoryNotMapped,
    Full,
    NoEnoughMemory,
    IndexOutOfRange,
    InvalidSlotID,
    InvalidEndpointNumber,
    TransferRingNotSet,
    AlreadyAllocated,
    TaskQueueIsFull,
    NotImplemented,
    Unknown,
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorKind::AddressNotAligned(err) => write!(f, "{}", err),
            ErrorKind::MapTo(err) => write!(f, "{:?}", err),
            ErrorKind::Full => write!(f, "buffer full"),
            _ => write!(f, "{:?}", self),
        }
    }
}

impl From<AddressNotAligned> for Error {
    #[track_caller]
    fn from(err: AddressNotAligned) -> Self {
        Error::from(ErrorKind::AddressNotAligned(err))
    }
}

impl From<MapToError<Size4KiB>> for Error {
    #[track_caller]
    fn from(err: MapToError<Size4KiB>) -> Self {
        Error::from(ErrorKind::MapTo(err))
    }
}

#[macro_export]
macro_rules! bail {
    ($err:expr) => {
        return Err($crate::error::Error::from($err))
    };
}
