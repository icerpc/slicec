// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::ast::Ast;
use crate::command_line::{DiagnosticFormat, SliceOptions};
use crate::diagnostics::*;
use crate::slice_file::{SliceFile, Span};
use console::{set_colors_enabled, set_colors_enabled_stderr, style, Term};
use std::collections::HashMap;
use std::io::Write;

#[derive(Debug)]
pub struct CompilationData {
    pub ast: Ast,
    pub diagnostic_reporter: DiagnosticReporter,
    pub files: HashMap<String, SliceFile>,
}

impl CompilationData {
    pub fn create(options: &SliceOptions) -> Self {
        CompilationData {
            ast: Ast::create(),
            diagnostic_reporter: DiagnosticReporter::new(options),
            files: HashMap::new(),
        }
    }

    pub fn into_exit_code(self) -> i32 {
        // If there are any errors, return a non-zero exit code.
        let exit_code = self.diagnostic_reporter.get_exit_code();

        // If any diagnostics were reported, emit them.
        if self.diagnostic_reporter.has_diagnostics() {
            self.emit_diagnostics(&mut Term::stderr());
        }

        exit_code
    }

    pub fn emit_diagnostics(self, writer: &mut impl Write) {
        // Disable colors if the user requested no colors.
        if self.diagnostic_reporter.disable_color {
            set_colors_enabled(false);
            set_colors_enabled_stderr(false);
        }

        match self.diagnostic_reporter.diagnostic_format {
            DiagnosticFormat::Human => self.output_to_console(writer),
            DiagnosticFormat::Json => self.output_to_json(writer),
        }
        .expect("Failed to write diagnostic output to writer");
    }

    fn output_to_json(self, writer: &mut impl Write) -> std::io::Result<()> {
        // The for loop consumes the diagnostics, so we compute the count now.
        let counts = self.diagnostic_reporter.get_totals();

        // Write each diagnostic as a single line of JSON.
        for diagnostic in self.diagnostic_reporter.into_diagnostics() {
            let json = serde_json::to_string(&diagnostic).expect("Failed to serialize diagnostic to JSON");
            writeln!(writer, "{json}")?;
        }
        Self::output_counts(counts)
    }

    fn output_to_console(self, writer: &mut impl Write) -> std::io::Result<()> {
        // Take ownership of the files from `self`
        let files = self.files;

        // The for loop consumes the diagnostics, so we compute the count now.
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
            message.push(format!("{prefix}: {}", style(diagnostic.message()).bold()));

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
                    style(&note.message).bold(),
                ));
                // Only display the snippet if the note has a different span than the diagnostic.
                if note.span.as_ref() != diagnostic.span() {
                    if let Some(span) = &note.span {
                        Self::append_snippet(&mut message, span, &files)
                    }
                }
            });
            writeln!(writer, "{}", message.join("\n"))?;
        }
        Self::output_counts(counts)
    }

    // Output the total number of errors and warnings.
    fn output_counts(counts: (usize, usize)) -> std::io::Result<()> {
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
        writeln!(Term::stdout(), "\n{}", counter_messages.join("\n"))
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

impl From<CompilationData> for CompilationResult {
    fn from(compilation_data: CompilationData) -> Self {
        match compilation_data.diagnostic_reporter.has_errors() {
            false => Ok(compilation_data),
            true => Err(compilation_data),
        }
    }
}

pub type CompilationResult = Result<CompilationData, CompilationData>;
