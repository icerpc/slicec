// Copyright (c) ZeroC, Inc.

use crate::diagnostics::{Diagnostic, Diagnostics, Error};
use crate::grammar::*;

pub fn validate_type_alias(type_alias: &TypeAlias, diagnostics: &mut Diagnostics) {
    type_aliases_cannot_be_optional(type_alias, diagnostics);
}

fn type_aliases_cannot_be_optional(type_alias: &TypeAlias, diagnostics: &mut Diagnostics) {
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
            .push_into(diagnostics)
    }
}
