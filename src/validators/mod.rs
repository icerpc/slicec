// Copyright (c) ZeroC, Inc. All rights reserved.

mod attribute;
mod comments;
mod dictionary;
mod enums;
mod identifiers;
mod miscellaneous;
mod tag;

use crate::diagnostics::DiagnosticsReporter;
use crate::grammar::*;
use crate::parse_result::{ParsedData, ParserResult};
use crate::utils::ptr_util::WeakPtr;
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

pub enum Validator {
    Attributes(fn(&dyn Attributable, &mut DiagnosticsReporter)),
    Dictionaries(fn(&[&Dictionary], &mut DiagnosticsReporter)),
    Enums(fn(&Enum, &mut DiagnosticsReporter)),
    Entities(fn(&dyn Entity, &mut DiagnosticsReporter)),
    Members(fn(Vec<&dyn Member>, &mut DiagnosticsReporter)),
    Identifiers(fn(Vec<&Identifier>, &mut DiagnosticsReporter)),
    InheritedIdentifiers(fn(Vec<&Identifier>, Vec<&Identifier>, &mut DiagnosticsReporter)),
    Operations(fn(&Operation, &mut DiagnosticsReporter)),
    Parameters(fn(&[&Parameter], &mut DiagnosticsReporter)),
    Struct(fn(&Struct, &mut DiagnosticsReporter)),
}

pub(crate) fn validate_parsed_data(mut data: ParsedData) -> ParserResult {
    let mut validator = ValidatorVisitor::new(&mut data.diagnostic_reporter);

    for slice_file in data.files.values() {
        slice_file.visit_with(&mut validator);
    }

    data.into()
}

struct ValidatorVisitor<'a> {
    diagnostic_reporter: &'a mut DiagnosticsReporter,
    validation_functions: Vec<Validator>,
}

impl<'a> ValidatorVisitor<'a> {
    pub fn new(diagnostic_reporter: &'a mut DiagnosticsReporter) -> Self {
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
            diagnostic_reporter,
            validation_functions,
        }
    }

    fn validate(&mut self, func: impl Fn(&Validator, &mut DiagnosticsReporter)) {
        for validator_function in &self.validation_functions {
            func(validator_function, self.diagnostic_reporter);
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

fn container_dictionaries<T>(container: &dyn Container<WeakPtr<T>>) -> Vec<&Dictionary>
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
    fn visit_class_start(&mut self, class: &Class) {
        self.validate(|validator, diagnostic_reporter| match validator {
            Validator::Attributes(function) => function(class, diagnostic_reporter),
            Validator::Dictionaries(function) => function(&container_dictionaries(class), diagnostic_reporter),
            Validator::Entities(function) => function(class, diagnostic_reporter),
            Validator::Identifiers(function) => function(class.members().get_identifiers(), diagnostic_reporter),
            Validator::InheritedIdentifiers(function) => function(
                class.members().get_identifiers(),
                class.all_inherited_members().get_identifiers(),
                diagnostic_reporter,
            ),
            Validator::Members(function) => function(class.members().as_member_vec(), diagnostic_reporter),
            _ => {}
        });
    }

    fn visit_enum_start(&mut self, enum_def: &Enum) {
        self.validate(|validator, diagnostic_reporter| match validator {
            Validator::Attributes(function) => function(enum_def, diagnostic_reporter),
            Validator::Entities(function) => function(enum_def, diagnostic_reporter),
            Validator::Enums(function) => function(enum_def, diagnostic_reporter),
            _ => {}
        });
    }

    fn visit_exception_start(&mut self, exception: &Exception) {
        self.validate(|validator, diagnostic_reporter| match validator {
            Validator::Attributes(function) => function(exception, diagnostic_reporter),
            Validator::Dictionaries(function) => function(&container_dictionaries(exception), diagnostic_reporter),
            Validator::Entities(function) => function(exception, diagnostic_reporter),
            Validator::Identifiers(function) => function(exception.members().get_identifiers(), diagnostic_reporter),
            Validator::InheritedIdentifiers(function) => function(
                exception.members().get_identifiers(),
                exception.all_inherited_members().get_identifiers(),
                diagnostic_reporter,
            ),
            Validator::Members(function) => function(exception.members().as_member_vec(), diagnostic_reporter),
            _ => {}
        });
    }

    fn visit_interface_start(&mut self, interface: &Interface) {
        self.validate(|validator, diagnostic_reporter| match validator {
            Validator::Attributes(function) => function(interface, diagnostic_reporter),
            Validator::Entities(function) => function(interface, diagnostic_reporter),
            Validator::Identifiers(function) => function(interface.operations().get_identifiers(), diagnostic_reporter),
            Validator::InheritedIdentifiers(function) => function(
                interface.operations().get_identifiers(),
                interface.all_inherited_operations().get_identifiers(),
                diagnostic_reporter,
            ),
            _ => {}
        });
    }

    fn visit_module_start(&mut self, module_def: &Module) {
        self.validate(|validator, diagnostic_reporter| match validator {
            Validator::Entities(function) => function(module_def, diagnostic_reporter),
            Validator::Identifiers(function) => {
                let identifiers = module_def
                    .contents()
                    .iter()
                    .map(|definition| definition.borrow().raw_identifier())
                    .collect::<Vec<_>>();
                function(identifiers, diagnostic_reporter)
            }
            _ => {}
        });
    }

    fn visit_operation_start(&mut self, operation: &Operation) {
        self.validate(|validator, diagnostic_reporter| match validator {
            Validator::Attributes(function) => function(operation, diagnostic_reporter),
            Validator::Dictionaries(function) => {
                function(&member_dictionaries(operation.parameters()), diagnostic_reporter);
                function(&member_dictionaries(operation.return_members()), diagnostic_reporter);
            }
            Validator::Entities(function) => function(operation, diagnostic_reporter),
            Validator::Members(function) => {
                function(operation.parameters().as_member_vec(), diagnostic_reporter);
                function(operation.return_members().as_member_vec(), diagnostic_reporter);
            }
            Validator::Operations(function) => function(operation, diagnostic_reporter),
            Validator::Parameters(function) => {
                function(operation.parameters().as_slice(), diagnostic_reporter);
                function(operation.return_members().as_slice(), diagnostic_reporter);
            }
            _ => {}
        });
    }

    fn visit_parameter(&mut self, parameter: &Parameter) {
        self.validate(|validator, diagnostic_reporter| {
            if let Validator::Attributes(function) = validator {
                function(parameter, diagnostic_reporter)
            }
        })
    }

    fn visit_struct_start(&mut self, struct_def: &Struct) {
        self.validate(|validator, diagnostic_reporter| match validator {
            Validator::Attributes(function) => function(struct_def, diagnostic_reporter),
            Validator::Dictionaries(function) => function(&container_dictionaries(struct_def), diagnostic_reporter),
            Validator::Entities(function) => function(struct_def, diagnostic_reporter),
            Validator::Identifiers(function) => function(struct_def.members().get_identifiers(), diagnostic_reporter),
            Validator::Members(function) => function(struct_def.members().as_member_vec(), diagnostic_reporter),
            Validator::Struct(function) => function(struct_def, diagnostic_reporter),
            _ => {}
        });
    }

    fn visit_type_alias(&mut self, type_alias: &TypeAlias) {
        self.validate(|validator, diagnostic_reporter| match validator {
            Validator::Dictionaries(function) => {
                if let Types::Dictionary(dictionary) = type_alias.underlying.concrete_type() {
                    function(&[dictionary], diagnostic_reporter)
                }
            }
            Validator::Entities(function) => function(type_alias, diagnostic_reporter),
            _ => {}
        });
    }
}
