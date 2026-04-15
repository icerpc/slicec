// Copyright (c) ZeroC, Inc.

mod diagnostic;
mod errors;
mod lints;

pub use diagnostic::*;
pub use errors::Error;
pub use lints::Lint;

use crate::slice_file::Span;
use serde::Serialize;

/// Stores additional information about a diagnostic.
#[derive(Serialize, Debug, Clone)]
pub struct Note {
    pub message: String,
    pub span: Option<Span>,
}
