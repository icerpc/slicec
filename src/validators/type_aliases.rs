// Copyright (c) ZeroC, Inc.

use crate::diagnostics::{Diagnostic, DiagnosticReporter, Error};
use crate::grammar::*;

pub fn validate_type_alias(type_alias: &TypeAlias, diagnostic_reporter: &mut DiagnosticReporter) {
    type_aliases_cannot_be_optional(type_alias, diagnostic_reporter);
}

fn type_aliases_cannot_be_optional(type_alias: &TypeAlias, diagnostic_reporter: &mut DiagnosticReporter) {
    if type_alias.underlying.is_optional {
        Diagnostic::new(Error::TypeAliasOfOptional)
            .set_span(type_alias.span())
            .add_note(
                "try removing the trailing `?` modifier from its definition",
                Some(type_alias.underlying.span()),
            )
            .add_note(
                "instead of aliasing an optional type directly, try making it optional where you use it",
                None,
            )
            .report(diagnostic_reporter)
    }
}
