// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::ast::Ast;
use crate::diagnostics::*;
use crate::slice_file::SliceFile;
use console::style;
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
        self.diagnostic_reporter.has_errors()
    }

    fn emit_errors(diagnostic_reporter: DiagnosticReporter, files: &HashMap<String, SliceFile>) {
        let counts = diagnostic_reporter.get_totals();
        for diagnostic in diagnostic_reporter.into_diagnostics() {
            // Style the prefix. Note that for `Notes` we do not insert a newline since they should be "attached"
            // to the previously emitted diagnostic.
            let prefix = match diagnostic.diagnostic_kind {
                DiagnosticKind::SyntaxError(_) | DiagnosticKind::LogicError(_) | DiagnosticKind::IOError(_) => {
                    style("\nerror").red()
                }
                DiagnosticKind::Warning(_) => style("\nwarning").yellow(),
                DiagnosticKind::Note(_) => style("note").white(),
            }
            .bold();

            // Create the message using the prefix
            match diagnostic.diagnostic_kind {
                DiagnosticKind::Note(_) => {
                    eprintln!("    {} {}: {}", style("=").blue().bold(), prefix, style(&diagnostic))
                }
                _ => eprintln!("{}: {}", prefix, style(&diagnostic).bold()),
            };

            // If the diagnostic contains a location, show a snippet containing the offending code
            if let Some(span) = diagnostic.span {
                // Display the file name and line row and column where the error began.
                let file_location = format!("{}:{}:{}", &span.file, span.start.row, span.start.col);
                let path = std::path::Path::new(&file_location);
                eprintln!(" {} {}", style("-->").blue().bold(), path.display());

                // Display the line of code where the error occurred.
                let snippet = files.get(&span.file).unwrap().get_snippet(span.start, span.end);
                eprintln!("{}", snippet);
            }
        }

        // Output the total number of errors and warnings.
        println!();
        if counts.1 != 0 {
            println!(
                "{}: Compilation generated {} warning(s)",
                style("Warnings").yellow().bold(),
                counts.1,
            )
        }
        if counts.0 != 0 {
            println!(
                "{}: Compilation failed with {} error(s)",
                style("Failed").red().bold(),
                counts.0,
            )
        }
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
