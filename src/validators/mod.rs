// Copyright (c) ZeroC, Inc.

mod attribute;
mod comments;
mod cycle_detection;
mod dictionary;
mod enums;
mod identifiers;
mod members;
mod operations;
mod parameters;
mod structs;
mod type_aliases;

use crate::compilation_state::CompilationState;
use crate::diagnostics::Diagnostics;
use crate::grammar::*;
use crate::slice_file::SliceFile;
use crate::visitor::Visitor;

use attribute::validate_attributes;
use comments::validate_common_doc_comments;
use dictionary::validate_dictionary;
use enums::validate_enum;
use identifiers::validate_inherited_identifiers;
use members::validate_members;
use operations::validate_operation;
use parameters::validate_parameters;
use structs::validate_struct;
use type_aliases::validate_type_alias;

pub(crate) fn validate_ast(compilation_state: &mut CompilationState) {
    let diagnostics = &mut compilation_state.diagnostics;

    // Check for any cyclic data structures. If any exist, exit early to avoid infinite loops during validation.
    cycle_detection::detect_cycles(&compilation_state.ast, diagnostics);
    if diagnostics.has_errors() {
        return;
    }

    // Check for any redefinitions. If any exist, exit early to avoid errors caused by looking at incorrect definitions.
    identifiers::check_for_redefinitions(&compilation_state.ast, diagnostics);
    if diagnostics.has_errors() {
        return;
    }

    let mut validator = ValidatorVisitor::new(diagnostics);
    for slice_file in &compilation_state.files {
        slice_file.visit_with(&mut validator);
    }
}

struct ValidatorVisitor<'a> {
    diagnostics: &'a mut Diagnostics,
}

impl<'a> ValidatorVisitor<'a> {
    pub fn new(diagnostics: &'a mut Diagnostics) -> Self {
        ValidatorVisitor { diagnostics }
    }
}

impl<'a> Visitor for ValidatorVisitor<'a> {
    fn visit_file(&mut self, slice_file: &SliceFile) {
        validate_attributes(slice_file, self.diagnostics);
    }

    fn visit_module(&mut self, module_def: &Module) {
        validate_attributes(module_def, self.diagnostics);
    }

    fn visit_class(&mut self, class: &Class) {
        validate_common_doc_comments(class, self.diagnostics);
        validate_attributes(class, self.diagnostics);

        validate_members(class.fields(), self.diagnostics);

        validate_inherited_identifiers(class.fields(), class.all_inherited_fields(), self.diagnostics);
    }

    fn visit_enum(&mut self, enum_def: &Enum) {
        validate_common_doc_comments(enum_def, self.diagnostics);
        validate_attributes(enum_def, self.diagnostics);

        validate_enum(enum_def, self.diagnostics);
    }

    fn visit_custom_type(&mut self, custom_type: &CustomType) {
        validate_common_doc_comments(custom_type, self.diagnostics);
        validate_attributes(custom_type, self.diagnostics);
    }

    fn visit_enumerator(&mut self, enumerator: &Enumerator) {
        validate_common_doc_comments(enumerator, self.diagnostics);
        validate_attributes(enumerator, self.diagnostics);

        validate_members(enumerator.contents(), self.diagnostics);
    }

    fn visit_exception(&mut self, exception: &Exception) {
        validate_common_doc_comments(exception, self.diagnostics);
        validate_attributes(exception, self.diagnostics);

        validate_members(exception.fields(), self.diagnostics);

        validate_inherited_identifiers(exception.fields(), exception.all_inherited_fields(), self.diagnostics);
    }

    fn visit_interface(&mut self, interface: &Interface) {
        validate_common_doc_comments(interface, self.diagnostics);
        validate_attributes(interface, self.diagnostics);

        validate_inherited_identifiers(
            interface.operations(),
            interface.all_inherited_operations(),
            self.diagnostics,
        );
    }

    fn visit_operation(&mut self, operation: &Operation) {
        validate_common_doc_comments(operation, self.diagnostics);
        validate_attributes(operation, self.diagnostics);

        validate_operation(operation, self.diagnostics);

        validate_members(operation.parameters(), self.diagnostics);
        validate_members(operation.return_members(), self.diagnostics);

        validate_parameters(&operation.parameters(), self.diagnostics);
        validate_parameters(&operation.return_members(), self.diagnostics);
    }

    fn visit_parameter(&mut self, parameter: &Parameter) {
        validate_attributes(parameter, self.diagnostics);
    }

    fn visit_struct(&mut self, struct_def: &Struct) {
        validate_common_doc_comments(struct_def, self.diagnostics);
        validate_attributes(struct_def, self.diagnostics);

        validate_struct(struct_def, self.diagnostics);

        validate_members(struct_def.fields(), self.diagnostics);
    }

    fn visit_field(&mut self, field: &Field) {
        validate_common_doc_comments(field, self.diagnostics);
        validate_attributes(field, self.diagnostics);
    }

    fn visit_type_alias(&mut self, type_alias: &TypeAlias) {
        validate_common_doc_comments(type_alias, self.diagnostics);
        validate_attributes(type_alias, self.diagnostics);

        validate_type_alias(type_alias, self.diagnostics);
    }

    fn visit_type_ref(&mut self, type_ref: &TypeRef) {
        validate_attributes(type_ref, self.diagnostics);

        if let Types::Dictionary(dictionary) = type_ref.concrete_type() {
            validate_dictionary(dictionary, self.diagnostics);
        }
    }
}
