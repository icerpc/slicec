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
    // A map storing `(scoped_identifier, entity)` pairs. We iterate through the AST and build up this map, and if we
    // try to add an entry but see it's already occupied, that means we've found a redefinition.
    let mut slice_definitions: HashMap<String, &dyn Entity> = HashMap::new();

    // Iterate through all the nodes in the AST.
    for node in ast.as_slice() {
        // Only check the node if its an entity, since only entities have identifiers.
        if let Ok(entity) = <&dyn Entity>::try_from(node) {
            // Check the entity's scoped identifier for uniqueness.
            // If the scoped identifier is already in the map, that means its identifier is non-unique.
            let scoped_identifier = entity.parser_scoped_identifier();
            if let Some(&other_entity) = slice_definitions.get(&scoped_identifier) {
                // Don't report an error is both entities are modules, since they can re-use identifiers.
                if !(is_module(entity) && is_module(other_entity)) {
                    report_redefinition_error(entity, other_entity, diagnostic_reporter);
                }
            } else {
                // Getting here means we haven't seen the entity's scoped identifier before, so we add it to the map.
                slice_definitions.insert(scoped_identifier, entity);
            }
        }
    }
}

fn is_module(entity: &dyn Entity) -> bool {
    matches!(entity.concrete_entity(), Entities::Module(_))
}

fn report_redefinition_error(entity: &dyn Entity, original_entity: &dyn Entity, reporter: &mut DiagnosticReporter) {
    Diagnostic::new(Error::Redefinition {
        identifier: entity.identifier().to_owned(),
    })
    .set_span(entity.raw_identifier().span())
    .add_note(
        format!("'{}' was previously defined here", original_entity.identifier()),
        Some(original_entity.raw_identifier().span()),
    )
    .report(reporter);
}
