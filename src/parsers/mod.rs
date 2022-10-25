// Copyright (c) ZeroC, Inc. All rights reserved.

//! TODO write a comment about how parsing works in Slice.

// We only export the preprocessor and parser to keep all the other logic private.
pub use self::preprocessor::parser::Preprocessor;
pub use self::slice::parser::Parser;

mod common;
mod preprocessor;
mod slice;
