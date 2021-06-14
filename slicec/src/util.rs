// Copyright (c) ZeroC, Inc. All rights reserved.

use std::path::Path;

/// Describes a single position, or a span over two positions in a slice file.
#[derive(Clone, Debug)]
pub struct Location {
    /// The starting position, stored as a tuple of line number, and column number, in that order.
    pub start: (usize, usize),
    /// The ending position, stored as a tuple of line number, and column number, in that order.
    pub end: (usize, usize),
    /// The path of the slice file this location is in.
    pub file: String,
}

/// Stores information about a single slice file, and its contents.
#[derive(Debug)]
pub struct SliceFile {
    /// The filename of the slice file (without its '.ice' extension).
    pub filename: String,
    /// The path of the slice file, relative to where the compiler was run from
    /// (including its '.ice' extension).
    pub relative_path: String,
    /// The raw text contained in the slice file.
    pub raw_text: String,
    /// The AST indices of all the top-level definitions contained in the slice file,
    /// in the order they were defined.
    pub contents: Vec<usize>,
    /// True if the slice file is a source file (which code should be generated for),
    /// or false if it's a reference file.
    pub is_source: bool,
    /// Stores the starting position of every line in the file. We compute these when the SliceFile
    /// is first created and cache them here, to speed up snippet extraction and line referencing.
    line_positions: Vec<usize>,
}

impl SliceFile {
    /// Creates a new slice file
    pub(crate) fn new(relative_path: String, raw_text: String, contents: Vec<usize>, is_source: bool) -> Self {
        // Store the starting position of each line the file.
        // Slice supports '\n', '\r', and '\r\n' as newlines.
        let mut line_positions = vec![0]; // The first line always starts at index 0.
        let mut last_char_was_carriage_return = false;

        // Iterate through each character in the file.
        // If we hit a '\n' we immediately store `index + 1` as the starting position for the next
        // line (`+ 1` because the line starts after the newline character).
        // If we hit a '\r' we wait and read the next character to see if it's a '\n'.
        // If so, the '\n' block handles it, otherwise we store `index`
        // (no plus one, because we've already read ahead to the next character).
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

        // Extract the name of the slice file without its extension.
        let filename = Path::new(&relative_path).file_stem().unwrap().to_os_string().into_string().unwrap();

        SliceFile { filename, relative_path, raw_text, contents, is_source, line_positions }
    }

    /// Retrieves a formatted snippet from the slice file. This method expects `start < end`.
    pub(crate) fn get_snippet(&self, start: (usize, usize), end: (usize, usize)) -> String {
        // TODO we should return nice snippets that snap whole lines and have underlining, etc...
        self.raw_text[self.raw_pos(start)..self.raw_pos(end)].to_owned() + "\n"
    }

    /// Converts the provided line and column numbers into an index in the file's raw text.
    fn raw_pos(&self, (line, col): (usize, usize)) -> usize {
        self.line_positions[line - 1] + (col - 1)
    }
}

/// The context that a type is being used in while generating code. This is used primarily by the
/// `type_to_string` methods in each of the language mapping's code generators.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum TypeContext {
    /// Used when generating the types of data members in structs and classes.
    DataMember,
    /// Used when generating the types of operation members (parameters and return types) in places
    /// where they're being read off the wire and unmarshalled.
    Incoming,
    /// Used when generating the types of operation members (parameters and return types) in places
    /// where they're being going to be marshalled and written onto the wire.
    Outgoing,
    /// Used when generating types that are parts of other types, such as the key & value types of
    /// dictionaries, or the element type of a sequence.
    Nested,
}
