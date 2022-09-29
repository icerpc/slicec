// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::ast::Ast;
use crate::command_line::DiagnosticFormat;
use crate::diagnostics::*;
use crate::slice_file::{SliceFile, Span};
use console::{strip_ansi_codes, style};
use std::collections::HashMap;
use std::io::{stderr, Write};

#[derive(Debug)]
pub struct ParsedData {
    pub ast: Ast,
    pub diagnostic_reporter: DiagnosticReporter,
    pub files: HashMap<String, SliceFile>,
}

impl ParsedData {
    pub fn into_exit_code(self) -> i32 {
        // Emit any diagnostics that were reported.
        let has_errors = self.has_errors();
        self.emit_diagnostics(None);

        i32::from(has_errors)
    }

    pub fn has_errors(&self) -> bool {
        self.diagnostic_reporter.has_errors()
    }

    pub fn emit_diagnostics(self, writer: Option<&mut dyn Write>) {
        let mut stderr = stderr();
        match self.diagnostic_reporter.diagnostic_format {
            DiagnosticFormat::Human => self.output_to_console(writer.unwrap_or(&mut stderr)),
            DiagnosticFormat::Json => self.output_to_json(writer.unwrap_or(&mut stderr)),
        }
        .expect("Failed to write diagnostic output to writer");
    }

    fn output_to_json(self, writer: &mut dyn Write) -> std::io::Result<()> {
        // The for loop consumes the diagnostics, so we need to take ownership of disable color and counts.
        let disable_color = self.diagnostic_reporter.disable_color;
        let counts = self.diagnostic_reporter.get_totals();

        // Write each diagnostic as a single line of JSON.
        for diagnostic in self.diagnostic_reporter.into_diagnostics() {
            let json = serde_json::to_string(&diagnostic).expect("Failed to serialize diagnostic to JSON");
            writeln!(writer, "{json}")?;
        }

        Self::output_status(counts, disable_color);
        Ok(())
    }

    fn output_to_console(self, writer: &mut dyn Write) -> std::io::Result<()> {
        // Take ownership of the files from `self`
        let files = self.files;

        // The for loop consumes the diagnostics, so we need to take ownership of disable color and counts.
        let disable_color = self.diagnostic_reporter.disable_color;
        let counts = self.diagnostic_reporter.get_totals();

        for diagnostic in self.diagnostic_reporter.into_diagnostics() {
            // Style the prefix. Note that for `Notes` we do not insert a newline since they should be "attached"
            // to the previously emitted diagnostic.
            let error_code = diagnostic
                .error_code()
                .map_or_else(String::new, |code| format!(" [{code}]"));
            let prefix = match diagnostic {
                Diagnostic::Error(_) => style("error".to_owned() + &error_code).red().bold(),
                Diagnostic::Warning(_) => style("warning".to_owned() + &error_code).yellow().bold(),
            };
            let mut message = vec![];

            // Emit the message with the prefix.
            message.push(format!("{}: {}", prefix, style(&diagnostic).bold()));

            // If the diagnostic contains a span, show a snippet containing the offending code.
            if let Some(span) = diagnostic.span() {
                Self::append_snippet(&mut message, span, &files);
            }

            // If the diagnostic contains notes, display them.
            diagnostic.notes().iter().for_each(|note| {
                message.push(format!(
                    "    {} {}: {:}",
                    style("=").blue().bold(),
                    style("note").bold(),
                    style(&note).bold(),
                ));
                if let Some(span) = &note.span {
                    Self::append_snippet(&mut message, span, &files)
                }
            });

            let mut output_message = message.join("\n");
            if disable_color {
                output_message = strip_ansi_codes(&output_message).to_string();
            }

            writeln!(writer, "{}", output_message)?;
        }
        Self::output_status(counts, disable_color);
        Ok(())
    }

    // Output the total number of errors and warnings.
    fn output_status(counts: (usize, usize), disable_color: bool) {
        let mut counter_messages = vec![];
        if counts.1 != 0 {
            counter_messages.push(format!(
                "{}: Compilation generated {} warning(s)",
                style("Warnings").yellow().bold(),
                counts.1,
            ));
        }
        if counts.0 != 0 {
            counter_messages.push(format!(
                "{}: Compilation failed with {} error(s)",
                style("Failed").red().bold(),
                counts.0,
            ));
        }
        let mut output_message = counter_messages.join("\n");
        if disable_color {
            output_message = strip_ansi_codes(&output_message).to_string();
        }

        println!();
        println!("{output_message}");
    }

    fn append_snippet(message: &mut Vec<String>, span: &Span, files: &HashMap<String, SliceFile>) {
        // Display the file name and line row and column where the error began.
        let file_location = format!("{}:{}:{}", &span.file, span.start.row, span.start.col);
        let path = std::path::Path::new(&file_location);
        message.push(format!(" {} {}", style("-->").blue().bold(), path.display()));

        // Display the line of code where the error occurred.
        let snippet = files.get(&span.file).unwrap().get_snippet(span.start, span.end);
        message.push(snippet);
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
