// Copyright (c) ZeroC, Inc.

use crate::diagnostics::DiagnosticReporter;
use crate::grammar::*;
use crate::validators::dictionary::{has_allowed_key_type, has_allowed_value_type};
use crate::validators::{ValidationChain, Validator};

pub fn sequence_validators() -> ValidationChain {
    vec![Validator::Sequences(has_allowed_contents)]
}

fn has_allowed_contents(sequences: &[&Sequence], diagnostic_reporter: &mut DiagnosticReporter) {
    sequences.iter().for_each(|s| validate_contents(s, diagnostic_reporter));
}

fn validate_contents(sequence: &Sequence, diagnostic_reporter: &mut DiagnosticReporter) {
    match sequence.element_type.concrete_type() {
        Types::Dictionary(dictionary) => {
            has_allowed_key_type(&[dictionary], diagnostic_reporter);
            has_allowed_value_type(&[dictionary], diagnostic_reporter);
        }
        Types::Sequence(sequence) => validate_contents(sequence, diagnostic_reporter),
        _ => {}
    }
}
