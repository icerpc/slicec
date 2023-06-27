// Copyright (c) ZeroC, Inc.

use super::{Diagnostic, DiagnosticKind, DiagnosticLevel, Lint};
use crate::ast::Ast;
use crate::compilation_state::CompilationState;
use crate::grammar::{attributes, Attributable, Entity};
use crate::slice_file::{SliceFile, Span};
use crate::slice_options::{DiagnosticFormat, SliceOptions};
use serde::ser::SerializeStruct;
use serde::Serializer;
use std::collections::HashMap;
use std::io::{Result, Write};

#[derive(Debug)]
pub struct DiagnosticReporter {
    /// Vector where all the diagnostics are stored, in the order they're reported.
    diagnostics: Vec<Diagnostic>,
    /// Lists all the lints that should be allowed by this reporter.
    allowed_lints: Vec<String>,
    /// Can specify json to serialize errors as JSON or console to output errors to console.
    diagnostic_format: DiagnosticFormat,
    /// If true, diagnostic output will not be styled with colors.
    disable_color: bool,
}

impl DiagnosticReporter {
    pub fn new(slice_options: &SliceOptions) -> Self {
        DiagnosticReporter {
            diagnostics: Vec::new(),
            diagnostic_format: slice_options.diagnostic_format,
            disable_color: slice_options.disable_color,
            allowed_lints: slice_options.allowed_lints.clone(),
        }
    }

    /// Checks if any errors have been reported during compilation.
    pub fn has_errors(&self) -> bool {
        let mut diagnostics = self.diagnostics.iter();
        diagnostics.any(|diagnostic| matches!(diagnostic.kind, DiagnosticKind::Error(_)))
    }

    pub(super) fn report(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }

    /// The final step of the compiler's execution.
    /// This consumes the entire `CompilationState`, emits any diagnostics that shouldn't be suppressed,
    /// and finally returns the exit code the compiler should terminate with.
    pub fn emit_diagnostics_and_get_exit_code(compilation_state: CompilationState, output: &mut impl Write) -> i32 {
        // TODO this should really be on one line.
        // Destructure the compilation state into it's components.
        let CompilationState {
            mut diagnostic_reporter,
            ast,
            files,
        } = compilation_state;

        // Update the diagnostic levels of all diagnostics based on any attributes or command line options.
        let (total_warnings, total_errors) = diagnostic_reporter.update_diagnostics(&ast, &files);

        // Print the diagnostics to the console, along with the total number of warnings and errors emitted.
        DiagnosticReporter::emit_diagnostics(diagnostic_reporter, &files, output).expect("failed to print diagnostics");
        DiagnosticReporter::emit_totals(total_warnings, total_errors).expect("failed to print totals");

        // Return exit code 1 if any errors were reported, and exit code 0 otherwise.
        i32::from(total_errors != 0)
    }

    // TODO COMMENT
    // TODO add support for deny/warn lint configuration attributes & command line options.
    fn update_diagnostics(&mut self, ast: &Ast, files: &HashMap<String, SliceFile>) -> (usize, usize) {
        // Helper function that checks whether a lint should be allowed according to the provided identifiers.
        fn is_lint_allowed_by<'b>(mut identifiers: impl Iterator<Item = &'b String>, lint: &Lint) -> bool {
            identifiers.any(|identifier| identifier == "All" || identifier == lint.code())
        }

        // Helper function that checks whether a lint is allowed by attributes on the provided entity.
        fn is_lint_allowed_by_attributes(attributable: &(impl Attributable + ?Sized), lint: &Lint) -> bool {
            let attributes = attributable.all_attributes().concat().into_iter();
            let mut allowed = attributes.filter_map(|a| a.downcast::<attributes::Allow>());
            allowed.any(|allow| is_lint_allowed_by(allow.allowed_lints.iter(), lint))
        }

        // TODO COMMENT
        let (mut total_warnings, mut total_errors) = (0, 0);
        for diagnostic in &mut self.diagnostics {
            // If this diagnostic is a lint, update its diagnostic level. Errors always have a level of `Error`.
            if let DiagnosticKind::Lint(lint) = &diagnostic.kind {
                // Check if the lint is allowed by an `--allow` flag passed on the command line.
                if is_lint_allowed_by(self.allowed_lints.iter(), lint) {
                    diagnostic.level = DiagnosticLevel::Allowed;
                }

                // If the diagnostic has a span, check if it's affected by an `allow` attribute on its file.
                if let Some(span) = diagnostic.span() {
                    let file = files.get(&span.file).expect("slice file didn't exist");
                    if is_lint_allowed_by_attributes(file, lint) {
                        diagnostic.level = DiagnosticLevel::Allowed;
                    }
                }

                // If the diagnostic has a scope, check if it's affected by an `allow` attribute in that scope.
                if let Some(scope) = diagnostic.scope() {
                    if let Ok(entity) = ast.find_element::<dyn Entity>(scope) {
                        if is_lint_allowed_by_attributes(entity, lint) {
                            diagnostic.level = DiagnosticLevel::Allowed;
                        }
                    }
                }
            }

            // Update the total number of errors/warnings accordingly.
            match diagnostic.level() {
                DiagnosticLevel::Error => total_errors += 1,
                DiagnosticLevel::Warning => total_warnings += 1,
                DiagnosticLevel::Allowed => {}
            }
        }

        (total_warnings, total_errors)
    }

    /// Consumes the diagnostic reporter and writes any non-allowed diagnostics to the provided output.
    fn emit_diagnostics(self, files: &HashMap<String, SliceFile>, output: &mut impl Write) -> Result<()> {
        // Disable colors if the user requested no colors.
        if self.disable_color {
            console::set_colors_enabled(false);
            console::set_colors_enabled_stderr(false);
        }

        match self.diagnostic_format {
            DiagnosticFormat::Human => self.emit_diagnostics_in_human(output, files),
            DiagnosticFormat::Json => self.emit_diagnostics_in_json(output),
        }
    }

    /// Prints the total number of warnings and errors to stdout.
    /// These messages are conditionally printed; if there were no warnings or errors we don't print them.
    fn emit_totals(total_warnings: usize, total_errors: usize) -> Result<()> {
        let mut stdout = std::io::stdout();

        if total_warnings > 0 {
            let warnings = console::style("Warnings").yellow().bold();
            writeln!(stdout, "{warnings}: Compilation generated {total_warnings} warning(s)")?;
        }
        if total_errors > 0 {
            let failed = console::style("Failed").red().bold();
            writeln!(stdout, "{failed}: Compilation failed with {total_errors} error(s)")?;
        }
        Ok(())
    }

    fn emit_diagnostics_in_human(self, output: &mut impl Write, files: &HashMap<String, SliceFile>) -> Result<()> {
        fn append_snippet(message: &mut Vec<String>, span: &Span, files: &HashMap<String, SliceFile>) {
            // Display the file name and line row and column where the error began.
            let file_location = format!("{}:{}:{}", &span.file, span.start.row, span.start.col);
            let path = std::path::Path::new(&file_location);
            message.push(format!(" {} {}", console::style("-->").blue().bold(), path.display()));

            // Display the line of code where the error occurred.
            let snippet = files.get(&span.file).unwrap().get_snippet(span.start, span.end);
            message.push(snippet);
        }

        for diagnostic in self.diagnostics {
            // Style the prefix. Note that for `Notes` we do not insert a newline since they should be "attached"
            // to the previously emitted diagnostic.
            let code = diagnostic.code();
            let prefix = match diagnostic.level() {
                DiagnosticLevel::Error => console::style(format!("error [{code}]")).red().bold(),
                DiagnosticLevel::Warning => console::style(format!("warning [{code}]")).yellow().bold(),
                DiagnosticLevel::Allowed => continue,
            };

            let mut message = vec![];

            // Emit the message with the prefix.
            message.push(format!("{prefix}: {}", console::style(diagnostic.message()).bold()));

            // If the diagnostic contains a span, show a snippet containing the offending code.
            if let Some(span) = diagnostic.span() {
                append_snippet(&mut message, span, files);
            }

            // If the diagnostic contains notes, display them.
            for note in diagnostic.notes() {
                message.push(format!(
                    "    {} {}: {:}",
                    console::style("=").blue().bold(),
                    console::style("note").bold(),
                    console::style(&note.message).bold(),
                ));
                // Only display the snippet if the note has a different span than the diagnostic.
                if note.span.as_ref() != diagnostic.span() {
                    if let Some(span) = &note.span {
                        append_snippet(&mut message, span, files)
                    }
                }
            }
            writeln!(output, "{}", message.join("\n"))?;
        }
        Ok(())
    }

    fn emit_diagnostics_in_json(self, output: &mut impl Write) -> Result<()> {
        // Write each diagnostic as a single line of JSON.
        for diagnostic in self.diagnostics {
            let severity = match diagnostic.level() {
                DiagnosticLevel::Error => "error",
                DiagnosticLevel::Warning => "warning",
                DiagnosticLevel::Allowed => continue,
            };

            let mut serializer = serde_json::Serializer::new(&mut *output);
            let mut state = serializer.serialize_struct("Diagnostic", 5)?;
            state.serialize_field("message", &diagnostic.message())?;
            state.serialize_field("severity", severity)?;
            state.serialize_field("span", &diagnostic.span())?;
            state.serialize_field("notes", diagnostic.notes())?;
            state.serialize_field("error_code", diagnostic.code())?;
            state.end()?;
            writeln!(output)?; // Separate each diagnostic by a newline character.
        }
        Ok(())
    }

    /// Updates and returns all the diagnostics reported during compilation.
    /// This method exists to simplify the testing of diagnostic emission.
    pub fn into_diagnostics(compilation_state: CompilationState) -> Vec<Diagnostic> {
        // TODO this should really be on one line.
        let CompilationState {
            mut diagnostic_reporter,
            ast,
            files,
        } = compilation_state;

        // Update and return all the diagnostics.
        diagnostic_reporter.update_diagnostics(&ast, &files);
        diagnostic_reporter.diagnostics
    }
}
