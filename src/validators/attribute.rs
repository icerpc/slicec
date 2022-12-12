// Copyright (c) ZeroC, Inc. All rights reserved.

use super::ValidatorVisitor;
use crate::diagnostics::{Error, ErrorKind};
use crate::grammar::*;

impl ValidatorVisitor<'_> {
/// Validates that the `deprecated` attribute cannot be applied to parameters.
pub(super) fn cannot_be_deprecated(&mut self, parameters: &[&Parameter]) {
    parameters.iter().for_each(|m| {
        if m.attributes(false)
            .iter()
            .any(|a| matches!(a.kind, AttributeKind::Deprecated { .. }))
        {
            Error::new(ErrorKind::DeprecatedAttributeCannotBeApplied(
                m.kind().to_owned() + "(s)",
            ))
            .set_span(m.span())
            .report(self.diagnostic_reporter)
        };
    });
}

/// Validates that the `compress` attribute is not on an disallowed Attributable Elements and
/// verifies that the user did not provide invalid arguments.
pub(super) fn is_compressible(&mut self, element: &dyn Entity) {
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
                .report(self.diagnostic_reporter);
        }
    }
}
}
