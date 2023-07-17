// Copyright (c) ZeroC, Inc.

use super::Module;
use crate::utils::ptr_util::WeakPtr;
use std::fmt;

#[derive(Clone, Debug, Default)]
pub struct Scope {
    pub parser_scope: String,
    pub module: Option<WeakPtr<Module>>,
}

impl Scope {
    pub fn push_scope(&mut self, scope: &str) {
        if !self.parser_scope.is_empty() {
            self.parser_scope.push_str("::");
        }
        self.parser_scope.push_str(scope);
    }

    pub fn pop_scope(&mut self) {
        if let Some(last_scope_index) = self.parser_scope.rfind("::") {
            // Remove any characters after the last '::' in the string.
            // We ensure that we're only removing additional parser scopes, and not any scopes that came from a module.
            debug_assert!(
                self.parser_scope.len() > self.module.as_ref().unwrap().borrow().nested_module_identifier().len()
            );
            self.parser_scope.truncate(last_scope_index);
        } else {
            // If the string doesn't contain '::', there's only a single scope. We pop it off by clearing the string.
            // This is only possible if we're not in a module, otherwise we'd always have at least 1 module scope.
            debug_assert!(self.module.is_none());
            self.parser_scope.clear();
        }
    }
}

/// Returns the scoped version of the provided identifier.
pub fn get_scoped_identifier(identifier: &str, scope: &str) -> String {
    if scope.is_empty() {
        identifier.to_owned()
    } else {
        scope.to_owned() + "::" + identifier
    }
}

/// This enum specifies all the encodings supported by IceRPC.
///
/// These encodings identity the format used to convert Slice types to and from byte streams.
#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Encoding {
    /// Version 1 of the Slice encoding, supported by IceRPC, and compatible with Ice 3.5 or
    /// greater.
    ///
    /// It is primarily for interoperability between Ice and IceRPC.
    Slice1,

    /// Version 2 of the Slice encoding, supported by IceRPC.
    ///
    /// The default encoding when using IceRPC.
    #[default]
    Slice2,
}

impl fmt::Display for Encoding {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match self {
            Self::Slice1 => "Slice1",
            Self::Slice2 => "Slice2",
        })
    }
}

/// This enum specifies all the compilation modes supported by Slice.
///
/// For convenience it's an alias of `Encoding`, since they are used interchangeably in places.
pub type CompilationMode = Encoding;

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
    OptimizedVSize,
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
            TagFormat::OptimizedVSize => write!(f, "OptimizedVSize"),
        }
    }
}
