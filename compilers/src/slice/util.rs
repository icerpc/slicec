
//------------------------------------------------------------------------------
// SliceFile
//------------------------------------------------------------------------------

#[derive(Debug)]
pub struct SliceFile {
    filename: String,
    raw_text: String,
    to_generate: Vec<usize>,
}

impl SliceFile {
    pub fn new() /*-> Self*/ {
        // TODO
    }

    pub fn filename(&self) -> &str {
        &self.filename
    }

    pub fn raw_text(&self) -> &str {
        &self.raw_text()
    }

    pub fn get_text(&self, location: &Location) -> &str {
        &self.raw_text[location.start .. location.end]
    }

    pub fn to_generate(&self) -> &Vec<usize> {
        &self.to_generate
    }
}

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




/// Custom error type that holds information about a parsing-related error.

pub struct ParserError
{
    message: String,
    location: Location,
}

impl ParserError
{
    pub fn new(message: String, location: Location) -> Self {
        Self {
            message,
            location
        }
    }
}

pub type ParseResult = Result<(usize, Vec<SliceFile>), ParserError>;
