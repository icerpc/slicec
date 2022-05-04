// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::ast::Ast;
use crate::grammar::*;
use crate::error::ErrorReporter;
use crate::visitor::Visitor;

#[derive(Debug)]
pub(crate) struct Validator<'a> {
    pub error_reporter: &'a mut ErrorReporter,
}

impl Validator<'_> {
    pub fn validate_dictionary_key_types(&mut self, ast: &Ast) {
        for type_ptr in &ast.anonymous_types {
            if let Types::Dictionary(dictionary) = type_ptr.borrow().concrete_type() {
                self.check_dictionary_key_type(&dictionary.key_type);
            }
        }
    }

    fn check_dictionary_key_type(&mut self, type_ref: &TypeRef) -> bool {
        // Optional types cannot be used as dictionary keys.
        if type_ref.is_optional {
            self.error_reporter.report_error(
                "invalid dictionary key type: optional types cannot be used as a dictionary key type".to_owned(),
                Some(&type_ref.location),
            );
            return false;
        }

        let definition = type_ref.definition();
        let (is_valid, named_symbol): (bool, Option<&dyn NamedSymbol>) = match definition.concrete_type() {
            Types::Struct(struct_def) => {
                // Only compact structs can be used for dictionary keys.
                if !struct_def.is_compact {
                    self.error_reporter.report_error(
                        "invalid dictionary key type: structs must be compact to be used as a dictionary key type".to_owned(),
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
                !matches!(primitive, Primitive::Float32 | Primitive::Float64 | Primitive::AnyClass),
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

// TODO add additional validation logic here.
impl<'a> Visitor for Validator<'a> {
    fn visit_struct_start(&mut self, struct_def: &Struct) {
        if struct_def.is_compact {
            // Compact structs must be non-empty.
            if struct_def.members().is_empty() {
                self.error_reporter.report_error(
                    "compact structs must be non-empty".to_owned(),
                    Some(&struct_def.location),
                )
            } else {
                // Compact structs cannot have tagged data members.
                let mut has_tags = false;
                for member in struct_def.members() {
                    if member.tag.is_some() {
                    self.error_reporter.report_error(
                            "tagged data members are not supported in compact structs\n\
                            consider removing the tag, or making the struct non-compact".to_owned(),
                            Some(&member.location),
                        );
                        has_tags = true;
                    }
                }

                if has_tags {
                self.error_reporter.report_note(
                        format!("struct '{}' is declared compact here", struct_def.identifier()),
                        Some(&struct_def.location),
                    );
                }
            }
        }
    }

    fn visit_operation_start(&mut self, operation_def: &Operation) {
        fn validate_stream_member(error_reporter: &mut ErrorReporter, members: Vec<&Parameter>) {
            // If members is empty, `split_last` returns None, and this check is skipped,
            // otherwise it returns all the members, except for the last one. None of these members
            // can be streamed, since only the last member can be.
            if let Some((_, unstreamable_members)) = members.split_last() {
                for member in unstreamable_members {
                    if member.is_streamed {
                        error_reporter.report_error(
                            "only the last parameter in an operation can be streamed".to_owned(),
                            Some(&member.location),
                        );
                    }
                }
            }
        }

        validate_stream_member(self.error_reporter, operation_def.parameters());
        validate_stream_member(self.error_reporter, operation_def.return_members());
    }
}
