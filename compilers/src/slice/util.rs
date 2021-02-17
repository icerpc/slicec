
use crate::grammar::*;
use crate::visitor::Visitable;



//------------------------------------------------------------------------------
// Location
//------------------------------------------------------------------------------

#[derive(Clone, Copy, Debug)]
pub struct Location {
    pub start: usize,
    pub end: usize,
}

impl Location {
    // TODO
}

use std::collections::HashMap;

pub type SliceTable = HashMap<String, usize>;







/// Custom error type that holds information about a parsing-related error.

pub struct SliceError
{
    message: String,
    location: Location,
}

impl SliceError
{
    pub fn new(message: String, location: Location) -> Self {
        Self {
            message,
            location
        }
    }
}

