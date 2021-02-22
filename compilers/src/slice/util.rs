
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
    location: Location,
    severity: SliceErrorLevel,
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
