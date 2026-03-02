// Copyright (c) ZeroC, Inc.

use core::fmt::{Display, Formatter};
use core::num::TryFromIntError;
use core::ops::Range;
use core::write;

#[cfg(feature = "alloc")]
use alloc::boxed::Box;
#[cfg(feature = "alloc")]
use alloc::collections::TryReserveError;
#[cfg(feature = "alloc")]
use alloc::string::FromUtf8Error;

/// A specialized [`Result`](core::result::Result) type for encoding and decoding functions which may produce errors.
///
/// This typedef is a convenience to avoid repetitively specifying [`Error`] as the error type, and is a direct mapping
/// to a [`core::result::Result`] with an `Err` type of [`Error`].
pub type Result<T> = core::result::Result<T, Error>;

/// The error type for encoding and decoding functions.
#[derive(Debug)]
pub struct Error {
    /// Describes the kind of error that occurred and provides additional information about it.
    kind: ErrorKind,

    /// The underlying cause of this error, if any exist.
    #[cfg(feature = "alloc")]
    source: Option<Box<dyn core::error::Error + 'static>>,
}

impl Error {
    /// Creates a new error of the specified kind, with no underlying source.
    pub fn new(kind: ErrorKind) -> Self {
        Self {
            kind,
            #[cfg(feature = "alloc")]
            source: None,
        }
    }

    /// Creates a new error of the specified kind, which was logically caused by the provided source.
    #[cfg(feature = "alloc")]
    pub fn new_with_source(kind: ErrorKind, source: impl core::error::Error + 'static) -> Self {
        Self {
            kind,
            source: Some(Box::new(source)),
        }
    }

    /// Returns the corresponding [`ErrorKind`] that describes this error.
    pub fn kind(&self) -> &ErrorKind {
        &self.kind
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        // Write this error's underlying `ErrorKind`.
        self.kind.fmt(f)?;

        #[cfg(feature = "alloc")]
        // If this error was caused by another error, also write that source error.
        if let Some(source) = &self.source {
            f.write_str("\nError was caused by:\n")?;
            source.fmt(f)?;
        }

        Ok(())
    }
}

#[cfg(feature = "alloc")]
impl core::error::Error for Error {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        self.source.as_deref()
    }
}

impl<T: Into<ErrorKind>> From<T> for Error {
    /// Creates a new [`Error`] from the provided [`ErrorKind`], with no underlying source.
    fn from(value: T) -> Self {
        Self::new(value.into())
    }
}

/// A list that specifies all the kinds of errors that can be returned by this crate's functions.
/// It is typically held by an [`Error`].
///
/// This list may grow over time, so it is not recommended to exhaustively match against it.
#[derive(Debug)]
#[non_exhaustive]
pub enum ErrorKind {
    /// A function attempted to read past the end of a buffer.
    UnexpectedEob {
        /// The number of bytes that the function tried to read.
        requested: usize,
        /// The number of readable bytes that were left in the buffer.
        remaining: usize,
    },

    /// A buffer reservation did not fit within its buffer.
    /// This error represents a serious problem in the implementation, or intentional tampering by callers.
    /// See [`write_bytes_exact_into_reserved`](crate::buffer::OutputTarget::write_bytes_into_reserved_exact).
    InvalidReservation {
        /// The length of the buffer.
        buffer_len: usize,
        /// The range (pair of indices) in the buffer which were reserved invalidly.
        reserved_range: Range<usize>,
    },

    /// The system failed to allocate memory.
    /// Unlike [`ErrorKind::AllocationLimitReached`],
    ///
    /// Most probably, this happened when a decoder attempted to allocate space for a string or collection.
    #[cfg(feature = "alloc")]
    AllocationError(TryReserveError),

    /// Hello
    AllocationLimitReached {
        requested: usize,

        remaining: usize,
    },

    InvalidData(InvalidDataErrorKind),
}

impl Display for ErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::UnexpectedEob { requested, remaining } => {
                write!(f, "unexpected end of buffer: attempted to read '{requested}' bytes from a buffer with only '{remaining}' bytes remaining")
            }
            Self::InvalidReservation {
                buffer_len,
                reserved_range,
            } => {
                let Range { start, end } = reserved_range;
                write!(
                    f,
                    "invalid reservation: range '[{start}..{end})' does not fit within buffer of length '{buffer_len}'"
                )
            }
            Self::InvalidData(inner) => inner.fmt(f),
            _ => todo!(),
        }
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub enum InvalidDataErrorKind {
    /// TODO
    IllegalValue { desc: &'static str, value: Option<i128> },

    /// TODO
    /// A malformed string (one whose bytes aren't valid UTF8) was encountered.
    #[cfg(feature = "alloc")]
    InvalidString(FromUtf8Error),

    /// TODO
    OutOfRange {
        value: i128,
        min: i128,
        max: i128,
        typename: &'static str,
    },
}

impl Display for InvalidDataErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::IllegalValue { desc, value } => {
                if let Some(value) = value {
                    write!(f, "illegal value: {desc} (value: {value})")
                } else {
                    write!(f, "illegal value: {desc}")
                }
            }
            Self::OutOfRange { value, min, max, typename } => {
                write!(
                    f,
                    "value '{value}' is outside the allowed range for type '{typename}'; values must be within [{min}..{max}]"
                )
            }
            _ => todo!(),
        }
    }
}

impl From<InvalidDataErrorKind> for ErrorKind {
    fn from(value: InvalidDataErrorKind) -> Self {
        ErrorKind::InvalidData(value)
    }
}

impl From<TryFromIntError> for Error {
    fn from(_: TryFromIntError) -> Self {
        Error::from(InvalidDataErrorKind::IllegalValue {
            desc: "failed to convert integer",
            value: None,
        })
    }
}

#[cfg(feature = "alloc")]
impl From<TryReserveError> for Error {
    fn from(value: TryReserveError) -> Self {
        Error::from(ErrorKind::AllocationError(value))
    }
}

#[cfg(feature = "alloc")]
impl From<FromUtf8Error> for Error {
    fn from(value: FromUtf8Error) -> Self {
        Error::from(InvalidDataErrorKind::InvalidString(value))
    }
}
