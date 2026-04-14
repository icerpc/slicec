// Copyright (c) ZeroC, Inc.

mod diagnostic;
mod errors;
mod lints;

pub use diagnostic::{Diagnostic, Diagnostics};
pub use errors::Error;
pub use lints::Lint;

use crate::slice_file::Span;
use serde::Serialize;

/// Wrapper enum for the 3 possible kinds of diagnostics.
#[derive(Debug)]
pub enum DiagnosticKind {
    /// An irrecoverable error; the compiler will terminate early at the end of the next phase.
    Error(Error),

    /// A minor and recoverable mistake; has no effect on the compiler's execution pipe-line.
    Lint(Lint),
}

/// Diagnostic levels describe the severity of a diagnostic, and how the compiler should react to their emission.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum DiagnosticLevel {
    #[rustfmt::skip] // See https://github.com/rust-lang/rustfmt/issues/5801
    /// Diagnostics with the `Error` level will be emitted and will cause compilation to fail with a non-zero exit code.
    Error,

    /// Diagnostics with the `Warning` level will be emitted, but will not influence the exit code of the compiler.
    Warning,

    /// Diagnostics with the `Allowed` level are ignored by the compiler and will not emit any message.
    Allowed,
}

/// Stores additional information about a diagnostic.
#[derive(Serialize, Debug, Clone)]
pub struct Note {
    pub message: String,
    pub span: Option<Span>,
}
