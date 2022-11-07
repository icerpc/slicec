// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::diagnostics::*;
use crate::grammar::*;
use crate::validators::{ValidationChain, Validator};

pub fn attribute_validators() -> ValidationChain {
    vec![
        Validator::Attributes(is_compressible),
        Validator::Parameters(cannot_be_deprecated),
    ]
}

/// Validates that the `deprecated` attribute cannot be applied to parameters.
fn cannot_be_deprecated(parameters: &[&Parameter], diagnostic_reporter: &mut DiagnosticReporter) {
    parameters.iter().for_each(|m| {
        if m.has_attribute(attributes::DEPRECATED, false) {
            let error = Error::new(
                ErrorKind::DeprecatedAttributeCannotBeApplied(m.kind().to_owned() + "(s)"),
                Some(m.span()),
            );
            diagnostic_reporter.report_error(error);
        }
    });
}

/// Validates that the `compress` attribute is not on an disallowed Attributable Elements and
/// verifies that the user did not provide invalid arguments.
fn is_compressible(element: &dyn Entity, diagnostic_reporter: &mut DiagnosticReporter) {
    // Validates that the `compress` attribute cannot be applied to anything other than
    // interfaces and operations.
    let supported_on = ["interface", "operation"];
    let kind = element.kind();
    if !supported_on.contains(&kind) {
        if let Some(attribute) = element.get_raw_attribute("compress", false) {
            diagnostic_reporter.report_error(Error::new(
                ErrorKind::CompressAttributeCannotBeApplied,
                Some(attribute.span()),
            ));
        }
    }
}
