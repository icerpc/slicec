// Copyright (c) ZeroC, Inc.

use crate::ast::Ast;
use crate::diagnostics::{Diagnostic, Diagnostics, Error};
use crate::grammar::*;
use std::collections::HashMap;

pub fn validate_inherited_identifiers(
    symbols: Vec<&impl NamedSymbol>,
    inherited_symbols: Vec<&impl NamedSymbol>,
    diagnostics: &mut Diagnostics,
) {
    check_for_shadowing(symbols, inherited_symbols, diagnostics);
}

fn check_for_shadowing(
    symbols: Vec<&impl NamedSymbol>,
    inherited_symbols: Vec<&impl NamedSymbol>,
    diagnostics: &mut Diagnostics,
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
                .push_into(diagnostics);
            }
        }
    }
}

pub fn check_for_redefinitions(ast: &Ast, diagnostics: &mut Diagnostics) {
    // A map storing `(scoped_identifier, named_symbol)` pairs. We iterate through the AST and build up this map,
    // and if we try to add an entry but see it's already occupied, that means we've found a redefinition.
    let mut slice_definitions: HashMap<String, &dyn NamedSymbol> = HashMap::new();

    // A list of all the 'scoped_identifier's we've issued redefinition errors for.
    // This is to avoid emitting errors for children of redefined containers.
    // NOTE: These entries must all end in "::" to avoid false matches between things like "foo" and "foobar".
    let mut redefinitions: Vec<String> = Vec::new();

    // Iterate through the nodes in reverse. This is important because our parser is 'bottom-up', meaning we create
    // fields before we create the struct that holds them. So, by iterating in reverse, we'll see the containers before
    // we see their children elements.
    for node in ast.as_slice().iter().rev() {
        // Only check nodes that have identifiers, everything else is irrelevant.
        if let Ok(definition) = <&dyn NamedSymbol>::try_from(node) {
            let scoped_identifier = definition.parser_scoped_identifier();
            match slice_definitions.get(&scoped_identifier) {
                // If we've already seen a node with this identifier, there's a name collision.
                // This is fine for modules (since they can be re-opened), but for any other type, we report an error.
                Some(other_definition) => {
                    let are_both_modules = is_module(definition) && is_module(*other_definition);
                    let parent_is_redefined = redefinitions.iter().any(|redefined_parent_id| {
                        scoped_identifier.contains(redefined_parent_id) && scoped_identifier != *redefined_parent_id
                    });

                    if !are_both_modules && !parent_is_redefined {
                        report_redefinition_error(definition, *other_definition, diagnostics);
                        redefinitions.push(scoped_identifier + "::");
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

// TODO improve this function.
fn is_module(definition: &dyn NamedSymbol) -> bool {
    definition.kind() == "module"
}

fn report_redefinition_error(new: &dyn NamedSymbol, original: &dyn NamedSymbol, diagnostics: &mut Diagnostics) {
    Diagnostic::new(Error::Redefinition {
        identifier: new.identifier().to_owned(),
    })
    .set_span(new.raw_identifier().span())
    .add_note(
        format!("'{}' was previously defined here", original.identifier()),
        Some(original.raw_identifier().span()),
    )
    .push_into(diagnostics);
}
