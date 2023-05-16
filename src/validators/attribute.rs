// Copyright (c) ZeroC, Inc.

use crate::diagnostics::{Diagnostic, DiagnosticReporter, Error};
use crate::grammar::module::Module;
use crate::grammar::r#struct::Struct;
use crate::grammar::type_ref::TypeRef;
use crate::grammar::{Attributable, Attribute, AttributeKind, Symbol};
use crate::slice_file::SliceFile;
use crate::visitor::Visitor;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::collections::HashMap;

/// Validates that attributes are used on the correct Slice types
pub struct AttributeValidator<'a> {
    pub diagnostic_reporter: &'a mut DiagnosticReporter,
}

impl Visitor for AttributeValidator<'_> {
    fn visit_file(&mut self, slice_file: &SliceFile) {
        let attributes = &slice_file.attributes(false);
        validate_repeated_attributes(attributes, self.diagnostic_reporter);
        for attribute in attributes {
            validate_common_attributes(attribute, self.diagnostic_reporter)
        }
    }

    fn visit_module(&mut self, module_def: &Module) {
        let attributes = &module_def.attributes(false);
        validate_repeated_attributes(attributes, self.diagnostic_reporter);
        for attribute in attributes {
            validate_common_attributes(attribute, self.diagnostic_reporter)
        }
    }

    fn visit_struct(&mut self, struct_def: &Struct) {
        let attributes = &struct_def.attributes(false);
        validate_repeated_attributes(attributes, self.diagnostic_reporter);
        for attribute in attributes {
            validate_common_attributes(attribute, self.diagnostic_reporter)
        }
    }

    fn visit_class(&mut self, class_def: &crate::grammar::class::Class) {
        let attributes = &class_def.attributes(false);
        validate_repeated_attributes(attributes, self.diagnostic_reporter);
        for attribute in attributes {
            validate_common_attributes(attribute, self.diagnostic_reporter)
        }
    }

    fn visit_exception(&mut self, exception_def: &crate::grammar::exception::Exception) {
        let attributes = &exception_def.attributes(false);
        validate_repeated_attributes(attributes, self.diagnostic_reporter);
        for attribute in attributes {
            validate_common_attributes(attribute, self.diagnostic_reporter)
        }
    }

    fn visit_interface(&mut self, interface_def: &crate::grammar::interface::Interface) {
        let attributes = &interface_def.attributes(false);
        validate_repeated_attributes(attributes, self.diagnostic_reporter);
        for attribute in attributes {
            match attribute.kind {
                AttributeKind::Compress { .. } => {}
                _ => validate_common_attributes(attribute, self.diagnostic_reporter),
            }
        }
    }

    fn visit_enum(&mut self, enum_def: &crate::grammar::r#enum::Enum) {
        let attributes = &enum_def.attributes(false);
        validate_repeated_attributes(attributes, self.diagnostic_reporter);
        for attribute in attributes {
            validate_common_attributes(attribute, self.diagnostic_reporter)
        }
    }

    fn visit_operation(&mut self, operation: &crate::grammar::operation::Operation) {
        let attributes = &operation.attributes(false);
        validate_repeated_attributes(attributes, self.diagnostic_reporter);
        for attribute in attributes {
            match attribute.kind {
                AttributeKind::Compress { .. } => {}
                AttributeKind::Oneway { .. } => {}
                AttributeKind::SlicedFormat { .. } => {}
                _ => validate_common_attributes(attribute, self.diagnostic_reporter),
            }
        }
    }

    fn visit_custom_type(&mut self, custom_type: &crate::grammar::custom_type::CustomType) {
        let attributes = &custom_type.attributes(false);
        validate_repeated_attributes(attributes, self.diagnostic_reporter);
        for attribute in attributes {
            validate_common_attributes(attribute, self.diagnostic_reporter)
        }
    }

    fn visit_type_alias(&mut self, type_alias: &crate::grammar::type_alias::TypeAlias) {
        let attributes = &type_alias.attributes(false);
        validate_repeated_attributes(attributes, self.diagnostic_reporter);
        for attribute in attributes {
            validate_common_attributes(attribute, self.diagnostic_reporter)
        }
    }

    fn visit_field(&mut self, field: &crate::grammar::field::Field) {
        let attributes = &field.attributes(false);
        validate_repeated_attributes(attributes, self.diagnostic_reporter);
        for attribute in attributes {
            validate_common_attributes(attribute, self.diagnostic_reporter)
        }
    }

    fn visit_parameter(&mut self, parameter: &crate::grammar::parameter::Parameter) {
        let attributes = &parameter.attributes(false);
        validate_repeated_attributes(attributes, self.diagnostic_reporter);
        for attribute in attributes {
            match attribute.kind {
                // Issue an error here since deprecated is allowed everywhere else
                AttributeKind::Deprecated { .. } => {
                    Diagnostic::new(Error::UnexpectedAttribute {
                        attribute: attribute.directive().to_owned(),
                    })
                    .set_span(attribute.span())
                    .add_note("parameters can not be individually deprecated", None)
                    .report(self.diagnostic_reporter);
                }
                _ => validate_common_attributes(attribute, self.diagnostic_reporter),
            }
        }
    }

    fn visit_enumerator(&mut self, enumerator: &crate::grammar::enumerator::Enumerator) {
        let attributes = &enumerator.attributes(false);
        validate_repeated_attributes(attributes, self.diagnostic_reporter);
        for attribute in attributes {
            validate_common_attributes(attribute, self.diagnostic_reporter)
        }
    }

    fn visit_type_ref(&mut self, type_ref: &TypeRef) {
        let attributes = &type_ref.attributes(false);
        for attribute in attributes {
            match attribute.kind {
                AttributeKind::LanguageKind { .. } => {}
                AttributeKind::Other { .. } => {}
                _ => {
                    Diagnostic::new(Error::UnexpectedAttribute {
                        attribute: attribute.directive().to_owned(),
                    })
                    .set_span(attribute.span())
                    .report(self.diagnostic_reporter);
                }
            }
        }
    }
}

/// Validates a list of attributes to ensure attributes which are not allowed to be repeated are not repeated.
pub fn validate_repeated_attributes(attributes: &[&Attribute], diagnostic_reporter: &mut DiagnosticReporter) {
    let mut first_attribute_occurrence = HashMap::new();

    for attribute in attributes {
        // We only care about attributes that are not allowed to repeat.
        if attribute.kind.is_repeatable() {
            continue;
        }

        let directive = attribute.directive();
        let span = attribute.span();

        match first_attribute_occurrence.entry(directive) {
            Occupied(entry) => {
                Diagnostic::new(Error::AttributeIsNotRepeatable {
                    attribute: directive.to_owned(),
                })
                .set_span(span)
                .add_note("attribute was previously used here", Some(entry.get()))
                .report(diagnostic_reporter);
            }
            Vacant(entry) => {
                entry.insert(span.clone());
            }
        }
    }
}

fn report_unexpected_attribute(attribute: &Attribute, diagnostic_reporter: &mut DiagnosticReporter) {
    let note = match attribute.kind {
        AttributeKind::Compress { .. } => {
            Some("the compress attribute can only be applied to interfaces and operations")
        }
        AttributeKind::SlicedFormat { .. } => Some("the slicedFormat attribute can only be applied to operations"),
        _ => None,
    };

    let mut diagnostic = Diagnostic::new(Error::UnexpectedAttribute {
        attribute: attribute.directive().to_owned(),
    })
    .set_span(&attribute.span);

    if let Some(note) = note {
        diagnostic = diagnostic.add_note(note, None);
    }

    diagnostic.report(diagnostic_reporter);
}

fn validate_common_attributes(attribute: &Attribute, diagnostic_reporter: &mut DiagnosticReporter) {
    match attribute.kind {
        AttributeKind::Allow { .. } => {}
        AttributeKind::Deprecated { .. } => {}
        AttributeKind::LanguageKind { .. } => {} // Validated by the language code generator
        AttributeKind::Other { .. } => {}        // Allow unknown attributes through.
        _ => report_unexpected_attribute(attribute, diagnostic_reporter),
    }
}
