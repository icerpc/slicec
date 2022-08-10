// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::ast::Ast;
use crate::errors::*;
use crate::utils::slice_file::SliceFile;
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
                ErrorKind::Syntax(_) | ErrorKind::Logic(_) | ErrorKind::IO(_) => "error",
                ErrorKind::Warning(_) => "warning",
                ErrorKind::Note(_) => "note",
            };

            // Insert the prefix at the start of the message.
            let mut message = format!("{}: {}", prefix, &error);

            if let Some(span) = error.span {
                // Specify the span where the error starts on its own line after the message.
                message = format!("{}\n@ '{}' ({},{})", message, &span.file, span.start.0, span.start.1);

                // If the span isn't empty, extract a snippet of the text contained within the span.
                if span.start != span.end {
                    message += ":\n";
                    let file = files.get(&span.file).expect("Slice file not in file map!");
                    message += &file.get_snippet(span.start, span.end);
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
