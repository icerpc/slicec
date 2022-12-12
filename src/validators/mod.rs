// Copyright (c) ZeroC, Inc. All rights reserved.

mod attribute;
mod comments;
mod cycle_detection;
mod dictionary;
mod enums;
mod identifiers;
mod miscellaneous;
mod tag;

use crate::ast::Ast;
use crate::compilation_result::{CompilationData, CompilationResult};
use crate::diagnostics::DiagnosticReporter;
use crate::grammar::*;
use crate::slice_file::SliceFile;
use crate::utils::ptr_util::WeakPtr;
use crate::visitor::Visitor;
use std::collections::HashMap;

pub(crate) fn validate_compilation_data(mut data: CompilationData) -> CompilationResult {
    let diagnostic_reporter = &mut data.diagnostic_reporter;

    // Update the diagnostic reporter with the slice files that contain the file level ignoreWarnings attribute.
    diagnostic_reporter.file_level_ignored_warnings = file_ignored_warnings_map(&data.files);

    // Check for any cyclic data structures. If any exist, exit early to avoid infinite loops during validation.
    cycle_detection::detect_cycles(&data.files, diagnostic_reporter);
    if diagnostic_reporter.has_errors() {
        return data.into();
    }

    let mut validator = ValidatorVisitor::new(&data.ast, diagnostic_reporter);
    for slice_file in data.files.values() {
        slice_file.visit_with(&mut validator);
    }

    // We always return `Ok` here to ensure the language mapping's validation logic is run,
    // instead of terminating early if this validator found any errors.
    Ok(data)
}

// Returns a HashMap where the keys are the relative paths of the .slice files that have the file level
// `ignoreWarnings` attribute and the values are the arguments of the attribute.
fn file_ignored_warnings_map(files: &HashMap<String, SliceFile>) -> HashMap<String, Vec<String>> {
    files
        .iter()
        .filter_map(|(path, file)| {
            file.attributes.iter().find_map(|attr| match &attr.kind {
                AttributeKind::IgnoreWarnings { warning_codes } => {
                    Some((path.clone(), warning_codes.clone().unwrap_or_default()))
                }
                _ => None,
            })
        })
        .collect()
}

struct ValidatorVisitor<'a> {
    ast: &'a Ast,
    diagnostic_reporter: &'a mut DiagnosticReporter,
}

impl<'a> ValidatorVisitor<'a> {
    pub fn new(ast: &'a Ast, diagnostic_reporter: &'a mut DiagnosticReporter) -> Self {
        ValidatorVisitor {
            ast,
            diagnostic_reporter,
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
        self.is_compressible(class);
        self.has_allowed_key_type(&container_dictionaries(class));
        self.linked_identifiers_exist(class);
        self.only_operations_can_throw(class);
        self.check_for_redefinition(class.members().get_identifiers());
        self.check_for_shadowing(
            class.members().get_identifiers(),
            class.all_inherited_members().get_identifiers(),
        );
        self.validate_member_tags(class.members().as_member_vec());
    }

    fn visit_enum_start(&mut self, enum_def: &Enum) {
        self.is_compressible(enum_def);
        self.linked_identifiers_exist(enum_def);
        self.only_operations_can_throw(enum_def);
        self.backing_type_bounds(enum_def);
        self.allowed_underlying_types(enum_def);
        self.enumerator_values_are_unique(enum_def);
        self.underlying_type_cannot_be_optional(enum_def);
        self.nonempty_if_checked(enum_def);
    }

    fn visit_exception_start(&mut self, exception: &Exception) {
        self.is_compressible(exception);
        self.has_allowed_key_type(&container_dictionaries(exception));
        self.linked_identifiers_exist(exception);
        self.only_operations_can_throw(exception);
        self.check_for_redefinition(exception.members().get_identifiers());
        self.check_for_shadowing(
            exception.members().get_identifiers(),
            exception.all_inherited_members().get_identifiers(),
        );
        self.validate_member_tags(exception.members().as_member_vec());
    }

    fn visit_interface_start(&mut self, interface: &Interface) {
        self.is_compressible(interface);
        self.linked_identifiers_exist(interface);
        self.only_operations_can_throw(interface);
        self.check_for_redefinition(interface.operations().get_identifiers());
        self.check_for_shadowing(
            interface.operations().get_identifiers(),
            interface.all_inherited_operations().get_identifiers(),
        );
    }

    fn visit_module_start(&mut self, module_def: &Module) {
        self.linked_identifiers_exist(module_def);
        self.only_operations_can_throw(module_def);
        self.check_for_redefinition(
            module_def
                .contents()
                .iter()
                .map(|definition| definition.borrow().raw_identifier())
                .collect::<_>()
        );
        self.file_scoped_modules_cannot_contain_sub_modules(module_def);
    }

    fn visit_operation_start(&mut self, operation: &Operation) {
        self.is_compressible(operation);
        self.linked_identifiers_exist(operation);
        self.only_operations_can_throw(operation);
        self.non_empty_return_comment(operation);
        self.missing_parameter_comment(operation);

        for members in [operation.parameters(), operation.return_members()] {
            self.cannot_be_deprecated(members.as_slice()); // TODOAUSTIN do we need as_slice?
            self.stream_parameter_is_last(members.as_slice());
            self.at_most_one_stream_parameter(members.as_slice());
            self.parameter_order(members.as_slice());
            self.validate_member_tags(members.as_member_vec());
            self.has_allowed_key_type(&member_dictionaries(members));
        }
    }

    fn visit_parameter(&mut self, parameter: &Parameter) {
        self.is_compressible(parameter);
    }

    fn visit_struct_start(&mut self, struct_def: &Struct) {
        self.is_compressible(struct_def);
        self.has_allowed_key_type(&container_dictionaries(struct_def));
        self.linked_identifiers_exist(struct_def);
        self.only_operations_can_throw(struct_def);
        self.check_for_redefinition(struct_def.members().get_identifiers());
        self.validate_compact_struct_not_empty(struct_def);
        self.compact_structs_cannot_contain_tags(struct_def);
        self.validate_member_tags(struct_def.members().as_member_vec());
    }

    fn visit_type_alias(&mut self, type_alias: &TypeAlias) {
        if let Types::Dictionary(dictionary) = type_alias.underlying.concrete_type() {
            self.has_allowed_key_type(&[dictionary]);
        }
        self.linked_identifiers_exist(type_alias);
        self.only_operations_can_throw(type_alias);
    }
}
