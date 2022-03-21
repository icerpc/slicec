// Copyright (c) ZeroC, Inc. All rights reserved.

use std::fmt;

#[derive(Clone, Debug, Default, Hash, PartialEq, Eq)]
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

        let module_scope = if is_module {
            parser_scope.clone()
        } else {
            Vec::new()
        };

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

/// These encodings identity the method used to convert Slice types to and from bytes.
/// Only the versions supported by IceRPC are listed here, although other older versions exist.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SliceEncoding {
    /// Version 1.1 of the Slice encoding, supported by IceRPC and Ice 3.5 or greater.
    /// It is supported for interoperability between Ice and IceRPC.
    Slice11,

    /// Version 2 of the Slice encoding, supported by IceRPC.
    /// The default encoding when using IceRPC.
    Slice2,
}

/// This tag format describes how the data is encoded and how it can be skipped by the decoding
/// code if the tagged parameter is present in the buffer but is not known to the receiver.
#[derive(Clone, Debug, PartialEq, Eq)]
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
        match self {
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
