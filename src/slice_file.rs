// Copyright (c) ZeroC, Inc.

use crate::grammar::{implement_Attributable_for, Attributable, Attribute, Definition, Encoding, FileEncoding, Module};
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

    pub encoding: Option<FileEncoding>,
    pub module: Option<WeakPtr<Module>>,
    pub attributes: Vec<WeakPtr<Attribute>>,
    pub contents: Vec<Definition>,

    pub is_source: bool,
}

impl SliceFile {
    pub fn new(relative_path: String, raw_text: String, is_source: bool) -> Self {
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
            encoding: None,
            module: None,
            attributes: Vec::new(),
            contents: Vec::new(),
            is_source,
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

        // The lines of text that should be included in the snippet.
        let lines = self
            .raw_text
            .lines()
            .enumerate()
            .filter(|(line_number, _)| *line_number >= start.row - 1 && *line_number < end.row);

        // The formatted snippet.
        let mut formatted_snippet = line_prefix.clone() + "\n";

        for (i, line) in lines {
            // The actual line number
            let line_number = i + 1;

            let width = line.chars().count();

            // The start and end positions of the underline.
            let underline_start = if line_number == start.row { start.col - 1 } else { 0 };
            let underline_end = if line_number == end.row { end.col - 1 } else { width };

            // If the start and end are not the same and the underline start the start position is at the end of the
            // line, then we don't want to print the line.
            if start != end && underline_start == width {
                continue;
            }

            // Expand tabs to 4 spaces so that we can properly compute the underline length.
            let prefix = line_number_prefix(Some(line_number));
            let space_separated_line = line.replace('\t', EXPANDED_TAB);
            formatted_snippet += prefix + ' ' + &space_separated_line + '\n';

            let underline = get_underline(line, underline_start, underline_end);
            writeln!(formatted_snippet, "{line_prefix} {underline}").expect("failed to write snippet");
        }

        formatted_snippet + &line_prefix
    }
}

implement_Attributable_for!(SliceFile);

fn get_underline(line: &str, underline_start: usize, underline_end: usize) -> String {
    // The whitespace that should be displayed before the underline. Tabs are expanded to 4 spaces.
    let whitespace: String = line
        .chars()
        .take(underline_start)
        .map(|c| if c == '\t' { EXPANDED_TAB } else { SPACE })
        .collect();

    // The underline that should be displayed.
    // If the underline is a single character, then we use a single point.
    // If the provided range is between 2 locations, underline everything between them.
    let underline = if underline_start == underline_end {
        // Point to a single character.
        style(r#"/\"#.to_owned()).yellow().bold()
    } else {
        // Number of tabs between the start and end of the underline.
        let underline_tab_count = line
            .chars()
            .skip(underline_start)
            .take(underline_end - underline_start)
            .filter(|c| *c == '\t')
            .count();

        // Since tab is only 1 character, we have to account for the extra 3 characters that are displayed
        // for each tab.
        let underline_length = (underline_end - underline_start) + (underline_tab_count * (EXPANDED_TAB.len() - 1));
        style(format!("{:-<1$}", "", underline_length)).yellow().bold()
    };

    // The whitespace that should be displayed before the underline. Tabs are displayed as 4 spaces.
    whitespace + &underline
}
