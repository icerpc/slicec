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
mod structs;
mod type_aliases;

use crate::compilation_state::CompilationState;
use crate::diagnostics::DiagnosticReporter;
use crate::grammar::*;
use crate::slice_file::SliceFile;
use crate::visitor::Visitor;

use attribute::{
    reject_attributes, reject_known_attribute, report_unexpected_attribute, validate_attributes,
    validate_attributes_excluding, validate_attributes_including, validate_common_attribute,
    validate_repeated_attributes,
};
use comments::validate_common_doc_comments;
use dictionary::validate_dictionary;
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

    validate_module_contents(compilation_state);
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
    fn visit_file(&mut self, slice_file: &SliceFile) {
        validate_attributes!(slice_file, self.diagnostic_reporter);
    }

    fn visit_module(&mut self, module_def: &Module) {
        reject_attributes!(module_def, self.diagnostic_reporter);

        validate_module(module_def, self.diagnostic_reporter);
    }

    fn visit_class(&mut self, class: &Class) {
        validate_common_doc_comments(class, self.diagnostic_reporter);
        validate_attributes!(class, self.diagnostic_reporter);

        validate_members(class.fields(), self.diagnostic_reporter);

        validate_identifiers(class.fields(), self.diagnostic_reporter);
        validate_inherited_identifiers(class.fields(), class.all_inherited_fields(), self.diagnostic_reporter);
    }

    fn visit_enum(&mut self, enum_def: &Enum) {
        validate_common_doc_comments(enum_def, self.diagnostic_reporter);
        validate_attributes!(enum_def, self.diagnostic_reporter);

        validate_enum(enum_def, self.diagnostic_reporter);

        validate_identifiers(enum_def.enumerators(), self.diagnostic_reporter);
    }

    fn visit_custom_type(&mut self, custom_type: &CustomType) {
        validate_common_doc_comments(custom_type, self.diagnostic_reporter);
        validate_attributes!(custom_type, self.diagnostic_reporter);
    }

    fn visit_enumerator(&mut self, enumerator: &Enumerator) {
        validate_common_doc_comments(enumerator, self.diagnostic_reporter);
        validate_attributes!(enumerator, self.diagnostic_reporter);
    }

    fn visit_exception(&mut self, exception: &Exception) {
        validate_common_doc_comments(exception, self.diagnostic_reporter);
        validate_attributes!(exception, self.diagnostic_reporter);

        validate_members(exception.fields(), self.diagnostic_reporter);

        validate_identifiers(exception.fields(), self.diagnostic_reporter);
        validate_inherited_identifiers(
            exception.fields(),
            exception.all_inherited_fields(),
            self.diagnostic_reporter,
        );
    }

    fn visit_interface(&mut self, interface: &Interface) {
        validate_common_doc_comments(interface, self.diagnostic_reporter);
        validate_attributes_including!(interface, self.diagnostic_reporter, Compress);

        validate_identifiers(interface.operations(), self.diagnostic_reporter);
        validate_inherited_identifiers(
            interface.operations(),
            interface.all_inherited_operations(),
            self.diagnostic_reporter,
        );
    }

    fn visit_operation(&mut self, operation: &Operation) {
        validate_common_doc_comments(operation, self.diagnostic_reporter);

        validate_attributes_including!(operation, self.diagnostic_reporter, Compress, Oneway, SlicedFormat);

        validate_operation(operation, self.diagnostic_reporter);

        validate_members(operation.parameters(), self.diagnostic_reporter);
        validate_members(operation.return_members(), self.diagnostic_reporter);

        validate_parameters(&operation.parameters(), self.diagnostic_reporter);
        validate_parameters(&operation.return_members(), self.diagnostic_reporter);

        validate_identifiers(operation.parameters(), self.diagnostic_reporter);
        validate_identifiers(operation.return_members(), self.diagnostic_reporter);
    }

    fn visit_parameter(&mut self, parameter: &Parameter) {
        validate_attributes_excluding!(
            parameter,
            self.diagnostic_reporter,
            Deprecated,
            Some("parameters can not be individually deprecated")
        );
    }

    fn visit_struct(&mut self, struct_def: &Struct) {
        validate_common_doc_comments(struct_def, self.diagnostic_reporter);
        validate_attributes!(struct_def, self.diagnostic_reporter);

        validate_struct(struct_def, self.diagnostic_reporter);

        validate_members(struct_def.fields(), self.diagnostic_reporter);
        validate_identifiers(struct_def.fields(), self.diagnostic_reporter);
    }

    fn visit_field(&mut self, field: &Field) {
        validate_common_doc_comments(field, self.diagnostic_reporter);
        validate_attributes!(field, self.diagnostic_reporter);
    }

    fn visit_type_alias(&mut self, type_alias: &TypeAlias) {
        validate_common_doc_comments(type_alias, self.diagnostic_reporter);
        validate_attributes!(type_alias, self.diagnostic_reporter);

        validate_type_alias(type_alias, self.diagnostic_reporter);
    }

    fn visit_type_ref(&mut self, type_ref: &TypeRef) {
        reject_attributes!(type_ref, self.diagnostic_reporter);

        if let Types::Dictionary(dictionary) = type_ref.concrete_type() {
            validate_dictionary(dictionary, self.diagnostic_reporter);
        }
    }
}
