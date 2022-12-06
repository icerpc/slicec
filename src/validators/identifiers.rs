// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::diagnostics::*;
use crate::grammar::*;
use crate::validators::{ValidationChain, Validator};

pub fn identifier_validators() -> ValidationChain {
    vec![
        Validator::Identifiers(check_for_redefinition),
        Validator::InheritedIdentifiers(check_for_shadowing),
    ]
}

pub fn check_for_redefinition(mut identifiers: Vec<&Identifier>, diagnostic_reporter: &mut DiagnosticReporter) {
    // Sort first so that we can use windows to search for duplicates.
    identifiers.sort_by_key(|identifier| identifier.value.to_owned());
    identifiers.windows(2).for_each(|window| {
        if window[0].value == window[1].value {
            Error::new(ErrorKind::Redefinition(window[1].value.clone()))
                .set_span(window[1].span())
                .add_note(
                    format!("`{}` was previously defined here", window[0].value),
                    Some(window[0].span()),
                )
                .report(diagnostic_reporter);
        }
    });
}

pub fn check_for_shadowing(
    identifiers: Vec<&Identifier>,
    inherited_symbols: Vec<&Identifier>,
    diagnostic_reporter: &mut DiagnosticReporter,
) {
    identifiers.iter().for_each(|identifier| {
        inherited_symbols
            .iter()
            .filter(|inherited_identifier| inherited_identifier.value == identifier.value)
            .for_each(|inherited_identifier| {
                Error::new(ErrorKind::Shadows(identifier.value.clone()))
                    .set_span(identifier.span())
                    .add_note(
                        format!("`{}` was previously defined here", inherited_identifier.value),
                        Some(inherited_identifier.span()),
                    )
                    .report(diagnostic_reporter);
            });
    });
}

trait EntityIdentifiersExtension {
    fn get_identifiers(&self) -> Vec<&Identifier>;
}

impl<T> EntityIdentifiersExtension for Vec<&T>
where
    T: Entity,
{
    fn get_identifiers(&self) -> Vec<&Identifier> {
        self.iter().map(|member| member.raw_identifier()).collect()
    }
}
