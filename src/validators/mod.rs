// Copyright (c) ZeroC, Inc.

mod attribute;
mod comments;
mod cycle_detection;
mod dictionary;
mod enums;
mod identifiers;
mod members;
mod modules;
mod operations;
mod parameters;
mod sequence;
mod structs;
mod type_aliases;

use crate::compilation_state::CompilationState;
use crate::diagnostics::DiagnosticReporter;
use crate::grammar::*;
use crate::visitor::Visitor;

use comments::validate_common_doc_comments;
use enums::validate_enum;
use identifiers::{validate_identifiers, validate_inherited_identifiers};
use members::validate_members;
use modules::{validate_module, validate_module_contents};
use operations::validate_operation;
use parameters::validate_parameters;
use structs::validate_struct;
use type_aliases::validate_type_alias;

pub(crate) fn validate_ast(compilation_state: &mut CompilationState) {
    let diagnostic_reporter = &mut compilation_state.diagnostic_reporter;

    // Check for any cyclic data structures. If any exist, exit early to avoid infinite loops during validation.
    cycle_detection::detect_cycles(&compilation_state.ast, diagnostic_reporter);
    if diagnostic_reporter.has_errors() {
        return;
    }

    let mut validator = ValidatorVisitor::new(diagnostic_reporter);
    for slice_file in compilation_state.files.values() {
        slice_file.visit_with(&mut validator);
    }

    let mut attribute_validator = attribute::AttributeValidator::new(diagnostic_reporter);
    for slice_file in compilation_state.files.values() {
        slice_file.visit_with(&mut attribute_validator);
    }

    validate_module_contents(compilation_state);
}

fn validate_type_ref(type_ref: &TypeRef, diagnostic_reporter: &mut DiagnosticReporter) {
    match type_ref.concrete_type() {
        Types::Dictionary(dictionary) => dictionary::validate(dictionary, diagnostic_reporter),
        Types::Sequence(sequence) => sequence::validate(sequence, diagnostic_reporter),
        _ => {}
    }
}

struct ValidatorVisitor<'a> {
    diagnostic_reporter: &'a mut DiagnosticReporter,
}

impl<'a> ValidatorVisitor<'a> {
    pub fn new(diagnostic_reporter: &'a mut DiagnosticReporter) -> Self {
        ValidatorVisitor { diagnostic_reporter }
    }
}

impl<'a> Visitor for ValidatorVisitor<'a> {
    fn visit_file(&mut self, _: &crate::slice_file::SliceFile) {}

    fn visit_module(&mut self, module_def: &Module) {
        validate_module(module_def, self.diagnostic_reporter);
    }

    fn visit_class(&mut self, class: &Class) {
        validate_common_doc_comments(class, self.diagnostic_reporter);
        validate_members(class.fields().as_member_vec(), self.diagnostic_reporter);

        validate_identifiers(class.fields().get_identifiers(), self.diagnostic_reporter);
        validate_inherited_identifiers(
            class.fields().get_identifiers(),
            class.all_inherited_fields().get_identifiers(),
            self.diagnostic_reporter,
        );
    }

    fn visit_enum(&mut self, enum_def: &Enum) {
        validate_enum(enum_def, self.diagnostic_reporter);
        validate_common_doc_comments(enum_def, self.diagnostic_reporter);
        validate_identifiers(enum_def.enumerators().get_identifiers(), self.diagnostic_reporter);
    }

    fn visit_custom_type(&mut self, custom_type: &CustomType) {
        validate_common_doc_comments(custom_type, self.diagnostic_reporter);
    }

    fn visit_enumerator(&mut self, enumerator: &Enumerator) {
        validate_common_doc_comments(enumerator, self.diagnostic_reporter);
    }

    fn visit_exception(&mut self, exception: &Exception) {
        validate_common_doc_comments(exception, self.diagnostic_reporter);
        validate_members(exception.fields().as_member_vec(), self.diagnostic_reporter);

        validate_identifiers(exception.fields().get_identifiers(), self.diagnostic_reporter);
        validate_inherited_identifiers(
            exception.fields().get_identifiers(),
            exception.all_inherited_fields().get_identifiers(),
            self.diagnostic_reporter,
        );
    }

    fn visit_interface(&mut self, interface: &Interface) {
        validate_common_doc_comments(interface, self.diagnostic_reporter);

        validate_identifiers(interface.operations().get_identifiers(), self.diagnostic_reporter);
        validate_inherited_identifiers(
            interface.operations().get_identifiers(),
            interface.all_inherited_operations().get_identifiers(),
            self.diagnostic_reporter,
        );
    }

    fn visit_operation(&mut self, operation: &Operation) {
        validate_common_doc_comments(operation, self.diagnostic_reporter);
        validate_operation(operation, self.diagnostic_reporter);

        validate_members(operation.parameters().as_member_vec(), self.diagnostic_reporter);
        validate_members(operation.return_members().as_member_vec(), self.diagnostic_reporter);

        validate_parameters(&operation.parameters(), self.diagnostic_reporter);
        validate_parameters(&operation.return_members(), self.diagnostic_reporter);

        validate_identifiers(operation.parameters().get_identifiers(), self.diagnostic_reporter);
        validate_identifiers(operation.return_members().get_identifiers(), self.diagnostic_reporter);
    }

    fn visit_parameter(&mut self, parameter: &Parameter) {
        validate_type_ref(&parameter.data_type, self.diagnostic_reporter);
    }

    fn visit_struct(&mut self, struct_def: &Struct) {
        validate_common_doc_comments(struct_def, self.diagnostic_reporter);

        validate_struct(struct_def, self.diagnostic_reporter);

        validate_members(struct_def.fields().as_member_vec(), self.diagnostic_reporter);
        validate_identifiers(struct_def.fields().get_identifiers(), self.diagnostic_reporter);
    }

    fn visit_field(&mut self, field: &Field) {
        validate_common_doc_comments(field, self.diagnostic_reporter);
        validate_type_ref(&field.data_type, self.diagnostic_reporter);
    }

    fn visit_type_alias(&mut self, type_alias: &TypeAlias) {
        validate_common_doc_comments(type_alias, self.diagnostic_reporter);
        validate_type_ref(&type_alias.underlying, self.diagnostic_reporter);
        validate_type_alias(type_alias, self.diagnostic_reporter);
    }

    fn visit_type_ref(&mut self, _: &TypeRef) {
        // TO Joe,
        // FROM Austin.
    }
}

// Helper extensions to make validation easier.
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
