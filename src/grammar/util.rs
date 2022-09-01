// Copyright (c) ZeroC, Inc. All rights reserved.

use std::fmt;
use std::str::FromStr;

#[derive(Clone, Debug, Default)]
pub struct Scope {
    pub raw_module_scope: String,
    pub module_scope: Vec<String>,
    pub raw_parser_scope: String,
    pub parser_scope: Vec<String>,
}

impl Scope {
    pub fn new(name: &str, is_module: bool) -> Scope {
        let parser_scope = name
            .split("::")
            .filter(|s| !s.is_empty())
            .map(|s| s.to_owned())
            .collect::<Vec<_>>();

        let module_scope = if is_module { parser_scope.clone() } else { Vec::new() };

        Scope {
            raw_module_scope: module_scope.join("::"),
            module_scope,
            raw_parser_scope: parser_scope.join("::"),
            parser_scope,
        }
    }

    pub fn push_scope(&mut self, name: &str, is_module: bool) {
        if is_module {
            self.module_scope.push(name.to_owned());
            self.raw_module_scope = self.module_scope.join("::");
        }
        self.parser_scope.push(name.to_owned());
        self.raw_parser_scope = self.parser_scope.join("::");
    }

    pub fn pop_scope(&mut self) {
        // If the last parser scope is also a module scope, pop off a module scope as well.
        if self.parser_scope.last() == self.module_scope.last() {
            self.module_scope.pop();
            self.raw_module_scope = self.module_scope.join("::");
        }
        self.parser_scope.pop();
        self.raw_parser_scope = self.parser_scope.join("::");
    }
}

/// This enum specifies all the encodings supported by IceRPC.
///
/// These encodings identity the format used to convert Slice types to and from byte streams.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Encoding {
    /// Version 1 of the Slice encoding, supported by IceRPC, and compatible with Ice 3.5 or
    /// greater.
    ///
    /// It is primarily for interoperability between Ice and IceRPC.
    Slice1,

    /// Version 2 of the Slice encoding, supported by IceRPC.
    ///
    /// The default encoding when using IceRPC.
    Slice2,
}

impl Default for Encoding {
    /// Returns the default encoding for this version of IceRPC: the Slice2 encoding.
    fn default() -> Self {
        Encoding::Slice2
    }
}

impl fmt::Display for Encoding {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Slice1 => write!(f, "Slice1"),
            Self::Slice2 => write!(f, "Slice2"),
        }
    }
}

/// These format types describe how classes and exceptions with inheritance hierarchies are
/// encoded. With IceRPC, exceptions are always sent in a Sliced format, but they can be received
/// in either format for backwards compatibility. Classes are sent in a Compact format by default.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ClassFormat {
    /// Used when both sender and receiver have the same Slice definitions for classes and
    /// exceptions. Encoding a class or exception with this format is more efficient, but if an
    /// application receives an instance it doesn't know, it's incapable of decoding it.
    Compact,

    /// Used when a sender and receiver may have different Slice definitions. Each layer of
    /// inheritance is sliced separately, allowing applications to ignore unknown slices.
    Sliced,
}

impl fmt::Display for ClassFormat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Compact => write!(f, "Compact"),
            Self::Sliced => write!(f, "Sliced"),
        }
    }
}

impl FromStr for ClassFormat {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Compact" => Ok(Self::Compact),
            "Sliced" => Ok(Self::Sliced),
            _ => Err(()),
        }
    }
}
/// This tag format describes how the data is encoded and how it can be skipped by the decoding
/// code if the tagged parameter is present in the buffer but is not known to the receiver.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TagFormat {
    /// A fixed size numeric encoded on 1 byte such as bool or int8.
    F1,

    /// A fixed size numeric encoded on 2 bytes such as int16.
    F2,

    /// A fixed size numeric encoded on 4 bytes such as int32 or float32.
    F4,

    /// A fixed size numeric encoded on 8 bytes such as int64 or float64.
    F8,

    /// A variable-length size encoded on 1 or 5 bytes.
    Size,

    /// A variable-length size followed by size bytes.
    VSize,

    /// A fixed length size (encoded on 4 bytes) followed by size bytes.
    FSize,

    /// Represents a class, but is no longer encoded or decoded.
    Class,

    /// Pseudo non-encoded format: like VSize but the size is optimized out.
    OVSize,
}

impl fmt::Display for TagFormat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TagFormat::F1 => write!(f, "F1"),
            TagFormat::F2 => write!(f, "F2"),
            TagFormat::F4 => write!(f, "F4"),
            TagFormat::F8 => write!(f, "F8"),
            TagFormat::Size => write!(f, "Size"),
            TagFormat::VSize => write!(f, "VSize"),
            TagFormat::FSize => write!(f, "FSize"),
            TagFormat::Class => write!(f, "Class"),
            TagFormat::OVSize => write!(f, "OVSize"),
        }
    }
}
