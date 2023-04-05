// Copyright (c) ZeroC, Inc.

use crate::diagnostics::DiagnosticReporter;
use crate::grammar::Sequence;

pub fn validate(sequence: &Sequence, diagnostic_reporter: &mut DiagnosticReporter) {
    super::validate_type_ref(&sequence.element_type, diagnostic_reporter);
}
