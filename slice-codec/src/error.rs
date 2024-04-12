// Copyright (c) ZeroC, Inc.

use core::fmt::{Display, Formatter};
use core::ops::Range;
use core::write;

#[cfg(feature = "std")]
use alloc::boxed::Box;

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
    // Until `Error` is moved from `std` into `core`, we need a feature; https://github.com/icerpc/slice-rust/issues/1.
    #[cfg(feature = "std")]
    source: Option<Box<dyn std::error::Error + 'static>>,
}

impl Error {
    /// Creates a new error of the specified kind, with no underlying source.
    pub fn new(kind: ErrorKind) -> Self {
        Self {
            kind,
            #[cfg(feature = "std")]
            source: None,
        }
    }

    /// Creates a new error of the specified kind, which was logically caused by the provided source.
    // Until `Error` is moved from `std` into `core`, we need a feature; https://github.com/icerpc/slice-rust/issues/1.
    #[cfg(feature = "std")]
    pub fn new_with_source(kind: ErrorKind, source: impl std::error::Error + 'static) -> Self {
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

        // Until `Error` is moved from `std` into `core`, we need a feature; https://github.com/icerpc/slice-rust/issues/1.
        #[cfg(feature = "std")]
        // If this error was caused by another error, also write that source error.
        if let Some(source) = &self.source {
            f.write_str("\nError was caused by:\n")?;
            source.fmt(f)?;
        }

        Ok(())
    }
}

// Until `Error` is moved from `std` into `core`, we need a feature; https://github.com/icerpc/slice-rust/issues/1.
#[cfg(feature = "std")]
impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source.as_deref()
    }
}

impl From<ErrorKind> for Error {
    /// Creates a new [`Error`] from the provided [`ErrorKind`], with no underlying source.
    fn from(value: ErrorKind) -> Self {
        Self::new(value)
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
    /// Receiving this error represents a serious problem in the implementation, or intentional tampering by callers.
    /// See [`write_bytes_exact_into_reserved`](crate::buffer::output::OutputTarget::write_bytes_into_reserved_exact).
    InvalidReservation {
        /// The length of the buffer.
        buffer_len: usize,
        /// The range (pair of indices) in the buffer which were reserved invalidly.
        reserved_range: Range<usize>,
    },
}

impl Display for ErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::UnexpectedEob { requested, remaining } => {
                write!(f, "unexpected end of buffer: attempted to read '{requested}' bytes from buffer with only '{remaining}' bytes remaining")
            }
            Self::InvalidReservation { buffer_len, reserved_range } => {
                let Range { start, end } = reserved_range;
                write!(f, "invalid reservation: range '[{start}..{end})' does not fit within buffer of length '{buffer_len}'")
            }
        }
    }
}
