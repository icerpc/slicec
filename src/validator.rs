// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::ast::Ast;
use crate::error::ErrorReporter;
use crate::grammar::*;
use crate::slice_file::SliceFile;
use crate::validators::{AttributeValidator, EnumValidator, TagValidator};
use crate::visitor::Visitor;
use std::collections::HashMap;

#[derive(Debug)]
pub(crate) struct Validator<'a> {
    pub error_reporter: &'a mut ErrorReporter,
    pub ast: &'a Ast,
}

impl Validator<'_> {
    pub fn validate(&mut self, slice_files: &HashMap<String, SliceFile>) {
        for slice_file in slice_files.values() {
            slice_file.visit_with(self);
            slice_file.visit_with(&mut AttributeValidator { error_reporter: self.error_reporter });
            slice_file.visit_with(&mut EnumValidator {
                error_reporter: self.error_reporter,
                encoding: slice_file.encoding(),
            });
            slice_file.visit_with(&mut TagValidator { error_reporter: self.error_reporter });
        }
        self.validate_dictionary_key_types();
    }

    fn validate_dictionary_key_types(&mut self) {
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
                "invalid dictionary key type: optional types cannot be used as a dictionary key type".to_owned(),
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

    fn validate_stream_member(&mut self, members: Vec<&Parameter>) {
        // If members is empty, `split_last` returns None, and this check is skipped,
        // otherwise it returns all the members, except for the last one. None of these members
        // can be streamed, since only the last member can be.
        if let Some((_, nonstreamed_members)) = members.split_last() {
            for member in nonstreamed_members {
                if member.is_streamed {
                    self.error_reporter.report_error(
                        "only the last parameter in an operation can use the stream modifier"
                            .to_owned(),
                        Some(&member.location),
                    );
                }
            }
        }
    }
}

impl<'a> Visitor for Validator<'a> {
    fn visit_struct_start(&mut self, struct_def: &Struct) {
        if struct_def.is_compact {
            // Compact structs must be non-empty.
            if struct_def.members().is_empty() {
                self.error_reporter.report_error(
                    "compact structs must be non-empty".to_owned(),
                    Some(&struct_def.location),
                )
            }
        }
    }

    fn visit_operation_start(&mut self, operation_def: &Operation) {
        self.validate_stream_member(operation_def.parameters());
        self.validate_stream_member(operation_def.return_members());
    }
}
