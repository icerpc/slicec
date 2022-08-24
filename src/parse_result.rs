// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::ast::Ast;
use crate::diagnostics::*;
use crate::slice_file::{SliceFile, Span};
use colored::*;
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
            // Styling the prefix
            let prefix = match diagnostic.diagnostic_kind {
                DiagnosticKind::SyntaxError(_) | DiagnosticKind::LogicError(_) | DiagnosticKind::IOError(_) => {
                    "error".red()
                }
                DiagnosticKind::Warning(_) => "warning".yellow(),
                DiagnosticKind::Note(_) => "note".blue(),
            }
            .bold();

            // Notes should be handled separately than the other diagnostics.
            match diagnostic.diagnostic_kind {
                DiagnosticKind::Note(note) => eprintln!("{}: {}", prefix, note.bold()),
                _ => eprintln!("\n{}: {}", prefix, &diagnostic.to_string().bold()),
            }

            if let Some(span) = diagnostic.span {
                // Display the file name and line row and column where the error began.
                let file_location = format!("{}:{}:{}", &span.file, span.start.0, span.start.1);
                let path = std::path::Path::new(&file_location);
                let formatted_path = format!(" {} {}", "-->".blue().bold(), path.display());
                eprintln!("{}", formatted_path);

                // Display the line of code where the error occurred.
                Self::show_error_location(files.get(&span.file).expect("Slice file not in file map!"), &span);
            }
        }

        // Output the total number of errors and warnings.
        println!();
        match counts.1 {
            0 => (),
            _ => println!(
                "{}: Compilation generated {} warning(s)",
                "Warnings".yellow().bold(),
                counts.1
            ),
        }
        match counts.0 {
            0 => println!("{}: Successfully compiled slice", "Finished".green().bold()),
            _ => println!(
                "{}: Compilation failed with {} error(s)",
                "Failed".red().bold(),
                counts.0
            ),
        }
    }

    fn show_error_location(file: &SliceFile, span: &Span) {
        // This is a safe unwrap because we know that if a diagnostic is reported with a span, then there must be
        // line of code in the file map corresponding to the error.
        let end_of_error_line = file.raw_text.lines().nth(span.end.0 - 1).unwrap().len();

        let mut start_snippet = file.get_snippet((span.start.0, 1), span.start);
        let mut error_snippet = file.get_snippet(span.start, span.end);
        let mut end_snippet = file.get_snippet(span.end, (span.end.0, end_of_error_line + 1));

        // Pop the newlines added by `get_snippet`
        start_snippet.pop();
        error_snippet.pop();
        end_snippet.pop();

        let formatted_error_lines = format!("{}{}{}", start_snippet, error_snippet, end_snippet);
        let formatted_error_lines = formatted_error_lines.split('\n').collect::<Vec<&str>>();
        let underline = "-".repeat(
            *formatted_error_lines
                .iter()
                .map(|s| s.len())
                .collect::<Vec<usize>>()
                .iter()
                .max()
                .unwrap(),
        );
        let mut line_number = span.start.0;

        // Output
        eprintln!("{}", "    |".blue().bold());
        for line in &formatted_error_lines {
            eprintln!(
                "{: <4}{} {}",
                line_number.to_string().blue().bold(),
                "|".blue().bold(),
                line
            );
            line_number += 1;
        }

        // Create the formatted error code section block.
        let blank_space = " ".repeat(start_snippet.len());
        eprintln!("{}{}{}", "    | ".blue().bold(), blank_space, underline.yellow().bold());
        eprintln!("{}", "    |".blue().bold());
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
