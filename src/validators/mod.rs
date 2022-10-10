// Copyright (c) ZeroC, Inc. All rights reserved.

mod attribute;
mod comments;
mod dictionary;
mod enums;
mod identifiers;
mod miscellaneous;
mod tag;

use crate::ast::Ast;
use crate::ast::node::Node;
use crate::diagnostics::DiagnosticReporter;
use crate::grammar::*;
use crate::parse_result::{ParsedData, ParserResult};
use crate::utils::ptr_util::WeakPtr;
use crate::visitor::Visitor;

use std::collections::HashMap;

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
    Attributes(fn(&dyn Entity, &mut DiagnosticReporter)),
    DocComments(fn(&dyn Entity, &Ast, &mut DiagnosticReporter)),
    Dictionaries(fn(&[&Dictionary], &mut DiagnosticReporter)),
    Enums(fn(&Enum, &mut DiagnosticReporter)),
    Entities(fn(&dyn Entity, &mut DiagnosticReporter)),
    Members(fn(Vec<&dyn Member>, &mut DiagnosticReporter)),
    Identifiers(fn(Vec<&Identifier>, &mut DiagnosticReporter)),
    InheritedIdentifiers(fn(Vec<&Identifier>, Vec<&Identifier>, &mut DiagnosticReporter)),
    Operations(fn(&Operation, &mut DiagnosticReporter)),
    Parameters(fn(&[&Parameter], &mut DiagnosticReporter)),
    Struct(fn(&Struct, &mut DiagnosticReporter)),
}

pub(crate) fn validate_parsed_data(mut data: ParsedData) -> ParserResult {
    let mut validator = ValidatorVisitor::new(&data.ast, &mut data.diagnostic_reporter);

    for slice_file in data.files.values() {
        slice_file.visit_with(&mut validator);
    }

    // Since modules can be re-opened, but each module is a distinct entity in the AST, our normal redefinition check
    // is inadequate; if 2 modules have the same name, we have to check for redefinitions across both modules.
    //
    // So we compute a map of all the identifiers in modules with the same name (fully scoped), then check that.
    let mut merged_module_content_identifiers: HashMap<String, Vec<&Identifier>> = HashMap::new();
    for node in data.ast.as_slice() {
        if let Node::Module(module_ptr) = node {
            // Borrow the module's pointer and store its fully scoped identifier.
            let module = module_ptr.borrow();
            let scoped_module_identifier = module.parser_scoped_identifier();
            // Get all the identifiers of the direct members of this module.
            let module_content_identifiers = module
                .contents()
                .iter()
                .map(|def| def.borrow().raw_identifier());
            // Add the identifiers to the map, with the module's scoped identifier as the key.
            merged_module_content_identifiers
                .entry(scoped_module_identifier)
                .or_insert(Vec::new()) // If an entry doesn't exist for the key, create one now.
                .extend(module_content_identifiers); // Add this module's identifiers to the existing vector.
        }
    }
    for identifiers in merged_module_content_identifiers.into_values() {
        check_for_redefinition(identifiers, &mut data.diagnostic_reporter);
    }

    data.into()
}

struct ValidatorVisitor<'a> {
    ast: &'a Ast,
    diagnostic_reporter: &'a mut DiagnosticReporter,
    validation_functions: Vec<Validator>,
}

impl<'a> ValidatorVisitor<'a> {
    pub fn new(ast: &'a Ast, diagnostic_reporter: &'a mut DiagnosticReporter) -> Self {
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
            ast,
            diagnostic_reporter,
            validation_functions,
        }
    }

    fn validate(&mut self, func: impl Fn(&Validator, &Ast, &mut DiagnosticReporter)) {
        for validator_function in &self.validation_functions {
            func(validator_function, self.ast, self.diagnostic_reporter);
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
        self.validate(|validator, ast, diagnostic_reporter| match validator {
            Validator::Attributes(function) => function(class, diagnostic_reporter),
            Validator::Dictionaries(function) => function(&container_dictionaries(class), diagnostic_reporter),
            Validator::DocComments(function) => function(class, ast, diagnostic_reporter),
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
        self.validate(|validator, ast, diagnostic_reporter| match validator {
            Validator::Attributes(function) => function(enum_def, diagnostic_reporter),
            Validator::DocComments(function) => function(enum_def, ast, diagnostic_reporter),
            Validator::Entities(function) => function(enum_def, diagnostic_reporter),
            Validator::Enums(function) => function(enum_def, diagnostic_reporter),
            _ => {}
        });
    }

    fn visit_exception_start(&mut self, exception: &Exception) {
        self.validate(|validator, ast, diagnostic_reporter| match validator {
            Validator::Attributes(function) => function(exception, diagnostic_reporter),
            Validator::Dictionaries(function) => function(&container_dictionaries(exception), diagnostic_reporter),
            Validator::DocComments(function) => function(exception, ast, diagnostic_reporter),
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
        self.validate(|validator, ast, diagnostic_reporter| match validator {
            Validator::Attributes(function) => function(interface, diagnostic_reporter),
            Validator::DocComments(function) => function(interface, ast, diagnostic_reporter),
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
        self.validate(|validator, ast, diagnostic_reporter| match validator {
            Validator::DocComments(function) => function(module_def, ast, diagnostic_reporter),
            Validator::Entities(function) => function(module_def, diagnostic_reporter),
            // Checking for redefinition errors is done in `validate_parsed_data` to handle reopened modules.
            _ => {}
        });
    }

    fn visit_operation_start(&mut self, operation: &Operation) {
        self.validate(|validator, ast, diagnostic_reporter| match validator {
            Validator::Attributes(function) => function(operation, diagnostic_reporter),
            Validator::Dictionaries(function) => {
                function(&member_dictionaries(operation.parameters()), diagnostic_reporter);
                function(&member_dictionaries(operation.return_members()), diagnostic_reporter);
            }
            Validator::DocComments(function) => function(operation, ast, diagnostic_reporter),
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
        self.validate(|validator, _ast, diagnostic_reporter| {
            if let Validator::Attributes(function) = validator {
                function(parameter, diagnostic_reporter)
            }
        })
    }

    fn visit_struct_start(&mut self, struct_def: &Struct) {
        self.validate(|validator, ast, diagnostic_reporter| match validator {
            Validator::Attributes(function) => function(struct_def, diagnostic_reporter),
            Validator::Dictionaries(function) => function(&container_dictionaries(struct_def), diagnostic_reporter),
            Validator::DocComments(function) => function(struct_def, ast, diagnostic_reporter),
            Validator::Entities(function) => function(struct_def, diagnostic_reporter),
            Validator::Identifiers(function) => function(struct_def.members().get_identifiers(), diagnostic_reporter),
            Validator::Members(function) => function(struct_def.members().as_member_vec(), diagnostic_reporter),
            Validator::Struct(function) => function(struct_def, diagnostic_reporter),
            _ => {}
        });
    }

    fn visit_type_alias(&mut self, type_alias: &TypeAlias) {
        self.validate(|validator, ast, diagnostic_reporter| match validator {
            Validator::Dictionaries(function) => {
                if let Types::Dictionary(dictionary) = type_alias.underlying.concrete_type() {
                    function(&[dictionary], diagnostic_reporter)
                }
            }
            Validator::DocComments(function) => function(type_alias, ast, diagnostic_reporter),
            Validator::Entities(function) => function(type_alias, diagnostic_reporter),
            _ => {}
        });
    }
}
