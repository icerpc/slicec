// Copyright (c) ZeroC, Inc.

use crate::ast::Ast;
use crate::diagnostics::{Diagnostic, Diagnostics, Error};
use crate::grammar::*;
use crate::visitor::Visitor;
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
    for node in ast.as_slice() {

            // Check if we've already seen another entity with this exact identifier.
            match seen_definitions.get(&scoped_identifier) {
                // If there is, we've found a name collision.
                Some(other_definition) => {
                    // The last thing to check is that these entities both have the same parent element.
                    // Otherwise we can hit false positives where redefining a container can trigger spurious
                    // redefinition errors for it's contents (since the scope segments are still the same).
                    if are_in_the_same_container(definition, *other_definition) {
                        report_redefinition_error(definition, *other_definition, diagnostics);
                    }
                }

                // If we haven't seen this identifier before, add it to the map and continue checking.
                None => {
                    seen_definitions.insert(scoped_identifier, definition);
                }
            }
        }
    }
}

/// Returns true if `definition` and `other` are in the same container, or if they're both at module level.
///
/// Note that when they're both at module level, we don't make sure that this is the same module!
/// This function is only called from `check_for_redefinitions` which already guarantees this.
fn are_in_the_same_container(definition: &dyn Entity, other: &dyn Entity) -> bool {
    eprintln!("Here we are:\n{definition:?}\n{other:?}");

    match definition.concrete_entity() {
        Entities::Field(field) => {
            eprintln!("We made it here at least");
            let x = match other.concrete_entity() {
                Entities::Field(other_field) => {
                    eprintln!("=== what the heck?? {}", field.parent == other_field.parent());
                    field.parent == other_field.parent()
                }
                _ => false, // if they aren't the same type, they can't be in the same container.
            };
            eprintln!("{x}");
            x
        }
        Entities::Enumerator(enumerator) => {
            match other.concrete_entity() {
                Entities::Enumerator(other_enumerator) => enumerator.parent == other_enumerator.parent(),
                _ => false, // if they aren't the same type, they can't be in the same container.
            }
        }
        Entities::Operation(operation) => {
            match other.concrete_entity() {
                Entities::Operation(other_operation) => operation.parent == &other_operation.parent(),
                _ => false, // if they aren't the same type, they can't be in the same container.
            }
        }
        Entities::Parameter(parameter) => {
            match other.concrete_entity() {
                Entities::Parameter(other_parameter) => parameter.parent == &other_parameter.parent(),
                _ => false, // if they aren't the same type, they can't be in the same container.
            }
        }

        // Otherwise the definition is at module level, all we have to do is make sure the other one is too.
        _ => !matches!(
            other.concrete_entity(),
            Entities::Field(_) | Entities::Enumerator(_) | Entities::Operation(_) | Entities::Parameter(_)
        ),
    }
}

fn report_redefinition_error(new: &dyn Entity, original: &dyn Entity, diagnostics: &mut Diagnostics) {
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
