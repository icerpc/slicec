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
        if m.attributes(false)
            .iter()
            .any(|a| matches!(a.kind, AttributeKind::Deprecated { .. }))
        {
            Error::new(ErrorKind::DeprecatedAttributeCannotBeApplied {
                kind: m.kind().to_owned() + "(s)",
            })
            .set_span(m.span())
            .report(diagnostic_reporter)
        };
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
        if let Some(attribute) = element
            .attributes(false)
            .into_iter()
            .find(|a| matches!(a.kind, AttributeKind::Compress { .. }))
        {
            Error::new(ErrorKind::CompressAttributeCannotBeApplied)
                .set_span(attribute.span())
                .report(diagnostic_reporter);
        }
    }
}
