// Copyright (c) ZeroC, Inc. All rights reserved.

mod attribute;
mod dictionary;
mod enums;
mod identifiers;
mod miscellaneous;
mod tag;

use crate::error::{Error, ErrorReporter};
use crate::grammar::*;
use crate::ptr_util::OwnedPtr;
use crate::slice_file::SliceFile;
use crate::visitor::Visitor;
use std::collections::HashMap;

// Re-export the contents of the validators submodules directly into the validators module. This is
// for convenience, so users don't need to worry about the submodule structure while importing.
pub use self::attribute::*;
pub use self::dictionary::*;
pub use self::enums::*;
pub use self::identifiers::*;
pub use self::miscellaneous::*;
pub use self::tag::*;

pub type ValidationChain = Vec<Validate>;
pub type ValidationResult = Result<(), Vec<Error>>;

pub enum Validate {
    Attributable(fn(&dyn Attributable) -> ValidationResult),
    Class(fn(&Class) -> ValidationResult),
    Dictionary(fn(&[&Dictionary]) -> ValidationResult),
    Enums(fn(&Enum) -> ValidationResult),
    Exception(fn(&[&DataMember]) -> ValidationResult),
    Interface(fn(&Interface) -> ValidationResult),
    Members(fn(&[&DataMember]) -> ValidationResult),
    Identifiers(fn(Vec<&Identifier>) -> ValidationResult),
    InheritedIdentifiers(fn(Vec<&Identifier>, Vec<&Identifier>) -> ValidationResult),
    Operation(fn(&Operation) -> ValidationResult),
    Parameter(fn(&Parameter) -> ValidationResult),
    Parameters(fn(&[&Parameter]) -> ValidationResult),
    ParametersAndReturnMember(fn(&[&Parameter]) -> ValidationResult),
    Struct(fn(&Struct) -> ValidationResult),
}

pub(crate) struct Validator<'a> {
    pub error_reporter: &'a mut ErrorReporter,
    validation_functions: Vec<Validate>,
    errors: Vec<Error>,
}

impl<'a> Validator<'a> {
    pub fn new(error_reporter: &'a mut ErrorReporter) -> Self {
        let validation_functions = vec![
            dictionary_validators(),
            tag_validators(),
            enum_validators(),
            attribute_validators(),
            identifier_validators(),
            miscellaneous_validators(),
        ]
        .into_iter()
        .flatten()
        .collect();
        Validator {
            error_reporter,
            validation_functions,
            errors: Vec::new(),
        }
    }

    /// This method is responsible for visiting each slice file with the various validators.
    pub fn validate(&mut self, slice_files: &HashMap<String, SliceFile>) {
        for slice_file in slice_files.values() {
            slice_file.visit_with(self);
            self.error_reporter.report_errors(&self.errors);
        }
    }
}

trait EntityIdentifiersExtension {
    fn get_identifiers(&self) -> Vec<&Identifier>;
}

impl<T> EntityIdentifiersExtension for Vec<&T>
where
    T: Entity,
{
    fn get_identifiers(&self) -> Vec<&Identifier> {
        self.iter().map(|member| member.raw_identifier()).collect()
    }
}

fn container_dictionaries<T>(container: &dyn Container<OwnedPtr<T>>) -> Vec<&Dictionary>
where
    T: Member,
{
    container
        .contents()
        .iter()
        .filter_map(|member| match member.borrow().data_type().concrete_type() {
            Types::Dictionary(dictionary) => Some(dictionary),
            _ => None,
        })
        .collect()
}

fn member_dictionaries<T>(members: Vec<&T>) -> Vec<&Dictionary>
where
    T: Member,
{
    members
        .iter()
        .filter_map(|member| match member.data_type().concrete_type() {
            Types::Dictionary(dictionary) => Some(dictionary),
            _ => None,
        })
        .collect()
}

impl<'a> Visitor for Validator<'a> {
    fn visit_class_start(&mut self, class: &Class) {
        let mut errors = vec![];
        self.validation_functions
            .iter()
            .filter_map(|function| match function {
                Validate::Class(function) => Some(function(class)),
                Validate::Dictionary(function) => Some(function(&container_dictionaries(class))),
                Validate::Members(function) => Some(function(class.members().as_slice())),
                Validate::Attributable(function) => Some(function(class)),
                Validate::Identifiers(function) => Some(function(class.members().get_identifiers())),
                Validate::InheritedIdentifiers(function) => Some(function(
                    class.members().get_identifiers(),
                    class.all_inherited_members().get_identifiers(),
                )),
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
                Validate::Dictionary(function) => Some(function(&container_dictionaries(struct_def))),
                Validate::Members(function) => Some(function(struct_def.members().as_slice())),
                Validate::Attributable(function) => Some(function(struct_def)),
                Validate::Identifiers(function) => Some(function(struct_def.members().get_identifiers())),
                _ => None,
            })
            .for_each(|result| match result {
                Ok(_) => (),
                Err(mut errs) => errors.append(&mut errs),
            });
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

    fn visit_exception_start(&mut self, exception: &Exception) {
        let mut errors = vec![];
        self.validation_functions
            .iter()
            .filter_map(|function| match function {
                Validate::Dictionary(function) => Some(function(&container_dictionaries(exception))),
                Validate::Exception(function) => Some(function(exception.members().as_slice())),
                Validate::Attributable(function) => Some(function(exception)),
                Validate::Identifiers(function) => Some(function(exception.members().get_identifiers())),
                Validate::InheritedIdentifiers(function) => Some(function(
                    exception.members().get_identifiers(),
                    exception.all_inherited_members().get_identifiers(),
                )),
                _ => None,
            })
            .for_each(|result| match result {
                Ok(_) => (),
                Err(mut errs) => errors.append(&mut errs),
            });
        self.errors.append(&mut errors);
    }

    fn visit_interface_start(&mut self, interface: &Interface) {
        let mut errors = vec![];
        self.validation_functions
            .iter()
            .filter_map(|function| match function {
                Validate::Interface(function) => Some(function(interface)),
                Validate::Attributable(function) => Some(function(interface)),
                Validate::Identifiers(function) => Some(function(interface.operations().get_identifiers())),
                Validate::InheritedIdentifiers(function) => Some(function(
                    interface.operations().get_identifiers(),
                    interface.all_inherited_operations().get_identifiers(),
                )),
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
                Validate::Dictionary(function) => Some(function(&member_dictionaries(
                    operation.parameters_and_return_members(),
                ))),
                Validate::Operation(function) => Some(function(operation)),
                Validate::Attributable(function) => Some(function(operation)),
                Validate::Parameters(function) => Some(function(operation.parameters().as_slice())),
                Validate::ParametersAndReturnMember(function) => {
                    Some(function(&operation.parameters_and_return_members()))
                }
                _ => None,
            })
            .for_each(|result| match result {
                Ok(_) => (),
                Err(mut errs) => errors.append(&mut errs),
            });
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

    fn visit_type_alias(&mut self, type_alias: &TypeAlias) {
        let mut errors = vec![];
        self.validation_functions
            .iter()
            .filter_map(|function| match function {
                Validate::Dictionary(function) => match type_alias.underlying.concrete_type() {
                    Types::Dictionary(dictionary) => Some(function(&[dictionary])),
                    _ => None,
                },
                _ => None,
            })
            .for_each(|result| match result {
                Ok(_) => (),
                Err(mut errs) => errors.append(&mut errs),
            });
        self.errors.append(&mut errors);
    }
}
