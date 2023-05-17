// Copyright (c) ZeroC, Inc.

use crate::diagnostics::{Diagnostic, DiagnosticReporter, Error};
use crate::grammar::*;

pub fn validate_struct(struct_def: &Struct, diagnostic_reporter: &mut DiagnosticReporter) {
    validate_compact_struct_not_empty(struct_def, diagnostic_reporter);
    compact_structs_cannot_contain_tags(struct_def, diagnostic_reporter);
}
fn validate_compact_struct_not_empty(struct_def: &Struct, diagnostic_reporter: &mut DiagnosticReporter) {
    // Compact structs must be non-empty.
    if struct_def.is_compact && struct_def.fields().is_empty() {
        Diagnostic::new(Error::CompactStructCannotBeEmpty)
            .set_span(struct_def.span())
            .report(diagnostic_reporter);
    }
}

/// Validate that tags cannot be used in compact structs.
fn compact_structs_cannot_contain_tags(struct_def: &Struct, diagnostic_reporter: &mut DiagnosticReporter) {
    if struct_def.is_compact {
        for field in struct_def.fields() {
            if field.is_tagged() {
                Diagnostic::new(Error::CompactStructCannotContainTaggedFields)
                    .set_span(field.span())
                    .add_note(
                        format!("struct '{}' is declared compact here", struct_def.identifier()),
                        Some(struct_def.span()),
                    )
                    .report(diagnostic_reporter);
            }
        }
    }
}
