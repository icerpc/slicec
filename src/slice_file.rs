// Copyright (c) ZeroC, Inc.

use crate::grammar::{Attributable, Attribute, Encoding, FileEncoding, Module};
use crate::utils::ptr_util::WeakPtr;
use console::style;
use serde::Serialize;
use std::fmt::{Display, Write};

const EXPANDED_TAB: &str = "    ";
const SPACE: &str = " ";

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

impl Span {
    pub fn new(start: Location, end: Location, file: &str) -> Self {
        let file = file.to_owned();
        Span { start, end, file }
    }
}

#[derive(Debug)]
pub struct SliceFile {
    pub filename: String,
    pub relative_path: String,
    pub raw_text: String,
    pub contents: Option<WeakPtr<Module>>, // TODO split the module declaration from the contents.
    pub attributes: Vec<WeakPtr<Attribute>>,
    pub encoding: Option<FileEncoding>,
    pub is_source: bool,
    line_positions: Vec<usize>,
}

impl SliceFile {
    pub fn new(relative_path: String, raw_text: String, is_source: bool) -> Self {
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
        // Treat EOF as an end-of-line character.
        line_positions.push(raw_text.chars().count());

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
            contents: None,
            attributes: Vec::new(),
            encoding: None,
            is_source,
            line_positions,
        }
    }

    /// Returns the Slice encoding used by this file.
    ///
    /// If no encoding was explicitly declared, it returns the default encoding.
    ///
    /// See [Encoding::default()](crate::grammar::Encoding::default)
    pub fn encoding(&self) -> Encoding {
        self.encoding
            .as_ref()
            .map_or(Encoding::default(), |encoding| encoding.version)
    }

    /// Retrieves a formatted snippet from the slice file.
    #[allow(unused_must_use)] // 'writeln' can't fail when writing to a string, so we ignore the result it returns.
    pub(crate) fn get_snippet(&self, start: Location, end: Location) -> String {
        debug_assert!(start <= end);

        // The number of columns that should be reserved for displaying line numbers to the left of snippets.
        // Equal to the number of digits in the longest line number plus one (longest number is always the end).
        // Ex:     "273 | source code"    `line_number_prefix_length` would be 4 for "273 " (4 chars long).
        let line_number_prefix_length = end.row.to_string().len() + 1;

        // Returns a formatted line prefix of the form: "[line number]<padding>|".
        let line_number_prefix = |line_number: Option<usize>| {
            // If a line number was provided, use it, otherwise use the empty string.
            let number_string: &dyn Display = line_number.as_ref().map_or(&"", |i| i);
            // Pad the string with spaces (on the right) so its total length is `line_number_prefix_length`.
            let padded_number_string = format!("{number_string:<line_number_prefix_length$}|");
            // Style the string and return it.
            style(padded_number_string).blue().bold().to_string()
        };

        // The prefix for lines not showing a line number.
        let line_prefix = line_number_prefix(None);

        // Raw text from the slice file. Contains all the lines that the specified range touches.
        // IMPORTANT NOTE: rows and columns are counted from 1 (not 0), so we have to `-1` them everywhere!
        let raw_snippet = &self.raw_text[self.line_positions[start.row - 1]..self.line_positions[end.row] - 1];
        // Convert the provided locations into string indexes (in the raw text).
        let start_pos = self.line_positions[start.row - 1] + (start.col - 1);
        let end_pos = self.line_positions[end.row - 1] + (end.col - 1);

        let mut formatted_snippet = line_number_prefix(None) + "\n";
        // Iterate through each line of raw text, and add it (and its line number) into the formatted snippet.
        // Add pointers and underlining on the line below it, as specified by the provided range.
        // We use `str::split` instead of `str::lines` to preserve '\r's, since our indexes count them as characters.
        let mut line_number = start.row;
        for line in raw_snippet.split('\n') {
            // We print tabs as 4 spaces so that we can properly compute the underline length.
            writeln!(
                formatted_snippet,
                "{} {}",
                line_number_prefix(Some(line_number)),
                line.replace('\t', EXPANDED_TAB)
            );

            if start_pos == end_pos {
                // If the provided range is a single location, point to that location.
                let point = style("/\\").yellow().bold();
                let point_offset = start_pos - self.line_positions[line_number - 1];
                let whitespace = whitespace_to_position(line, point_offset);

                writeln!(formatted_snippet, "{line_prefix} {whitespace}{point}");
            } else {
                // If the provided range is between 2 locations, underline everything between them.
                let underline_start = start_pos.saturating_sub(self.line_positions[line_number - 1]);
                let underline_end = match (self.line_positions[line_number] - 1).checked_sub(end_pos) {
                    Some(pos) => line.len() - pos, // If the end position is on this line.
                    None => line.trim_end().len(), // If the end position is past the end of this line.
                };

                // Number of tabs between the start and end of the underline.
                let underline_tab_count = line
                    .chars()
                    .enumerate()
                    .filter(|(index, char)| *index >= underline_start && *index < underline_end && *char == '\t')
                    .count();

                // Since tab is only 1 character, we have to account for the extra 3 characters that are displayed for
                // each tab.
                let underline_length =
                    (underline_end - underline_start) + (underline_tab_count * (EXPANDED_TAB.len() - 1));
                let underline = style(format!("{:-<1$}", "", underline_length)).yellow().bold();

                // The whitespace that should be displayed before the underline. Tabs are displayed as 4 spaces.
                let whitespace = whitespace_to_position(line, underline_start);

                writeln!(formatted_snippet, "{line_prefix} {whitespace}{underline}",);
            }
            line_number += 1; // Move to the next line.
        }
        formatted_snippet + &line_prefix
    }
}

impl Attributable for SliceFile {
    fn attributes(&self) -> Vec<&Attribute> {
        self.attributes.iter().map(WeakPtr::borrow).collect()
    }

    fn all_attributes(&self) -> Vec<Vec<&Attribute>> {
        vec![self.attributes()]
    }
}

fn whitespace_to_position(line: &str, pos: usize) -> String {
    line.chars()
        .take(pos)
        .map(|c| if c == '\t' { EXPANDED_TAB } else { SPACE })
        .collect()
}
