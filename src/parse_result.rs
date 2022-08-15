// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::ast::Ast;
use crate::diagnostics::*;
use crate::slice_file::SliceFile;
use std::collections::HashMap;

pub struct ParsedData {
    pub ast: Ast,
    pub diagnostic_reporter: DiagnosticReporter,
    pub files: HashMap<String, SliceFile>,
}

impl ParsedData {
    pub fn into_exit_code(self) -> i32 {
        if self.has_errors() {
            Self::emit_errors(self.diagnostic_reporter, &self.files);
            1
        } else {
            0
        }
    }

    pub fn has_errors(&self) -> bool {
        self.diagnostic_reporter.has_diagnostics()
    }

    fn emit_errors(diagnostic_reporter: DiagnosticReporter, files: &HashMap<String, SliceFile>) {
        let counts = diagnostic_reporter.get_totals();

        for error in diagnostic_reporter.into_diagnostics() {
            let prefix = match error.diagnostic_kind {
                DiagnosticKind::SyntaxError(_) | DiagnosticKind::LogicError(_) | DiagnosticKind::IOError(_) => "error",
                DiagnosticKind::Warning(_) => "warning",
                DiagnosticKind::Note(_) => "note",
            };

            // Insert the prefix at the start of the message.
            let mut message = format!("{prefix}: {error}");

            if let Some(span) = error.span {
                let file = &span.file;
                // Specify the span where the error starts on its own line after the message.
                message = format!("{message}\n@ '{file}' ({},{})", span.start.0, span.start.1);

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
