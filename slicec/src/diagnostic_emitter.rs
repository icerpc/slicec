// Copyright (c) ZeroC, Inc.

use crate::diagnostics::{DiagnosticLevel, AnnotatedDiagnostic, Snippet};
use crate::slice_options::{DiagnosticFormat, SliceOptions};
use serde::ser::SerializeStruct;
use serde::Serializer;
use std::io::{Result, Write};
use std::path::Path;

pub struct DiagnosticEmitter<'a> {
    /// Reference to the output that diagnostics should be emitted to.
    output: &'a mut dyn Write,
    /// Can specify `json` to serialize errors as JSON or `human` to pretty-print them.
    diagnostic_format: DiagnosticFormat,
}

impl<'a> DiagnosticEmitter<'a> {
    pub fn new(output: &'a mut dyn Write, slice_options: &SliceOptions) -> Self {
        DiagnosticEmitter {
            output,
            diagnostic_format: slice_options.diagnostic_format,
        }
    }

    pub fn get_totals(diagnostics: &[AnnotatedDiagnostic]) -> (usize, usize) {
        let (mut total_warnings, mut total_errors) = (0, 0);

        for diagnostic in diagnostics {
            match diagnostic.level {
                DiagnosticLevel::Error => total_errors += 1,
                DiagnosticLevel::Warning => total_warnings += 1,
                DiagnosticLevel::Allowed => {}
            }
        }

        (total_warnings, total_errors)
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

    pub fn emit_diagnostics(&mut self, diagnostics: &[AnnotatedDiagnostic]) -> Result<()> {
        // Emit the diagnostics in whatever form the user requested.
        match self.diagnostic_format {
            DiagnosticFormat::Human => self.emit_diagnostics_in_human(diagnostics),
            DiagnosticFormat::Json => self.emit_diagnostics_in_json(diagnostics),
        }
    }

    fn emit_diagnostics_in_human(&mut self, diagnostics: &[AnnotatedDiagnostic]) -> Result<()> {
        for diagnostic in diagnostics {
            // Style the prefix. Note that for `Notes` we do not insert a newline since they should be "attached"
            // to the previously emitted diagnostic.
            let code = &diagnostic.code;
            let prefix = match &diagnostic.level {
                DiagnosticLevel::Error => console::style(format!("error [{code}]")).red().bold(),
                DiagnosticLevel::Warning => console::style(format!("warning [{code}]")).yellow().bold(),
                DiagnosticLevel::Allowed => continue,
            };

            // Emit the message with the prefix.
            writeln!(self.output, "{prefix}: {}", console::style(&diagnostic.message).bold())?;

            // If the diagnostic contains a snippet of the offending code, display it.
            if let Some(snippet) = &diagnostic.snippet {
                self.emit_snippet(snippet)?;
            }

            // If the diagnostic contains notes, display them.
            for note in &diagnostic.notes {
                writeln!(
                    self.output,
                    "{}: {}",
                    console::style("note").blue().bold(),
                    console::style(&note.message).bold(),
                )?;

                if let Some(snippet) = &note.snippet {
                    self.emit_snippet(snippet)?;
                }
            }
        }
        Ok(())
    }

    fn emit_diagnostics_in_json(&mut self, diagnostics: &[AnnotatedDiagnostic]) -> Result<()> {
        // Write each diagnostic as a single line of JSON.
        for diagnostic in diagnostics {
            let severity = match diagnostic.level {
                DiagnosticLevel::Error => "error",
                DiagnosticLevel::Warning => "warning",
                DiagnosticLevel::Allowed => continue,
            };

            let mut serializer = serde_json::Serializer::new(&mut *self.output);
            let mut state = serializer.serialize_struct("Diagnostic", 5)?;
            state.serialize_field("message", &diagnostic.message)?;
            state.serialize_field("severity", severity)?;
            state.serialize_field("snippet", &diagnostic.snippet)?;
            state.serialize_field("notes", &diagnostic.notes)?;
            state.serialize_field("error_code", &diagnostic.code)?;
            state.end()?;
            writeln!(self.output)?; // Separate each diagnostic by a newline character.
        }
        Ok(())
    }

    fn emit_snippet(&mut self, snippet: &Snippet) -> Result<()> {
        // Display the file name and line row and column where the error began.
        writeln!(
            self.output,
            " {} {}:{}:{}",
            console::style("-->").blue().bold(),
            Path::new(&snippet.span.file).display(),
            snippet.span.start.row,
            snippet.span.start.col,
        )?;

        // Display the line of code where the error occurred.
        writeln!(self.output, "{}", snippet.text)?;

        Ok(())
    }
}
