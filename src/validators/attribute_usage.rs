// Copyright (c) ZeroC, Inc.

use crate::diagnostics::{Diagnostic, DiagnosticReporter, Error};
use crate::grammar::r#struct::Struct;
use crate::grammar::{Attributable, Attribute, AttributeKind};
use crate::slice_file::SliceFile;
use crate::visitor::Visitor;

/// Validates that attributes are used on the correct Slice types
pub struct AttributeUsageValidator<'a> {
    pub diagnostic_reporter: &'a mut DiagnosticReporter,
}

fn report_unexpected_attribute(attribute: &Attribute, diagnostic_reporter: &mut DiagnosticReporter) {
    Diagnostic::new(Error::UnexpectedAttribute {
        attribute: attribute.directive().to_owned(),
    })
    .set_span(&attribute.span)
    .report(diagnostic_reporter);
}

fn validate_common_attributes(attribute: &Attribute, diagnostic_reporter: &mut DiagnosticReporter) {
    match attribute.kind {
        AttributeKind::Allow { .. } => {}
        AttributeKind::Deprecated { .. } => {}
        AttributeKind::LanguageKind { .. } => {}
        AttributeKind::Other { .. } => {}
        _ => report_unexpected_attribute(attribute, diagnostic_reporter),
    }
}

impl Visitor for AttributeUsageValidator<'_> {
    fn visit_file_start(&mut self, slice_file: &SliceFile) {}

    fn visit_struct_start(&mut self, struct_def: &Struct) {
        for attribute in &struct_def.attributes(false) {
            match attribute {
                _ => validate_common_attributes(attribute, self.diagnostic_reporter),
            }
        }
    }
}
