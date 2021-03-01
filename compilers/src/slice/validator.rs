
use crate::error::ErrorHandler;
use crate::visitor::Visitor;

//------------------------------------------------------------------------------
// Vaidator
//------------------------------------------------------------------------------
#[derive(Debug)]
pub(crate) struct Validator<'a> {
    error_handler: &'a mut ErrorHandler,
}

impl<'a> Validator<'a> {
    pub(crate) fn new(error_handler: &'a mut ErrorHandler) -> Self {
        Validator { error_handler }
    }
}

impl<'a> Visitor for Validator<'a> {
    // TODO add validation logic here!
}
