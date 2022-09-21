// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::grammar::{Attributable, Attribute, Encoding, FileEncoding, Module};
use crate::utils::ptr_util::WeakPtr;
use console::style;
use serde::Serialize;
use std::fmt::Write;

/// Stores the row and column numbers of a location in a Slice file.
/// These values are indexed starting at 1 instead of 0 for human readability.
/// Ex: (1,1) is the start of a file: the first column in the first row.
#[derive(Serialize, Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Location {
    pub row: usize,
    pub col: usize,
}

impl From<(usize, usize)> for Location {
    /// Creates a [Location] from a pair of indices, where the first element represents the line number,
    /// and the second element represents the column number.
    fn from(x: (usize, usize)) -> Self {
        Location { row: x.0, col: x.1 }
    }
}

impl Default for Location {
    /// Returns a [Location] representing the start of a file: (1,1).
    fn default() -> Self {
        Location { row: 1, col: 1 }
    }
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub struct Span {
    pub start: Location,
    pub end: Location,
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
    /// If no encoding was explicitly declared, it returns the default encoding.
    ///
    /// See [Encoding::default()](crate::grammar::Encoding::default())
    pub fn encoding(&self) -> Encoding {
        self.encoding
            .as_ref()
            .map_or(Encoding::default(), |encoding| encoding.version)
    }

    /// Retrieves a formatted snippet from the slice file.
    pub(crate) fn get_snippet(&self, start: Location, end: Location) -> String {
        debug_assert!(start < end); // Assert that the start of the snippet comes before the end.

        // The snippet of code on the same line as the error, but directly before it.
        let start_snippet = &self.raw_text[self.line_positions[start.row - 1]..self.raw_pos(start)];

        // The snippet of code containing the error.
        let error_snippet = &self.raw_text[self.raw_pos(start)..self.raw_pos(end)];

        // The snippet of code on the same line as the error, but directly after it.
        let end_snippet = &self.raw_text[self.raw_pos(end)..self.line_positions[end.row]];

        // Create an underline that is the length of the error snippet. For error snippets that span multiple
        // lines, the underline is the length of the longest line.
        let underline = "-".repeat(error_snippet.lines().map(|line| line.len()).max().unwrap());
        let mut line_number = start.row;

        // Create a formatted snippet.
        let mut snippet = style("    |\n").blue().bold().to_string();
        for line in format!("{}{}{}", start_snippet, style(error_snippet), end_snippet).lines() {
            writeln!(
                snippet,
                "{: <4}{} {}",
                style(line_number).blue().bold(),
                style("|").blue().bold(),
                line,
            )
            .unwrap();
            line_number += 1;
        }
        writeln!(
            snippet,
            "{}{}{}",
            style("    | ").blue().bold(),
            " ".repeat(start_snippet.len()),
            style(underline).yellow().bold(),
        )
        .unwrap();
        write!(snippet, "{}", style("    |").blue().bold()).unwrap();

        // Return the formatted snippet.
        snippet
    }

    /// Converts the provided [Location] into an index in the file's raw text.
    fn raw_pos(&self, location: Location) -> usize {
        // `row` and `col` are decremented because they are indexed starting at 1 instead of 0.
        self.line_positions[location.row - 1] + (location.col - 1)
    }
}

impl Attributable for SliceFile {
    fn attributes(&self) -> &Vec<Attribute> {
        &self.attributes
    }

    fn get_raw_attribute(&self, directive: &str, recurse: bool) -> Option<&Attribute> {
        if recurse {
            panic!("Cannot recursively get attributes on a Slice file");
        }
        self.attributes
            .iter()
            .find(|&attribute| attribute.prefixed_directive == directive)
    }
}
