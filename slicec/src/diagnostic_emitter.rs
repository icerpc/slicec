// Copyright (c) ZeroC, Inc.

use crate::diagnostics::{AnnotatedDiagnostic, DiagnosticLevel, Snippet};
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
                DiagnosticLevel::Allowed | DiagnosticLevel::Info => {}
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

    /// Emits the provided diagnostics to this emitter's output.
    pub fn emit_diagnostics(&mut self, diagnostics: &[AnnotatedDiagnostic]) -> Result<()> {
        // Select an emission function corresponding to what 'diagnostic format' the user requested.
        let emission_func = match self.diagnostic_format {
            DiagnosticFormat::Human => Self::emit_diagnostics_in_human,
            DiagnosticFormat::Json => Self::emit_diagnostics_in_json,
        };

        // Emit the diagnostics.
        // We do not emit a diagnostic if we have already emitted an exact duplicate of it.
        // Duplicates are expected when using multiple plugins which run identical validation, best to filter them out.
        let mut alread_emitted: Vec<&AnnotatedDiagnostic> = Vec::with_capacity(diagnostics.len());
        for diagnostic in diagnostics {
            if !alread_emitted.contains(&diagnostic) {
                emission_func(self, diagnostic)?;
                alread_emitted.push(diagnostic);
            }
        }

        Ok(())
    }

    fn emit_diagnostics_in_human(&mut self, diagnostic: &AnnotatedDiagnostic) -> Result<()> {
        // Style the prefix. Note that for `Notes` we do not insert a newline since they should be "attached"
        // to the previously emitted diagnostic.
        let code = &diagnostic.code;
        let prefix = match &diagnostic.level {
            DiagnosticLevel::Error => console::style(format!("error [{code}]")).red().bold(),
            DiagnosticLevel::Warning => console::style(format!("warning [{code}]")).yellow().bold(),
            DiagnosticLevel::Info => console::style("info".to_owned()).blue().bold(),
            DiagnosticLevel::Allowed => return Ok(()),
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
        Ok(())
    }

    fn emit_diagnostics_in_json(&mut self, diagnostic: &AnnotatedDiagnostic) -> Result<()> {
        let severity = match diagnostic.level {
            DiagnosticLevel::Error => "error",
            DiagnosticLevel::Warning => "warning",
            DiagnosticLevel::Info => "info",
            DiagnosticLevel::Allowed => return Ok(()),
        };

        let mut serializer = serde_json::Serializer::new(&mut *self.output);
        let mut state = serializer.serialize_struct("Diagnostic", 5)?;
        state.serialize_field("message", &diagnostic.message)?;
        state.serialize_field("severity", severity)?;
        state.serialize_field("snippet", &diagnostic.snippet)?;
        state.serialize_field("notes", &diagnostic.notes)?;
        state.serialize_field("error_code", &diagnostic.code)?;
        state.end()?;
        writeln!(self.output)?; // Each diagnostic should end with a newline character.

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
