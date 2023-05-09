// Copyright (c) ZeroC, Inc.

mod attribute;
mod comments;
mod cycle_detection;
mod dictionary;
mod enums;
mod identifiers;
mod miscellaneous;
mod sequence;
mod tag;

use crate::ast::node::Node;
use crate::ast::Ast;
use crate::compilation_state::CompilationState;
use crate::diagnostics::{Diagnostic, DiagnosticReporter, Error};
use crate::grammar::*;
use crate::visitor::Visitor;
use std::collections::HashMap;

pub use attribute::validate_repeated_attributes;

pub type ValidationChain = Vec<Validator>;

pub enum Validator {
    Attributes(fn(&dyn Entity, &mut DiagnosticReporter)),
    DocComments(fn(&dyn Entity, &Ast, &mut DiagnosticReporter)),
    Enums(fn(&Enum, &mut DiagnosticReporter)),
    Entities(fn(&dyn Entity, &mut DiagnosticReporter)),
    Members(fn(Vec<&dyn Member>, &mut DiagnosticReporter)),
    Module(fn(&Module, &mut DiagnosticReporter)),
    Identifiers(fn(Vec<&Identifier>, &mut DiagnosticReporter)),
    InheritedIdentifiers(fn(Vec<&Identifier>, Vec<&Identifier>, &mut DiagnosticReporter)),
    Operations(fn(&Operation, &mut DiagnosticReporter)),
    Parameters(fn(&[&Parameter], &mut DiagnosticReporter)),
    Struct(fn(&Struct, &mut DiagnosticReporter)),
    TypeAlias(fn(&TypeAlias, &mut DiagnosticReporter)),
}

pub(crate) fn validate_ast(compilation_state: &mut CompilationState) {
    let diagnostic_reporter = &mut compilation_state.diagnostic_reporter;

    // Check for any cyclic data structures. If any exist, exit early to avoid infinite loops during validation.
    cycle_detection::detect_cycles(&compilation_state.ast, diagnostic_reporter);
    if diagnostic_reporter.has_errors() {
        return;
    }

    let mut validator = ValidatorVisitor::new(&compilation_state.ast, diagnostic_reporter);
    for slice_file in compilation_state.files.values() {
        slice_file.visit_with(&mut validator);
    }

    validate_module_contents(compilation_state);
}

/// Since modules can be re-opened, but each module is a distinct entity in the AST, our normal redefinition check
/// is inadequate. If 2 modules have the same name we have to check for redefinitions across both modules.
///
/// So we compute a map of all the contents in modules with the same name (fully scoped), then check that.
fn validate_module_contents(compilation_state: &mut CompilationState) {
    let mut merged_module_contents: HashMap<String, Vec<&Definition>> = HashMap::new();
    for node in compilation_state.ast.as_slice() {
        if let Node::Module(module_ptr) = node {
            // Borrow the module's pointer and store its fully scoped identifier.
            let module = module_ptr.borrow();
            let scoped_module_identifier = module.parser_scoped_identifier();

            // Add the contents to the map, with the module's scoped identifier as the key.
            merged_module_contents
                 .entry(scoped_module_identifier)
                 .or_default() // If an entry doesn't exist for the key, create one now.
                 .extend(module.contents()); // Add this module's contents to the existing vector.
        }
    }

    for mut module_contents in merged_module_contents.into_values() {
        // Sort the contents by identifier first so that we can use windows to search for duplicates.
        module_contents.sort_by_key(|def| def.borrow().raw_identifier().value.to_owned());
        module_contents.windows(2).for_each(|window| {
            let identifier_0 = window[0].borrow().raw_identifier();
            let identifier_1 = window[1].borrow().raw_identifier();

            // We don't want to report a redefinition error if both definitions are modules, since
            // that's allowed. If both identifiers are the same and either definition is not a module, then we have a
            // redefinition error.
            if identifier_0.value == identifier_1.value
                && !(matches!(window[0], Definition::Module(_)) && matches!(window[1], Definition::Module(_)))
            {
                Diagnostic::new(Error::Redefinition {
                    identifier: identifier_1.value.clone(),
                })
                .set_span(identifier_1.span())
                .add_note(
                    format!("'{}' was previously defined here", identifier_0.value),
                    Some(identifier_0.span()),
                )
                .report(&mut compilation_state.diagnostic_reporter);
            }
        });
    }
}

fn validate_type_ref(type_ref: &TypeRef, diagnostic_reporter: &mut DiagnosticReporter) {
    match type_ref.concrete_type() {
        Types::Dictionary(dictionary) => dictionary::validate(dictionary, diagnostic_reporter),
        Types::Sequence(sequence) => sequence::validate(sequence, diagnostic_reporter),
        _ => {}
    }
}

struct ValidatorVisitor<'a> {
    ast: &'a Ast,
    diagnostic_reporter: &'a mut DiagnosticReporter,
    validation_functions: Vec<Validator>,
}

impl<'a> ValidatorVisitor<'a> {
    pub fn new(ast: &'a Ast, diagnostic_reporter: &'a mut DiagnosticReporter) -> Self {
        let validation_functions = [
            attribute::attribute_validators(),
            comments::comments_validators(),
            enums::enum_validators(),
            identifiers::identifier_validators(),
            miscellaneous::miscellaneous_validators(),
            tag::tag_validators(),
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

impl<'a> Visitor for ValidatorVisitor<'a> {
    fn visit_class_start(&mut self, class: &Class) {
        self.validate(|validator, ast, diagnostic_reporter| match validator {
            Validator::Attributes(function) => function(class, diagnostic_reporter),
            Validator::DocComments(function) => function(class, ast, diagnostic_reporter),
            Validator::Entities(function) => function(class, diagnostic_reporter),
            Validator::Identifiers(function) => function(class.fields().get_identifiers(), diagnostic_reporter),
            Validator::InheritedIdentifiers(function) => function(
                class.fields().get_identifiers(),
                class.all_inherited_fields().get_identifiers(),
                diagnostic_reporter,
            ),
            Validator::Members(function) => function(class.fields().as_member_vec(), diagnostic_reporter),
            _ => {}
        });
    }

    fn visit_enum_start(&mut self, enum_def: &Enum) {
        self.validate(|validator, ast, diagnostic_reporter| match validator {
            Validator::Attributes(function) => function(enum_def, diagnostic_reporter),
            Validator::DocComments(function) => function(enum_def, ast, diagnostic_reporter),
            Validator::Entities(function) => function(enum_def, diagnostic_reporter),
            Validator::Enums(function) => function(enum_def, diagnostic_reporter),
            Validator::Identifiers(function) => function(enum_def.enumerators().get_identifiers(), diagnostic_reporter),
            _ => {}
        });
    }

    fn visit_custom_type(&mut self, custom_type: &CustomType) {
        self.validate(|validator, _ast, diagnostic_reporter| {
            if let Validator::Attributes(function) = validator {
                function(custom_type, diagnostic_reporter)
            }
        });
    }

    fn visit_enumerator(&mut self, enumerator: &Enumerator) {
        self.validate(|validator, _ast, diagnostic_reporter| {
            if let Validator::Attributes(function) = validator {
                function(enumerator, diagnostic_reporter)
            }
        });
    }

    fn visit_exception_start(&mut self, exception: &Exception) {
        self.validate(|validator, ast, diagnostic_reporter| match validator {
            Validator::Attributes(function) => function(exception, diagnostic_reporter),
            Validator::DocComments(function) => function(exception, ast, diagnostic_reporter),
            Validator::Entities(function) => function(exception, diagnostic_reporter),
            Validator::Identifiers(function) => function(exception.fields().get_identifiers(), diagnostic_reporter),
            Validator::InheritedIdentifiers(function) => function(
                exception.fields().get_identifiers(),
                exception.all_inherited_fields().get_identifiers(),
                diagnostic_reporter,
            ),
            Validator::Members(function) => function(exception.fields().as_member_vec(), diagnostic_reporter),
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
            Validator::Attributes(function) => function(module_def, diagnostic_reporter),
            Validator::DocComments(function) => function(module_def, ast, diagnostic_reporter),
            Validator::Entities(function) => function(module_def, diagnostic_reporter),
            Validator::Module(function) => function(module_def, diagnostic_reporter),
            // Checking for redefinition errors is done in `validate_parsed_data` to handle reopened modules.
            _ => {}
        });
    }

    fn visit_operation_start(&mut self, operation: &Operation) {
        self.validate(|validator, ast, diagnostic_reporter| match validator {
            Validator::Attributes(function) => function(operation, diagnostic_reporter),
            Validator::DocComments(function) => function(operation, ast, diagnostic_reporter),
            Validator::Entities(function) => function(operation, diagnostic_reporter),
            Validator::Identifiers(function) => {
                function(operation.parameters().get_identifiers(), diagnostic_reporter);
                function(operation.return_members().get_identifiers(), diagnostic_reporter);
            }
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
        validate_type_ref(&parameter.data_type, self.diagnostic_reporter);
        self.validate(|validator, _ast, diagnostic_reporter| {
            if let Validator::Attributes(function) = validator {
                function(parameter, diagnostic_reporter)
            }
        })
    }

    fn visit_struct_start(&mut self, struct_def: &Struct) {
        self.validate(|validator, ast, diagnostic_reporter| match validator {
            Validator::Attributes(function) => function(struct_def, diagnostic_reporter),
            Validator::DocComments(function) => function(struct_def, ast, diagnostic_reporter),
            Validator::Entities(function) => function(struct_def, diagnostic_reporter),
            Validator::Identifiers(function) => function(struct_def.fields().get_identifiers(), diagnostic_reporter),
            Validator::Members(function) => function(struct_def.fields().as_member_vec(), diagnostic_reporter),
            Validator::Struct(function) => function(struct_def, diagnostic_reporter),
            _ => {}
        });
    }

    fn visit_field(&mut self, field: &Field) {
        validate_type_ref(&field.data_type, self.diagnostic_reporter);
        self.validate(|validator, _ast, diagnostic_reporter| {
            if let Validator::Attributes(function) = validator {
                function(field, diagnostic_reporter)
            }
        })
    }

    fn visit_type_alias(&mut self, type_alias: &TypeAlias) {
        validate_type_ref(&type_alias.underlying, self.diagnostic_reporter);
        self.validate(|validator, ast, diagnostic_reporter| match validator {
            Validator::Attributes(function) => function(type_alias, diagnostic_reporter),
            Validator::DocComments(function) => function(type_alias, ast, diagnostic_reporter),
            Validator::Entities(function) => function(type_alias, diagnostic_reporter),
            Validator::TypeAlias(function) => function(type_alias, diagnostic_reporter),
            _ => {}
        });
    }
}
