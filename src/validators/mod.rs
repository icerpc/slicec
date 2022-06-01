// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::ast::Ast;
use crate::error::{Error, ErrorReporter};
use crate::grammar::*;
use crate::slice_file::SliceFile;
use crate::validators::DictionaryValidator;
use crate::visitor::Visitor;
use std::collections::HashMap;

mod attribute;
mod dictionary;
mod enums;
mod tag;

// Re-export the contents of the validators submodules directly into the validators module. This is
// for convenience, so users don't need to worry about the submodule structure while importing.
pub use self::attribute::*;
pub use self::dictionary::*;
pub use self::enums::*;
pub use self::tag::*;

pub type ValidationChain = Vec<Validate>;
pub type ValidationResult = Result<(), Vec<Error>>;
pub enum Validate {
    Attributable(fn(&dyn Attributable) -> ValidationResult),
    Class(fn(&Class) -> ValidationResult),
    Members(fn(&[&DataMember]) -> ValidationResult),
    Parameters(fn(&[&Parameter]) -> ValidationResult),
    Parameter(fn(&Parameter) -> ValidationResult),
    Struct(fn(&Struct) -> ValidationResult),
    Enums(fn(&Enum) -> ValidationResult),
    Interface(fn(&Interface) -> ValidationResult),
    Operation(fn(&Operation) -> ValidationResult),
    Exception(fn(&[&DataMember]) -> ValidationResult),
}

pub(crate) struct Validator<'a> {
    pub error_reporter: &'a mut ErrorReporter,
    pub ast: &'a Ast,
    validation_functions: Vec<Validate>,
    errors: Vec<Error>,
}

impl<'a> Validator<'a> {
    pub fn new(error_reporter: &'a mut ErrorReporter, ast: &'a Ast) -> Validator<'a> {
        Validator {
            error_reporter,
            ast,
            validation_functions: Vec::new(),
            errors: Vec::new(),
        }
    }

    /// This method is responsible for visiting each slice file with the various validators.
    pub fn validate(&mut self, slice_files: &HashMap<String, SliceFile>) {
        self.add_validation_functions(tag_validators());
        self.add_validation_functions(enum_validators());
        self.add_validation_functions(attribute_validators());
        for slice_file in slice_files.values() {
            slice_file.visit_with(self);
            self.error_reporter.report_errors(&self.errors);
            // TODO: Dictionaries are being changed.
            let dictionary_validator =
                &mut DictionaryValidator { error_reporter: self.error_reporter, ast: self.ast };
            dictionary_validator.validate_dictionary_key_types();
        }
    }

    pub fn add_validation_functions(&mut self, validation_functions: Vec<Validate>) {
        self.validation_functions.extend(validation_functions);
    }

    // Miscellaneous validators
    fn validate_stream_member(&mut self, members: Vec<&Parameter>) {
        // If members is empty, `split_last` returns None, and this check is skipped,
        // otherwise it returns all the members, except for the last one. None of these members
        // can be streamed, since only the last member can be.
        if let Some((_, nonstreamed_members)) = members.split_last() {
            for member in nonstreamed_members {
                if member.is_streamed {
                    self.error_reporter.report_error(
                        "only the last parameter in an operation can use the stream modifier",
                        Some(&member.location),
                    );
                }
            }
        }
    }

    fn validate_compact_struct_not_empty(&mut self, struct_def: &Struct) {
        if struct_def.is_compact {
            // Compact structs must be non-empty.
            if struct_def.members().is_empty() {
                self.error_reporter.report_error(
                    "compact structs must be non-empty",
                    Some(&struct_def.location),
                )
            }
        }
    }
}

impl<'a> Visitor for Validator<'a> {
    fn visit_class_start(&mut self, class_def: &Class) {
        let mut errors = vec![];

        self.validation_functions
            .iter()
            .filter_map(|function| match function {
                Validate::Class(function) => Some(function(class_def)),
                Validate::Members(function) => Some(function(class_def.members().as_slice())),
                Validate::Attributable(function) => Some(function(class_def)),
                _ => None,
            })
            .for_each(|result| match result {
                Ok(_) => (),
                Err(mut errs) => errors.append(&mut errs),
            });
        self.errors.append(&mut errors);
    }

    fn visit_struct_start(&mut self, struct_def: &Struct) {
        let mut errors = vec![];
        self.validation_functions
            .iter()
            .filter_map(|function| match function {
                Validate::Struct(function) => Some(function(struct_def)),
                Validate::Members(function) => Some(function(struct_def.members().as_slice())),
                Validate::Attributable(function) => Some(function(struct_def)),
                _ => None,
            })
            .for_each(|result| match result {
                Ok(_) => (),
                Err(mut errs) => errors.append(&mut errs),
            });
        self.validate_compact_struct_not_empty(struct_def);
        self.errors.append(&mut errors);
    }

    fn visit_enum_start(&mut self, enum_def: &Enum) {
        let mut errors = vec![];
        self.validation_functions
            .iter()
            .filter_map(|function| match function {
                Validate::Enums(function) => Some(function(enum_def)),
                Validate::Attributable(function) => Some(function(enum_def)),
                _ => None,
            })
            .for_each(|result| match result {
                Ok(_) => (),
                Err(mut errs) => errors.append(&mut errs),
            });
        self.errors.append(&mut errors);
    }

    fn visit_interface_start(&mut self, interface_def: &Interface) {
        let mut errors = vec![];
        self.validation_functions
            .iter()
            .filter_map(|function| match function {
                Validate::Interface(function) => Some(function(interface_def)),
                Validate::Attributable(function) => Some(function(interface_def)),
                _ => None,
            })
            .for_each(|result| match result {
                Ok(_) => (),
                Err(mut errs) => errors.append(&mut errs),
            });
        self.errors.append(&mut errors);
    }

    fn visit_operation_start(&mut self, operation: &Operation) {
        let mut errors = vec![];
        self.validation_functions
            .iter()
            .filter_map(|function| match function {
                Validate::Operation(function) => Some(function(operation)),
                Validate::Attributable(function) => Some(function(operation)),
                Validate::Parameters(function) => Some(function(operation.parameters().as_slice())),
                _ => None,
            })
            .for_each(|result| match result {
                Ok(_) => (),
                Err(mut errs) => errors.append(&mut errs),
            });
        self.validate_stream_member(operation.parameters());
        self.validate_stream_member(operation.return_members());
        self.errors.append(&mut errors);
    }

    fn visit_parameter(&mut self, parameter: &Parameter) {
        let mut errors = vec![];
        self.validation_functions
            .iter()
            .filter_map(|function| match function {
                Validate::Parameter(function) => Some(function(parameter)),
                Validate::Attributable(function) => Some(function(parameter)),
                _ => None,
            })
            .for_each(|result| match result {
                Ok(_) => (),
                Err(mut errs) => errors.append(&mut errs),
            });
        self.errors.append(&mut errors);
    }
}
