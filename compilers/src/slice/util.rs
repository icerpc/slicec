
//------------------------------------------------------------------------------
// Location
//------------------------------------------------------------------------------
#[derive(Clone, Debug)]
pub struct Location {
    pub start: (usize, usize),
    pub end: (usize, usize),
    pub file: String,
}

//------------------------------------------------------------------------------
// SliceError
//------------------------------------------------------------------------------
#[derive(Debug)]
pub struct SliceError {
    message: String,
    severity: SliceErrorLevel,
    location: Option<Location>,
}

impl SliceError {
    pub fn new(message: String, severity: SliceErrorLevel) -> Self {
        SliceError { message, severity, location: None }
    }

    pub fn new_with_location(message: String, severity: SliceErrorLevel, loc: Location) -> Self {
        SliceError { message, severity, location: Some(loc) }
    }
}

//------------------------------------------------------------------------------
// SliceErrorLevel
//------------------------------------------------------------------------------
#[derive(Clone, Eq, Hash, PartialEq, Debug)]
pub enum SliceErrorLevel {
    Error,
    Warning,
    Note,
}
