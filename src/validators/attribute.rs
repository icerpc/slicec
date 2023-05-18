// Copyright (c) ZeroC, Inc.

use crate::diagnostics::{Diagnostic, DiagnosticReporter, Error};
use crate::grammar::*;
use crate::slice_file::SliceFile;
use crate::visitor::Visitor;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::collections::HashMap;

/// Validates that attributes are used on the correct Slice types.
pub struct AttributeValidator<'a> {
    diagnostic_reporter: &'a mut DiagnosticReporter,
}

impl<'a> AttributeValidator<'a> {
    pub fn new(diagnostic_reporter: &'a mut DiagnosticReporter) -> Self {
        Self { diagnostic_reporter }
    }
}

impl Visitor for AttributeValidator<'_> {
    fn visit_file(&mut self, slice_file: &SliceFile) {
        let attributes = slice_file.attributes(false);
        validate_repeated_attributes(&attributes, self.diagnostic_reporter);
        for attribute in attributes {
            validate_common_attributes(attribute, self.diagnostic_reporter);
        }
    }

    fn visit_module(&mut self, module_def: &Module) {
        let attributes = module_def.attributes(false);
        validate_repeated_attributes(&attributes, self.diagnostic_reporter);
        for attribute in attributes {
            validate_common_attributes(attribute, self.diagnostic_reporter);
        }
    }

    fn visit_struct(&mut self, struct_def: &Struct) {
        let attributes = struct_def.attributes(false);
        validate_repeated_attributes(&attributes, self.diagnostic_reporter);
        for attribute in attributes {
            validate_common_attributes(attribute, self.diagnostic_reporter);
        }
    }

    fn visit_class(&mut self, class_def: &Class) {
        let attributes = class_def.attributes(false);
        validate_repeated_attributes(&attributes, self.diagnostic_reporter);
        for attribute in attributes {
            validate_common_attributes(attribute, self.diagnostic_reporter);
        }
    }

    fn visit_exception(&mut self, exception_def: &Exception) {
        let attributes = exception_def.attributes(false);
        validate_repeated_attributes(&attributes, self.diagnostic_reporter);
        for attribute in attributes {
            validate_common_attributes(attribute, self.diagnostic_reporter);
        }
    }

    fn visit_interface(&mut self, interface_def: &Interface) {
        let attributes = interface_def.attributes(false);
        validate_repeated_attributes(&attributes, self.diagnostic_reporter);
        for attribute in attributes {
            match attribute.kind {
                AttributeKind::Compress { .. } => {}
                _ => validate_common_attributes(attribute, self.diagnostic_reporter),
            }
        }
    }

    fn visit_enum(&mut self, enum_def: &Enum) {
        let attributes = enum_def.attributes(false);
        validate_repeated_attributes(&attributes, self.diagnostic_reporter);
        for attribute in attributes {
            validate_common_attributes(attribute, self.diagnostic_reporter);
        }
    }

    fn visit_operation(&mut self, operation: &Operation) {
        let attributes = operation.attributes(false);
        validate_repeated_attributes(&attributes, self.diagnostic_reporter);
        for attribute in attributes {
            match attribute.kind {
                AttributeKind::Compress { .. } => {}
                AttributeKind::Oneway { .. } => {}
                AttributeKind::SlicedFormat { .. } => {}
                _ => validate_common_attributes(attribute, self.diagnostic_reporter),
            }
        }
    }

    fn visit_custom_type(&mut self, custom_type: &CustomType) {
        let attributes = custom_type.attributes(false);
        validate_repeated_attributes(&attributes, self.diagnostic_reporter);
        for attribute in attributes {
            validate_common_attributes(attribute, self.diagnostic_reporter);
        }
    }

    fn visit_type_alias(&mut self, type_alias: &TypeAlias) {
        let attributes = type_alias.attributes(false);
        validate_repeated_attributes(&attributes, self.diagnostic_reporter);
        for attribute in attributes {
            validate_common_attributes(attribute, self.diagnostic_reporter);
        }
    }

    fn visit_field(&mut self, field: &Field) {
        let attributes = field.attributes(false);
        validate_repeated_attributes(&attributes, self.diagnostic_reporter);
        for attribute in attributes {
            validate_common_attributes(attribute, self.diagnostic_reporter);
        }
    }

    fn visit_parameter(&mut self, parameter: &Parameter) {
        let attributes = parameter.attributes(false);
        validate_repeated_attributes(&attributes, self.diagnostic_reporter);
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

    fn visit_enumerator(&mut self, enumerator: &Enumerator) {
        let attributes = enumerator.attributes(false);
        validate_repeated_attributes(&attributes, self.diagnostic_reporter);
        for attribute in attributes {
            validate_common_attributes(attribute, self.diagnostic_reporter);
        }
    }

    fn visit_type_ref(&mut self, type_ref: &TypeRef) {
        let attributes = type_ref.attributes(false);
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
        AttributeKind::Oneway { .. } => Some("the oneway attribute can only be applied to operations"),
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
    match &attribute.kind {
        AttributeKind::Allow { .. } => {}
        AttributeKind::Deprecated { .. } => {}
        // Validated by the language code generator.
        AttributeKind::LanguageKind { .. } => {}
        // Allow other language attributes (directives that contain "::" ) through.
        // This is a sufficient check since the compiler rejects `::`, `x::`, and `::x` as invalid identifiers.
        AttributeKind::Other { directive, .. } if directive.contains("::") => {}
        _ => report_unexpected_attribute(attribute, diagnostic_reporter),
    }
}
