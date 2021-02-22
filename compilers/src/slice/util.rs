
// TODO we should make this hold line/col numbers instead of just a start and end.
// it's easier to go from line/col -> start/end than it is the other way around.
//------------------------------------------------------------------------------
// Location
//------------------------------------------------------------------------------
#[derive(Clone, Debug)]
pub struct Location {
    pub start: usize,
    pub end: usize,
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
