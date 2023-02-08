// Copyright (c) ZeroC, Inc.

//! This module contains common types and functions that are useful to multiple parsers.

use crate::slice_file::Location;

/// Stores a reference to a block of source code in a Slice file.
#[derive(Clone, Copy, Debug)]
pub struct SourceBlock<'input> {
    /// The raw text contained in the block, taken directly from the input.
    pub content: &'input str,
    /// The starting [Location] of the block in its source file.
    pub start: Location,
    /// The ending [Location] of the block in its source file.
    pub end: Location,
}

/// A specialized [Result] type used by parsing functions. The `Err` variant is empty because errors are reported with
/// a [DiagnosticReporter](crate::diagnostics::DiagnosticReporter) instead of being directly returned.
pub type ParserResult<T> = Result<T, ()>;
