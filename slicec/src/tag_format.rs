// Copyright (c) ZeroC, Inc. All rights reserved.

use std::fmt;

/// This tag format describes how the data is
/// encoded and how it can be skipped by the decoding code if the tagged parameter is present in the
/// buffer but is not known to the receiver.
#[derive(Clone, Debug)]
pub enum TagFormat {
    /// A fixed size numeric encoded on 1 byte such as bool or byte.
    F1,

    /// A fixed size numeric encoded on 2 bytes such as short.
    F2,

    /// A fixed size numeric encoded on 4 bytes such as int or float.
    F4,

    /// A fixed size numeric encoded on 8 bytes such as long or double.
    F8,

    /// A variable-length size encoded on 1 or 5 bytes.
    Size,

    /// A variable-length size followed by size bytes.
    VSize,

    /// A fixed length size (encoded on 4 bytes) followed by size bytes.
    FSize,

    /// Represents a class, but is no longer encoded or decoded.
    Class,

    /// Pseudo non-encoded format that means one of F1, F2, F4 or F8.
    VInt,

    /// Pseudo non-encoded format: like VSize but the size is optimized out.
    OVSize,
}

impl fmt::Display for TagFormat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            TagFormat::F1 => write!(f, "F1"),
            TagFormat::F2 => write!(f, "F2"),
            TagFormat::F4 => write!(f, "F4"),
            TagFormat::F8 => write!(f, "F8"),
            TagFormat::Size => write!(f, "Size"),
            TagFormat::VSize => write!(f, "VSize"),
            TagFormat::FSize => write!(f, "FSize"),
            TagFormat::Class => write!(f, "Class"),
            TagFormat::VInt => write!(f, "VInt"),
            TagFormat::OVSize => write!(f, "OVSize"),
        }
    }
}
