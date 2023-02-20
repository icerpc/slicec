// Copyright (c) ZeroC, Inc.

use crate::ast::Ast;
use crate::command_line::{DiagnosticFormat, SliceOptions};
use crate::diagnostics::Diagnostic;
use crate::grammar::{Attributable, Attribute, Entity};
use crate::slice_file::SliceFile;

use std::collections::HashMap;

#[derive(Debug)]
pub struct DiagnosticReporter {
    /// Vector where all the diagnostics are stored, in the order they're reported.
    diagnostics: Vec<Diagnostic>,
    /// The total number of errors reported.
    error_count: usize,
    /// The total number of warnings reported.
    warning_count: usize,
    /// If true, compilation will fail on warnings in addition to errors.
    treat_warnings_as_errors: bool,
    // The list of warnings to allow from the 'allow-warnings' flag.
    // Some([]) means allow all warnings.
    // Some([...]) means allow the specified warnings.
    // None means don't allow any warnings.
    pub allowed_warnings: Option<Vec<String>>,
    /// Can specify json to serialize errors as JSON or console to output errors to console.
    pub diagnostic_format: DiagnosticFormat,
    // If true, diagnostic output will not be styled.
    pub disable_color: bool,
}

impl DiagnosticReporter {
    pub fn new(slice_options: &SliceOptions) -> Self {
        DiagnosticReporter {
            diagnostics: Vec::new(),
            error_count: 0,
            warning_count: 0,
            treat_warnings_as_errors: slice_options.warn_as_error,
            diagnostic_format: slice_options.diagnostic_format,
            disable_color: slice_options.disable_color,
            allowed_warnings: slice_options.allow_warnings.clone(),
        }
    }

    /// Checks if any errors have been reported during compilation.
    pub fn has_errors(&self) -> bool {
        self.error_count != 0
    }

    /// Checks if any diagnostics (warnings or errors) have been reported during compilation.
    pub fn has_diagnostics(&self) -> bool {
        self.error_count + self.warning_count != 0
    }

    /// Returns the total number of errors and warnings reported through the diagnostic reporter.
    pub fn get_totals(&self) -> (usize, usize) {
        (self.error_count, self.warning_count)
    }

    /// Returns 1 if any errors were reported and 0 if no errors were reported.
    /// If `treat_warnings_as_errors` is true, warnings well be counted as errors by this function.
    pub fn get_exit_code(&self) -> i32 {
        i32::from(self.has_errors() || (self.treat_warnings_as_errors && self.has_diagnostics()))
    }

    /// Consumes the diagnostic reporter and returns an iterator over all its diagnostics, but with any that should
    /// be allowed filtered out (due to `allow` attributes, or the `--allow-warnings` command line
    /// option).
    pub fn into_diagnostics<'a>(
        self,
        ast: &'a Ast,
        files: &'a HashMap<String, SliceFile>,
    ) -> impl Iterator<Item = Diagnostic> + 'a {
        // Helper function that returns true if a warning should be allowed according to the provided list.
        // This happens if the list exists and is empty (ignores everything), or if the code is contained within it.
        fn is_warning_allowed_by(code: &String, allowed_codes: &Vec<String>) -> bool {
            allowed_codes.is_empty() || allowed_codes.contains(code)
        }

        // Helper function that checks whether a warning should be allowed according to the provided attributes.
        fn is_warning_allowed_by_attributes(code: &String, attributes: Vec<&Attribute>) -> bool {
            attributes
                .into_iter()
                .filter_map(Attribute::match_allow_warnings)
                .any(|allowed_codes| is_warning_allowed_by(code, &allowed_codes))
        }

        // Filter out any warnings that should be allowed.
        self.diagnostics.into_iter().filter(move |diagnostic| {
            let mut is_allowed = false;

            if let Diagnostic::Warning(warning) = &diagnostic {
                let warning_code = warning.error_code().to_owned();

                // Check if the warning is allowed by an `allowed-warnings` flag passed on the command line.
                if let Some(allowed_codes) = &self.allowed_warnings {
                    is_allowed |= is_warning_allowed_by(&warning_code, allowed_codes)
                }

                // If the warning has a span, check if it's allowed by an `allow` attribute on its file.
                if let Some(span) = &warning.span {
                    let file = files.get(&span.file).expect("slice file didn't exist");
                    is_allowed |= is_warning_allowed_by_attributes(&warning_code, file.attributes(false));
                }

                // If the warning has a scope, check if it's allowed by an `allow` attribute in that
                // scope.
                if let Some(scope) = &warning.scope {
                    let entity = ast.find_element::<dyn Entity>(scope).expect("entity didn't exist");
                    is_allowed |= is_warning_allowed_by_attributes(&warning_code, entity.attributes(true));
                }
            }
            !is_allowed
        })
    }

    pub(super) fn report(&mut self, diagnostic: impl Into<Diagnostic>) {
        let diagnostic = diagnostic.into();
        match &diagnostic {
            Diagnostic::Error(_) => self.error_count += 1,
            Diagnostic::Warning(_) => self.warning_count += 1,
        }
        self.diagnostics.push(diagnostic);
    }
}
