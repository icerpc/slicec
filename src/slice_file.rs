// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::grammar::{Attribute, Encoding, FileEncoding, Module};
use crate::utils::ptr_util::WeakPtr;
use console::style;
use std::fmt::Write;

type Location = (usize, usize);

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
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

    /// Retrieves a formatted snippet from the slice file. This method expects `start < end`.
    pub(crate) fn get_snippet(&self, start: (usize, usize), end: (usize, usize)) -> String {
        // The snippet of code on the same line as the error, but directly before it.
        let start_snippet = self.raw_text[self.raw_pos((start.0, 1))..self.raw_pos(start)].to_owned();

        // The snippet of code containing the error.
        let error_snippet = self.raw_text[self.raw_pos(start)..self.raw_pos(end)].to_owned();

        let end_of_error_line = self.raw_text.lines().nth(end.0 - 1).unwrap().len();

        // The snippet of code on the same line as the error, but directly after it.
        let end_snippet = self.raw_text[self.raw_pos(end)..self.raw_pos((end.0, end_of_error_line + 1))].to_owned();

        // Create an underline that is the length of the error snippet. For error snippets that span multiple
        // lines, the underline is the length of the longest line.
        let underline = "-".repeat(error_snippet.lines().map(|line| line.len()).max().unwrap());
        let mut line_number = start.0;

        // Create a formatted snippet.
        let mut snippet = style("    |\n".to_string()).blue().bold().to_string();
        for line in format!("{}{}{}", start_snippet, style(&error_snippet), end_snippet).lines() {
            writeln!(
                snippet,
                "{: <4}{} {}",
                style(line_number).blue().bold(),
                style("|").blue().bold(),
                line
            )
            .unwrap();
            line_number += 1;
        }
        writeln!(
            snippet,
            "{}{}{}",
            style("    | ").blue().bold(),
            " ".repeat(start_snippet.len()),
            style(underline).yellow().bold()
        )
        .unwrap();
        write!(snippet, "{}", style("    |").blue().bold()).unwrap();

        // Return the formatted snippet.
        snippet
    }

    /// Converts the provided line and column numbers into an index in the file's raw text.
    fn raw_pos(&self, (line, col): (usize, usize)) -> usize {
        self.line_positions[line - 1] + (col - 1)
    }
}
