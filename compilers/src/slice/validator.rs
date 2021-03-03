
use crate::error::ErrorHandler;
use crate::visitor::Visitor;

//------------------------------------------------------------------------------
// Vaidator
//------------------------------------------------------------------------------
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

// TODO add validation logic here.
impl<'a> Visitor for Validator<'a> {
}
