// Copyright (c) ZeroC, Inc. All rights reserved.

//! TODO write a comment about how parsing works in Slice.

// We only export the parser and preprocessor and keep all the other logic private.
pub use self::preprocessor::parser::Preprocessor;
pub use self::slice::parser::Parser;

mod common;
mod preprocessor;
mod slice;
