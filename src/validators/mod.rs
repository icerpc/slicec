// Copyright (c) ZeroC, Inc. All rights reserved.

mod attribute;
mod comments;
mod dictionary;
mod enums;
mod identifiers;
mod miscellaneous;
mod tag;

use crate::error::{Error, ErrorReporter};
use crate::grammar::*;
use crate::parse_result::{ParsedData, ParserResult};
use crate::ptr_util::OwnedPtr;
use crate::visitor::Visitor;

// Re-export the contents of the validators submodules directly into the validators module. This is
// for convenience, so users don't need to worry about the submodule structure while importing.
pub use self::attribute::*;
pub use self::comments::*;
pub use self::dictionary::*;
pub use self::enums::*;
pub use self::identifiers::*;
pub use self::miscellaneous::*;
pub use self::tag::*;

pub type ValidationChain = Vec<Validator>;
pub type ValidationResult = Result<(), Vec<Error>>;

pub enum Validator {
    Attributes(fn(&dyn Attributable) -> ValidationResult),
    Dictionaries(fn(&[&Dictionary]) -> ValidationResult),
    Enums(fn(&Enum) -> ValidationResult),
    Members(fn(Vec<&dyn Member>) -> ValidationResult),
    Identifiers(fn(Vec<&Identifier>) -> ValidationResult),
    InheritedIdentifiers(fn(Vec<&Identifier>, Vec<&Identifier>) -> ValidationResult),
    Operations(fn(&Operation) -> ValidationResult),
    Parameters(fn(&[&Parameter]) -> ValidationResult),
    Struct(fn(&Struct) -> ValidationResult),
}

pub(crate) fn validate_parsed_data(mut data: ParsedData) -> ParserResult {
    let mut validator = ValidatorVisitor::new(&mut data.error_reporter);

    for slice_file in data.files.values() {
        slice_file.visit_with(&mut validator);
    }

    data.into()
}

struct ValidatorVisitor<'a> {
    error_reporter: &'a mut ErrorReporter,
    validation_functions: Vec<Validator>,
}

impl<'a> ValidatorVisitor<'a> {
    pub fn new(error_reporter: &'a mut ErrorReporter) -> Self {
        let validation_functions = vec![
            attribute_validators(),
            comments_validators(),
            dictionary_validators(),
            enum_validators(),
            identifier_validators(),
            miscellaneous_validators(),
            tag_validators(),
        ]
        .into_iter()
        .flatten()
        .collect();
        ValidatorVisitor {
            error_reporter,
            validation_functions,
        }
    }

    fn validate<F>(&mut self, func: F)
    where
        F: Fn(&Validator) -> Option<ValidationResult>,
    {
        let error_reporter = &mut self.error_reporter;
        self.validation_functions
            .iter()
            .filter_map(func)
            .for_each(|result| match result {
                Ok(_) => (),
                Err(errs) => error_reporter.append_errors(errs),
            });
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

trait AsMemberVecExt {
    fn as_member_vec(&self) -> Vec<&dyn Member>;
}

impl<T: Member> AsMemberVecExt for Vec<&T> {
    fn as_member_vec(&self) -> Vec<&dyn Member> {
        let mut v: Vec<&dyn Member> = Vec::new();
        self.iter().for_each(|m| v.push(*m));
        v
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

impl<'a> Visitor for ValidatorVisitor<'a> {
    fn visit_module_start(&mut self, module_def: &Module) {
        self.validate(|function| match function {
            Validator::Identifiers(function) => {
                let identifiers = module_def
                    .contents()
                    .iter()
                    .map(|definition| definition.borrow().raw_identifier())
                    .collect::<Vec<_>>();
                Some(function(identifiers))
            }
            _ => None,
        });
    }

    fn visit_class_start(&mut self, class: &Class) {
        self.validate(|function| match function {
            Validator::Attributes(function) => Some(function(class)),
            Validator::Dictionaries(function) => Some(function(&container_dictionaries(class))),
            Validator::Identifiers(function) => Some(function(class.members().get_identifiers())),
            Validator::InheritedIdentifiers(function) => Some(function(
                class.members().get_identifiers(),
                class.all_inherited_members().get_identifiers(),
            )),
            Validator::Members(function) => Some(function(class.members().as_member_vec())),
            _ => None,
        });
    }

    fn visit_struct_start(&mut self, struct_def: &Struct) {
        self.validate(|function| match function {
            Validator::Attributes(function) => Some(function(struct_def)),
            Validator::Dictionaries(function) => Some(function(&container_dictionaries(struct_def))),
            Validator::Identifiers(function) => Some(function(struct_def.members().get_identifiers())),
            Validator::Members(function) => Some(function(struct_def.members().as_member_vec())),
            Validator::Struct(function) => Some(function(struct_def)),
            _ => None,
        });
    }

    fn visit_enum_start(&mut self, enum_def: &Enum) {
        self.validate(|function| match function {
            Validator::Attributes(function) => Some(function(enum_def)),
            Validator::Enums(function) => Some(function(enum_def)),
            _ => None,
        });
    }

    fn visit_exception_start(&mut self, exception: &Exception) {
        self.validate(|function| match function {
            Validator::Attributes(function) => Some(function(exception)),
            Validator::Dictionaries(function) => Some(function(&container_dictionaries(exception))),
            Validator::Identifiers(function) => Some(function(exception.members().get_identifiers())),
            Validator::InheritedIdentifiers(function) => Some(function(
                exception.members().get_identifiers(),
                exception.all_inherited_members().get_identifiers(),
            )),
            Validator::Members(function) => Some(function(exception.members().as_member_vec())),
            _ => None,
        });
    }

    fn visit_interface_start(&mut self, interface: &Interface) {
        self.validate(|function| match function {
            Validator::Attributes(function) => Some(function(interface)),
            Validator::Identifiers(function) => Some(function(interface.operations().get_identifiers())),
            Validator::InheritedIdentifiers(function) => Some(function(
                interface.operations().get_identifiers(),
                interface.all_inherited_operations().get_identifiers(),
            )),
            _ => None,
        });
    }

    fn visit_operation_start(&mut self, operation: &Operation) {
        self.validate(|function| match function {
            Validator::Attributes(function) => Some(function(operation)),
            Validator::Dictionaries(function) => Some(function(&member_dictionaries(
                operation.parameters_and_return_members(),
            ))),
            Validator::Members(function) => Some(function(operation.parameters_and_return_members().as_member_vec())),
            Validator::Operations(function) => Some(function(operation)),
            Validator::Parameters(function) => Some(function(operation.parameters_and_return_members().as_slice())),
            _ => None,
        });
    }

    fn visit_parameter(&mut self, parameter: &Parameter) {
        self.validate(|function| match function {
            Validator::Attributes(function) => Some(function(parameter)),
            _ => None,
        });
    }

    fn visit_type_alias(&mut self, type_alias: &TypeAlias) {
        self.validate(|function| match function {
            Validator::Dictionaries(function) => match type_alias.underlying.concrete_type() {
                Types::Dictionary(dictionary) => Some(function(&[dictionary])),
                _ => None,
            },
            _ => None,
        });
    }
}
