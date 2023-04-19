// Copyright (c) ZeroC, Inc.

use crate::ast::Ast;
use crate::command_line::{DiagnosticFormat, SliceOptions};
use crate::diagnostics::{Diagnostic, DiagnosticKind, Error, Warning};
use crate::grammar::{Attributable, Attribute, Entity};
use crate::slice_file::SliceFile;
use std::collections::HashMap;
use std::str::FromStr;

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
    // A vector of all the warnings that should be suppressed by the diagnostic reporter.
    pub allowed_warnings: Vec<SuppressWarnings>,
    /// Can specify json to serialize errors as JSON or console to output errors to console.
    pub diagnostic_format: DiagnosticFormat,
    // If true, diagnostic output will not be styled.
    pub disable_color: bool,
}

impl DiagnosticReporter {
    pub fn new(slice_options: &SliceOptions) -> Self {
        let mut diagnostic_reporter = DiagnosticReporter {
            diagnostics: Vec::new(),
            error_count: 0,
            warning_count: 0,
            treat_warnings_as_errors: slice_options.warn_as_error,
            diagnostic_format: slice_options.diagnostic_format,
            disable_color: slice_options.disable_color,
            allowed_warnings: Vec::new(),
        };

        for allow_warning in &slice_options.allow_warnings {
            match SuppressWarnings::from_str(allow_warning) {
                Ok(suppress_warning) => diagnostic_reporter.allowed_warnings.push(suppress_warning),
                Err(error) => diagnostic_reporter.diagnostics.push(error),
            }
        }

        diagnostic_reporter
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

    /// Consumes the diagnostic reporter and returns an iterator over its diagnostics, with any suppressed warnings
    /// filtered out (due to either `allow` attributes, or the `--allow-warnings` command line option).
    pub fn into_diagnostics<'a>(
        self,
        ast: &'a Ast,
        files: &'a HashMap<String, SliceFile>,
    ) -> impl Iterator<Item = Diagnostic> + 'a {
        // Helper function that checks whether a warning should be allowed according to the provided attributes.
        fn is_warning_allowed_by_attributes(warning: &Warning, attributes: Vec<&Attribute>) -> bool {
            attributes
                .into_iter()
                .filter_map(Attribute::match_allow_warnings)
                .any(|allowed_warnings| {
                    allowed_warnings
                        .iter()
                        .any(|allowed_warning| allowed_warning.does_suppress(warning))
                })
        }

        // Filter out any warnings that should be allowed.
        self.diagnostics.into_iter().filter(move |diagnostic| {
            let mut is_allowed = false;

            if let DiagnosticKind::Warning(warning) = &diagnostic.kind {
                // Check if the warning is allowed by an `allowed-warnings` flag passed on the command line.
                for allowed_warning in &self.allowed_warnings {
                    is_allowed |= allowed_warning.does_suppress(warning);
                }

                // If the warning has a span, check if it's allowed by an `allow` attribute on its file.
                if let Some(span) = diagnostic.span() {
                    let file = files.get(&span.file).expect("slice file didn't exist");
                    is_allowed |= is_warning_allowed_by_attributes(warning, file.attributes(false));
                }

                // If the warning has a scope, check if it's allowed by an `allow` attribute in that
                // scope.
                if let Some(scope) = diagnostic.scope() {
                    let entity = ast.find_element::<dyn Entity>(scope).expect("entity didn't exist");
                    is_allowed |= is_warning_allowed_by_attributes(warning, entity.attributes(true));
                }
            }
            !is_allowed
        })
    }

    pub(super) fn report(&mut self, diagnostic: Diagnostic) {
        match &diagnostic.kind {
            DiagnosticKind::Error(_) => self.error_count += 1,
            DiagnosticKind::Warning(_) => self.warning_count += 1,
        }
        self.diagnostics.push(diagnostic);
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SuppressWarnings {
    Single(String),
    All,
    Deprecated,
    Comments,
}

impl SuppressWarnings {
    pub fn does_suppress(&self, warning: &Warning) -> bool {
        match self {
            Self::Single(code) => warning.error_code() == code,
            Self::All => true,
            Self::Deprecated => matches!(warning, Warning::UseOfDeprecatedEntity { .. }),
            Self::Comments => matches!(
                warning,
                Warning::DocCommentSyntax { .. }
                    | Warning::ExtraParameterInDocComment { .. }
                    | Warning::ExtraReturnValueInDocComment
                    | Warning::ExtraThrowInDocComment { .. }
                    | Warning::CouldNotResolveLink { .. }
                    | Warning::LinkToInvalidElement { .. }
                    | Warning::InvalidThrowInDocComment { .. }
                    | Warning::OperationDoesNotThrow { .. }
            ),
        }
    }
}

impl std::str::FromStr for SuppressWarnings {
    type Err = Diagnostic;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Deprecated" => Ok(SuppressWarnings::Deprecated),
            "Comments" => Ok(SuppressWarnings::Comments),
            "All" => Ok(SuppressWarnings::All),
            code => {
                if Warning::all_codes().contains(&code) {
                    Ok(SuppressWarnings::Single(code.to_owned()))
                } else {
                    let error = Diagnostic::new(Error::ArgumentNotSupported {
                        argument: code.to_owned(),
                        directive: "allow".to_owned(),
                    })
                    .add_note(
                        "warnings can be specified as a category or a code of the form 'W###'.",
                        None,
                    )
                    .add_note("valid categories are: 'Deprecated', 'Comments', and 'All'", None);
                    Err(error)
                }
            }
        }
    }
}
