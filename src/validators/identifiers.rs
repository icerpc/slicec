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
    RedefinitionChecker { diagnostics }.check_for_redefinitions(ast);
}

struct RedefinitionChecker<'a> {
    diagnostics: &'a mut Diagnostics,
}

impl<'a> RedefinitionChecker<'a> {
    fn check_for_redefinitions(&mut self, ast: &'a Ast) {
        // Stores all the _module-scoped_ Slice definitions we've seen so far.
        // Keys are the definition's fully-scoped identifiers, and values are references to the definitions themselves.
        let mut seen_definitions = HashMap::new();

        for node in ast.as_slice() {
            // We only check `Entity`s so as to exclude any Slice elements which don't have names (and hence cannot be
            // redefined), and also to exclude modules (which are reopened, not redefined).
            let Ok(definition) = <&dyn Entity>::try_from(node) else { continue };

            match definition.concrete_entity() {
                Entities::Struct(struct_def) => {
                    self.check_if_redefined(struct_def, &mut seen_definitions);
                    self.check_contents_for_redefinitions(struct_def.contents());
                }
                Entities::Class(class_def) => {
                    self.check_if_redefined(class_def, &mut seen_definitions);
                    self.check_contents_for_redefinitions(class_def.contents());
                }
                Entities::Exception(exception_def) => {
                    self.check_if_redefined(exception_def, &mut seen_definitions);
                    self.check_contents_for_redefinitions(exception_def.contents());
                }
                Entities::Interface(interface_def) => {
                    self.check_if_redefined(interface_def, &mut seen_definitions);
                    self.check_contents_for_redefinitions(interface_def.contents());

                    for operation in interface_def.operations() {
                        self.check_contents_for_redefinitions(operation.parameters());
                        self.check_contents_for_redefinitions(operation.return_members());
                    }
                }
                Entities::Enum(enum_def) => {
                    self.check_if_redefined(enum_def, &mut seen_definitions);
                    self.check_contents_for_redefinitions(enum_def.contents());
                }
                Entities::CustomType(custom_type) => {
                    self.check_if_redefined(custom_type, &mut seen_definitions);
                }
                Entities::TypeAlias(type_alias) => {
                    self.check_if_redefined(type_alias, &mut seen_definitions);
                }

                // No need to check `Field`, `Enumerator`, `Operation`, or `Parameter`; We just check their containers.
                Entities::Field(_) | Entities::Enumerator(_) | Entities::Operation(_) | Entities::Parameter(_) => {}
            }
        }
    }

    fn check_contents_for_redefinitions<T: NamedSymbol>(&mut self, contents: Vec<&T>) {
        // We create a separate hashmap, so redefinitions are isolated to just the container we're checking.
        let mut seen_definitions = HashMap::new();
        for element in contents {
            self.check_if_redefined(element, &mut seen_definitions);
        }
    }

    /// Checks if the provided `definition` already has an entry in the `already_seen` map. If it does, we report a
    /// redefinition error, otherwise, we just add it to the map and return.
    fn check_if_redefined<'b>(
        &mut self,
        definition: &'b impl NamedSymbol,
        already_seen: &mut HashMap<String, &'b dyn NamedSymbol>,
    ) {
        let scoped_identifier = definition.parser_scoped_identifier();

        if let Some(other_definition) = already_seen.get(&scoped_identifier) {
            // We found a name collision; report an error.
            self.report_redefinition_error(definition, *other_definition);
        } else {
            // This is the first time we've seen this identifier, so we add it to the map.
            already_seen.insert(scoped_identifier, definition);
        }
    }

    fn report_redefinition_error(&mut self, new: &dyn NamedSymbol, original: &dyn NamedSymbol) {
        Diagnostic::new(Error::Redefinition {
            identifier: new.identifier().to_owned(),
        })
        .set_span(new.raw_identifier().span())
        .add_note(
            format!("'{}' was previously defined here", original.identifier()),
            Some(original.raw_identifier().span()),
        )
        .push_into(self.diagnostics);
    }
}
