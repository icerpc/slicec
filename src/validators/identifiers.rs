// Copyright (c) ZeroC, Inc. All rights reserved.

use super::ValidatorVisitor;
use crate::diagnostics::{Error, ErrorKind};
use crate::grammar::{Entity, Identifier, Symbol};

impl ValidatorVisitor<'_> {
pub(super) fn check_for_redefinition(&mut self, mut identifiers: Vec<&Identifier>) {
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
                .report(self.diagnostic_reporter);
        }
    });
}

pub(super) fn check_for_shadowing(&mut self, identifiers: Vec<&Identifier>, inherited_symbols: Vec<&Identifier>) {
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
                    .report(self.diagnostic_reporter);
            });
    });
}
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
