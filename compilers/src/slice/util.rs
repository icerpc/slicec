
use std::path::Path;

//------------------------------------------------------------------------------
// Location
//------------------------------------------------------------------------------
/// Describes a single position, or a span over two positions in a slice file.
#[derive(Clone, Debug)]
pub struct Location {
    /// The starting position, stored as a tuple of line number, and column number, in that order.
    pub start: (usize, usize),
    /// The ending position, stored as a tuple of line number, and column number, in that order.
    pub end: (usize, usize),
    /// The path of the slice file where this location is in.
    pub file: String,
}

//------------------------------------------------------------------------------
// SliceFile
//------------------------------------------------------------------------------
/// Stores information about a single slice file, and it's contents.
#[derive(Debug)]
pub struct SliceFile {
    /// The pathless filename of the slice file (without it's '.ice' extension).
    pub filename: String,
    /// The path of the slice file, relative to where the slice compiler was run from (including it's '.ice' extension).
    pub path: String,
    /// The raw text contained in the slice file.
    pub raw_text: String,
    /// The AST indices of all the top-level definitions in the slice file, in the order they're defined.
    pub contents: Vec<usize>,
    /// True if this slice file is a source file (that code should be generated for), or false if it's a reference file.
    pub is_source: bool,
    /// Stores the starting position of every new line in the file. We pre-compute these when the SliceFile is first
    /// created and cache them here, to make snippet extraction and line referencing more efficient.
    line_positions: Vec<usize>,
}

impl SliceFile {
    /// Creates a new slice file
    pub(crate) fn new(path: String, raw_text: String, contents: Vec<usize>, is_source: bool) -> Self {
        // Store the starting position of each line the file.
        // These are needed to translate `(line,col)` positions into string indices for snippets.
        let mut line_positions = vec![0]; // The first line always starts at index 0.
        let mut last_char_was_carriage_return = false;
        for (index, character) in raw_text.chars().enumerate() {
            if character == '\n' {
                line_positions.push(index + 1);
                last_char_was_carriage_return = false;
            } else {
                if last_char_was_carriage_return {
                    line_positions.push(index);
                }
                last_char_was_carriage_return = character == '\r';
            }
        }

        // Extract the name of the slice file without it's extension or directory's path.
        let filename = Path::new(&path).file_stem().unwrap().to_os_string().into_string().unwrap();

        SliceFile { filename, path, raw_text, contents, is_source, line_positions }
    }

    /// TODO
    pub(crate) fn get_snippet(&self, start: (usize, usize), end: (usize, usize)) -> &str {
        // TODO we should return nice snippets that snap whole lines and have underlining, etc...
        &self.raw_text[self.raw_pos(start)..self.raw_pos(end)]
    }

    /// Calculates the position in this file's raw text corresponding to the provided line and column numbers.
    fn raw_pos(&self, (line, col): (usize, usize)) -> usize {
        self.line_positions[line - 1] + (col - 1)
    }
}
