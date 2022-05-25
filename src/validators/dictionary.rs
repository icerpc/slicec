// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::ast::Ast;
use crate::error::ErrorReporter;
use crate::grammar::*;
use crate::visitor::Visitor;

#[derive(Debug)]
pub struct DictionaryValidator<'a> {
    pub error_reporter: &'a mut ErrorReporter,
    pub ast: &'a Ast,
}

impl DictionaryValidator<'_> {
    pub fn validate_dictionary_key_types(&mut self) {
        for type_ptr in &self.ast.anonymous_types {
            if let Types::Dictionary(dictionary) = type_ptr.borrow().concrete_type() {
                self.check_dictionary_key_type(&dictionary.key_type);
            }
        }
    }

    fn check_dictionary_key_type(&mut self, type_ref: &TypeRef) -> bool {
        // Optional types cannot be used as dictionary keys.
        if type_ref.is_optional {
            self.error_reporter.report_error(
                "invalid dictionary key type: optional types cannot be used as a dictionary key type",
                Some(&type_ref.location),
            );
            return false;
        }

        let definition = type_ref.definition();
        let (is_valid, named_symbol): (bool, Option<&dyn NamedSymbol>) = match definition
            .concrete_type()
        {
            Types::Struct(struct_def) => {
                // Only compact structs can be used for dictionary keys.
                if !struct_def.is_compact {
                    self.error_reporter.report_error(
                        "invalid dictionary key type: structs must be compact to be used as a dictionary key type",
                        Some(&type_ref.location),
                    );
                    self.error_reporter.report_note(
                        format!("struct '{}' is defined here:", struct_def.identifier()),
                        Some(&struct_def.location),
                    );
                    return false;
                }

                // Check that all the data members of the struct are also valid key types.
                let mut contains_invalid_key_types = false;
                for member in struct_def.members() {
                    if !self.check_dictionary_key_type(member.data_type()) {
                        self.error_reporter.report_error(
                            format!(
                                "data member '{}' cannot be used as a dictionary key type",
                                member.identifier(),
                            ),
                            Some(&member.location),
                        );
                        contains_invalid_key_types = true;
                    }
                }

                if contains_invalid_key_types {
                    self.error_reporter.report_error(
                        format!(
                            "invalid dictionary key type: struct '{}' contains members that cannot be used as a dictionary key type",
                            struct_def.identifier(),
                        ),
                        Some(&type_ref.location),
                    );
                    self.error_reporter.report_note(
                        format!("struct '{}' is defined here:", struct_def.identifier()),
                        Some(&struct_def.location),
                    );
                    return false;
                }
                return true;
            }
            Types::Class(class_def) => (false, Some(class_def)),
            Types::Exception(exception_def) => (false, Some(exception_def)),
            Types::Interface(interface_def) => (false, Some(interface_def)),
            Types::Enum(_) => (true, None),
            Types::Trait(trait_def) => (false, Some(trait_def)),
            Types::CustomType(_) => (true, None),
            Types::Sequence(_) => (false, None),
            Types::Dictionary(_) => (false, None),
            Types::Primitive(primitive) => (
                !matches!(
                    primitive,
                    Primitive::Float32 | Primitive::Float64 | Primitive::AnyClass
                ),
                None,
            ),
        };

        if !is_valid {
            let pluralized_kind = match definition.concrete_type() {
                Types::Primitive(_) => definition.kind().to_owned(),
                Types::Class(_) => "classes".to_owned(),
                Types::Dictionary(_) => "dictionaries".to_owned(),
                _ => definition.kind().to_owned() + "s",
            };

            self.error_reporter.report_error(
                format!(
                    "invalid dictionary key type: {} cannot be used as a dictionary key type",
                    pluralized_kind,
                ),
                Some(&type_ref.location),
            );

            // If the key type is a user-defined type, point to where it was defined.
            if let Some(named_symbol_def) = named_symbol {
                self.error_reporter.report_note(
                    format!(
                        "{} '{}' is defined here:",
                        named_symbol_def.kind(),
                        named_symbol_def.identifier(),
                    ),
                    Some(named_symbol_def.location()),
                );
            }
        }
        is_valid
    }
}

impl<'a> Visitor for DictionaryValidator<'a> {}
