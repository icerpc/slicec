// Copyright (c) ZeroC, Inc.

use crate::ast::Ast;
use crate::diagnostics::{Diagnostic, DiagnosticReporter, Error};
use crate::grammar::*;
use std::collections::HashMap;

pub fn validate_inherited_identifiers(
    symbols: Vec<&impl NamedSymbol>,
    inherited_symbols: Vec<&impl NamedSymbol>,
    diagnostic_reporter: &mut DiagnosticReporter,
) {
    check_for_shadowing(symbols, inherited_symbols, diagnostic_reporter);
}

fn check_for_shadowing(
    symbols: Vec<&impl NamedSymbol>,
    inherited_symbols: Vec<&impl NamedSymbol>,
    diagnostic_reporter: &mut DiagnosticReporter,
) {
    let identifiers = symbols.into_iter().map(NamedSymbol::raw_identifier);
    let inherited_identifiers = inherited_symbols
        .into_iter()
        .map(NamedSymbol::raw_identifier)
        .collect::<Vec<_>>();

    for identifier in identifiers {
        for inherited_identifier in &inherited_identifiers {
            if identifier.value == inherited_identifier.value {
                Diagnostic::new(Error::Shadows {
                    identifier: identifier.value.clone(),
                })
                .set_span(identifier.span())
                .add_note(
                    format!("'{}' was previously defined here", inherited_identifier.value),
                    Some(inherited_identifier.span()),
                )
                .report(diagnostic_reporter);
            }
        }
    }
}

pub fn check_for_redefinitions(ast: &Ast, diagnostic_reporter: &mut DiagnosticReporter) {
    // A map storing `(scoped_identifier, named_symbol)` pairs. We iterate through the AST and build up this map,
    // and if we try to add an entry but see it's already occupied, that means we've found a redefinition.
    let mut slice_definitions: HashMap<String, &dyn NamedSymbol> = HashMap::new();

    for node in ast.as_slice() {
        // Only check nodes that have identifiers, everything else is irrelevant.
        if let Ok(definition) = <&dyn NamedSymbol>::try_from(node) {
            let scoped_identifier = definition.parser_scoped_identifier();
            match slice_definitions.get(&scoped_identifier) {
                // If we've already seen a node with this identifier, there's a name collision.
                // This is fine for modules (since they can be re-opened), but for any other type, we report an error.
                Some(other_definition) => {
                    if !(is_module(definition) && is_module(*other_definition)) {
                        report_redefinition_error(definition, *other_definition, diagnostic_reporter);
                    }
                }

                // If we haven't seen a node with this identifier before, add it to the map and continue checking.
                None => {
                    slice_definitions.insert(scoped_identifier, definition);
                }
            }
        }
    }
}

// TODO maybe add an 'Elements' wrapper to avoid these string comparisons?
fn is_module(definition: &dyn NamedSymbol) -> bool {
    definition.kind() == "module"
}

fn report_redefinition_error(new: &dyn NamedSymbol, original: &dyn NamedSymbol, reporter: &mut DiagnosticReporter) {
    Diagnostic::new(Error::Redefinition {
        identifier: new.identifier().to_owned(),
    })
    .set_span(new.raw_identifier().span())
    .add_note(
        format!("'{}' was previously defined here", original.identifier()),
        Some(original.raw_identifier().span()),
    )
    .report(reporter);
}
