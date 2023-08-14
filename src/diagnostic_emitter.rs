// Copyright (c) ZeroC, Inc.

use crate::diagnostics::{Diagnostic, DiagnosticLevel};
use crate::slice_file::{SliceFile, Span};
use crate::slice_options::{DiagnosticFormat, SliceOptions};
use serde::ser::SerializeStruct;
use serde::Serializer;
use std::collections::HashMap;
use std::io::{Result, Write};
use std::path::Path;

#[derive(Debug)]
pub struct DiagnosticEmitter<'a, T: Write> {
    /// Reference to the output that diagnostics should be emitted to.
    output: &'a mut T,
    /// Can specify `json` to serialize errors as JSON or `human` to pretty-print them.
    diagnostic_format: DiagnosticFormat,
    /// If true, diagnostic output will not be styled with colors (only used in `human` format).
    disable_color: bool,
    /// Provides the emitter access to the slice files that were compiled so it can extract snippets from them.
    files: &'a HashMap<String, SliceFile>,
}

impl<'a, T: Write> DiagnosticEmitter<'a, T> {
    pub fn new(output: &'a mut T, slice_options: &SliceOptions, files: &'a HashMap<String, SliceFile>) -> Self {
        DiagnosticEmitter {
            output,
            diagnostic_format: slice_options.diagnostic_format,
            disable_color: slice_options.disable_color,
            files,
        }
    }

    pub fn emit_diagnostics(&mut self, diagnostics: Vec<Diagnostic>) -> Result<()> {
        // Disable colors if the user requested no colors.
        if self.disable_color {
            console::set_colors_enabled(false);
            console::set_colors_enabled_stderr(false);
        }

        // Emit the diagnostics in whatever form the user requested.
        match self.diagnostic_format {
            DiagnosticFormat::Human => self.emit_diagnostics_in_human(diagnostics),
            DiagnosticFormat::Json => self.emit_diagnostics_in_json(diagnostics),
        }
    }

    fn emit_diagnostics_in_human(&mut self, diagnostics: Vec<Diagnostic>) -> Result<()> {
        for diagnostic in diagnostics {
            // Style the prefix. Note that for `Notes` we do not insert a newline since they should be "attached"
            // to the previously emitted diagnostic.
            let code = diagnostic.code();
            let prefix = match diagnostic.level() {
                DiagnosticLevel::Error => console::style(format!("error [{code}]")).red().bold(),
                DiagnosticLevel::Warning => console::style(format!("warning [{code}]")).yellow().bold(),
                DiagnosticLevel::Allowed => continue,
            };

            // Emit the message with the prefix.
            writeln!(self.output, "{prefix}: {}", console::style(diagnostic.message()).bold())?;

            // If the diagnostic contains a span, show a snippet containing the offending code.
            if let Some(span) = diagnostic.span() {
                self.emit_snippet(span)?;
            }

            // If the diagnostic contains notes, display them.
            for note in diagnostic.notes() {
                writeln!(
                    self.output,
                    "{}: {}",
                    console::style("note").blue().bold(),
                    console::style(&note.message).bold(),
                )?;

                if let Some(span) = &note.span {
                    self.emit_snippet(span)?;
                }
            }
        }
        Ok(())
    }

    fn emit_diagnostics_in_json(&mut self, diagnostics: Vec<Diagnostic>) -> Result<()> {
        // Write each diagnostic as a single line of JSON.
        for diagnostic in diagnostics {
            let severity = match diagnostic.level() {
                DiagnosticLevel::Error => "error",
                DiagnosticLevel::Warning => "warning",
                DiagnosticLevel::Allowed => continue,
            };

            let mut serializer = serde_json::Serializer::new(&mut *self.output);
            let mut state = serializer.serialize_struct("Diagnostic", 5)?;
            state.serialize_field("message", &diagnostic.message())?;
            state.serialize_field("severity", severity)?;
            state.serialize_field("span", &diagnostic.span())?;
            state.serialize_field("notes", diagnostic.notes())?;
            state.serialize_field("error_code", diagnostic.code())?;
            state.end()?;
            writeln!(self.output)?; // Separate each diagnostic by a newline character.
        }
        Ok(())
    }

    fn emit_snippet(&mut self, span: &Span) -> Result<()> {
        // Display the file name and line row and column where the error began.
        writeln!(
            self.output,
            " {} {}:{}:{}",
            console::style("-->").blue().bold(),
            Path::new(&span.file).display(),
            span.start.row,
            span.start.col,
        )?;

        // Display the line of code where the error occurred.
        let snippet = self.files.get(&span.file).unwrap().get_snippet(span.start, span.end);
        writeln!(self.output, "{}", snippet)?;

        Ok(())
    }
}

pub fn emit_totals(total_warnings: usize, total_errors: usize) -> Result<()> {
    // Totals are always printed to stdout.
    let stdout = &mut console::Term::stdout();

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
