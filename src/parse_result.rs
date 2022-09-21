// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::ast::Ast;
use crate::command_line::DiagnosticFormat;
use crate::diagnostics::*;
use crate::slice_file::{SliceFile, Span};
use console::style;
use std::collections::HashMap;

pub struct ParsedData {
    pub ast: Ast,
    pub diagnostic_reporter: DiagnosticReporter,
    pub files: HashMap<String, SliceFile>,
}

impl ParsedData {
    pub fn into_exit_code(self) -> i32 {
        // Emit any diagnostics that were reported.
        let has_errors = self.has_errors();
        Self::emit_diagnostics(self.diagnostic_reporter, &self.files);

        i32::from(has_errors)
    }

    pub fn has_errors(&self) -> bool {
        self.diagnostic_reporter.has_errors()
    }

    fn emit_diagnostics(diagnostic_reporter: DiagnosticReporter, files: &HashMap<String, SliceFile>) {
        match diagnostic_reporter.diagnostic_format {
            DiagnosticFormat::Human => Self::output_to_console(diagnostic_reporter, files),
            DiagnosticFormat::Json => Self::output_to_json(diagnostic_reporter),
        }
    }

    fn output_to_json(diagnostic_reporter: DiagnosticReporter) {
        for diagnostic in diagnostic_reporter.into_diagnostics() {
            let json = serde_json::to_string(&diagnostic).expect("Failed to serialize diagnostic to JSON");
            println!("{json}");
        }
    }

    fn output_to_console(diagnostic_reporter: DiagnosticReporter, files: &HashMap<String, SliceFile>) {
        let counts = diagnostic_reporter.get_totals();
        for diagnostic in diagnostic_reporter.into_diagnostics() {
            // Style the prefix. Note that for `Notes` we do not insert a newline since they should be "attached"
            // to the previously emitted diagnostic.
            let error_code = diagnostic.error_code().map_or_else(|| String::new(), |code| format!("[{code}]"));
            let prefix = match diagnostic {
                Diagnostic::Error(_) => {
                    format!("{} {}", style("error").red().bold(), style(error_code).red().bold())
                }
                Diagnostic::Warning(_) => format!(
                    "{} {}",
                    style("warning").yellow().bold(),
                    style(error_code).yellow().bold()
                ),
            };

            // Emit the message with the prefix.
            eprintln!("{}: {}", prefix, style(&diagnostic).bold());

            // If the diagnostic contains a span, show a snippet containing the offending code.
            if let Some(span) = diagnostic.span() {
                Self::show_snippet(span, files)
            }
            // If the diagnostic contains notes, display them.
            diagnostic.notes().iter().for_each(|note| {
                eprintln!(
                    "    {} {}: {:}",
                    style("=").blue().bold(),
                    style("note").bold(),
                    style(&note).bold(),
                );
                if let Some(span) = &note.span {
                    Self::show_snippet(span, files)
                }
            });
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

    fn show_snippet(span: &Span, files: &HashMap<String, SliceFile>) {
        // Display the file name and line row and column where the error began.
        let file_location = format!("{}:{}:{}", &span.file, span.start.row, span.start.col);
        let path = std::path::Path::new(&file_location);
        eprintln!(" {} {}", style("-->").blue().bold(), path.display());

        // Display the line of code where the error occurred.
        let snippet = files.get(&span.file).unwrap().get_snippet(span.start, span.end);
        eprintln!("{}", snippet);
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
