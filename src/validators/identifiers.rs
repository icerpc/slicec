// Copyright (c) ZeroC, Inc.

use crate::diagnostics::{Diagnostic, DiagnosticReporter, Error};
use crate::grammar::*;

pub fn validate_identifiers(named_symbols: Vec<&impl NamedSymbol>, diagnostic_reporter: &mut DiagnosticReporter) {
    check_for_redefinition(named_symbols, diagnostic_reporter);
}

pub fn validate_inherited_identifiers(
    symbols: Vec<&impl NamedSymbol>,
    inherited_symbols: Vec<&impl NamedSymbol>,
    diagnostic_reporter: &mut DiagnosticReporter,
) {
    check_for_shadowing(symbols, inherited_symbols, diagnostic_reporter);
}

fn check_for_redefinition(mut symbols: Vec<&impl NamedSymbol>, diagnostic_reporter: &mut DiagnosticReporter) {
    // Sort first so that we can use windows to search for duplicates.
    let mut identifiers = symbols
        .drain(..)
        .map(|symbol| symbol.raw_identifier())
        .collect::<Vec<_>>();

    identifiers.sort_by_key(|identifier| identifier.value.to_owned());
    identifiers.windows(2).for_each(|window| {
        if window[0].value == window[1].value {
            Diagnostic::new(Error::Redefinition {
                identifier: window[1].value.clone(),
            })
            .set_span(window[1].span())
            .add_note(
                format!("'{}' was previously defined here", window[0].value),
                Some(window[0].span()),
            )
            .report(diagnostic_reporter);
        }
    });
}

fn check_for_shadowing(
    mut symbols: Vec<&impl NamedSymbol>,
    mut inherited_symbols: Vec<&impl NamedSymbol>,
    diagnostic_reporter: &mut DiagnosticReporter,
) {
    let identifiers = symbols
        .drain(..)
        .map(|symbol| symbol.raw_identifier())
        .collect::<Vec<_>>();

    let inherited_identifiers = inherited_symbols
        .drain(..)
        .map(|symbol| symbol.raw_identifier())
        .collect::<Vec<_>>();

    identifiers.into_iter().for_each(|identifier| {
        inherited_identifiers
            .iter()
            .filter(|inherited_identifier| inherited_identifier.value == identifier.value)
            .for_each(|inherited_identifier| {
                Diagnostic::new(Error::Shadows {
                    identifier: identifier.value.clone(),
                })
                .set_span(identifier.span())
                .add_note(
                    format!("'{}' was previously defined here", inherited_identifier.value),
                    Some(inherited_identifier.span()),
                )
                .report(diagnostic_reporter);
            });
    });
}
