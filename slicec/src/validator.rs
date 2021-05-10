// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::error::ErrorHandler;
use crate::visitor::Visitor;

/// Validator visits all the elements in a slice file to check for additional errors and warnings not caught by previous
/// phases of parsing and that are common to all slice compilers.
#[derive(Debug)]
pub(crate) struct Validator<'a> {
    /// Reference to the parser's error handler,
    error_handler: &'a mut ErrorHandler,
}

impl<'a> Validator<'a> {
    /// Creates a new validator.
    pub(crate) fn new(error_handler: &'a mut ErrorHandler) -> Self {
        Validator { error_handler }
    }
}

impl<'a> Visitor for Validator<'a> {
    // TODO add validation logic here.
}
