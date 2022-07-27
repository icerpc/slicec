// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::ast::Ast;
use crate::errors::*;
use crate::slice_file::SliceFile;
use std::collections::HashMap;

pub struct ParsedData {
    pub ast: Ast,
    pub error_reporter: ErrorReporter,
    pub files: HashMap<String, SliceFile>,
}

impl ParsedData {
    pub fn into_exit_code(self) -> i32 {
        if self.has_errors() {
            Self::emit_errors(self.error_reporter, &self.files);
            1
        } else {
            0
        }
    }

    pub fn has_errors(&self) -> bool {
        self.error_reporter.has_errors()
    }

    fn emit_errors(error_reporter: ErrorReporter, files: &HashMap<String, SliceFile>) {
        let counts = error_reporter.get_totals();

        for error in error_reporter.into_errors() {
            let prefix = match error.error_kind {
                ErrorKind::Rule(_) => "error",
                ErrorKind::Warning(_) => "warning",
                ErrorKind::Note(_) => "note",
                _ => "error",
            };

            // Insert the prefix at the start of the message.
            let mut message = prefix.to_owned() + ": " + &error.error_kind.to_string();

            if let Some(location) = error.location {
                // Specify the location where the error starts on its own line after the message.
                message = format!(
                    "{}\n@ '{}' ({},{})",
                    message, &location.file, location.start.0, location.start.1
                );

                // If the location spans between two positions, add a snippet from the slice file.
                if location.start != location.end {
                    message += ":\n";
                    let file = files.get(&location.file).expect("Slice file not in file map!");
                    message += &file.get_snippet(location.start, location.end);
                } else {
                    message += "\n";
                }
            }
            // Print the message to stderr.
            eprintln!("{}", message);
        }

        println!(
            "Compilation failed with {} error(s) and {} warning(s).\n",
            counts.0, counts.1
        );
    }
}

impl From<ParsedData> for ParserResult {
    fn from(parsed_data: ParsedData) -> Self {
        match parsed_data.has_errors() {
            false => Ok(parsed_data),
            true => Err(parsed_data),
        }
    }
}

pub type ParserResult = Result<ParsedData, ParsedData>;
