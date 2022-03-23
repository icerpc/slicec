// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::grammar::{Attribute, Module, FileEncoding, SliceEncoding};
use crate::ptr_util::WeakPtr;

#[derive(Clone, Debug)]
pub struct Location {
    pub start: (usize, usize),
    pub end: (usize, usize),
    pub file: String,
}

pub struct SliceFile {
    pub filename: String,
    pub relative_path: String,
    pub raw_text: String,
    pub contents: Vec<WeakPtr<Module>>,
    pub attributes: Vec<Attribute>,
    pub encoding: Option<FileEncoding>,
    pub is_source: bool,
    line_positions: Vec<usize>,
}

impl SliceFile {
    pub fn new(
        relative_path: String,
        raw_text: String,
        contents: Vec<WeakPtr<Module>>,
        attributes: Vec<Attribute>,
        encoding: Option<FileEncoding>,
        is_source: bool,
    ) -> SliceFile {
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
        let filename = std::path::Path::new(&relative_path)
            .file_stem()
            .unwrap()
            .to_os_string()
            .into_string()
            .unwrap();

        SliceFile {
            filename,
            relative_path,
            raw_text,
            contents,
            attributes,
            encoding,
            is_source,
            line_positions,
        }
    }

    /// Returns the Slice encoding used by this file.
    ///
    /// If no encoding was explicitely declared, it returns the default encoding.
    ///
    /// See [SliceEncoding::default()](crate::grammar::SliceEncoding::default())
    pub fn encoding(&self) -> SliceEncoding {
        self.encoding
            .as_ref()
            .map_or(SliceEncoding::default(), |encoding| encoding.version)
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
