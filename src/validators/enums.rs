// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::diagnostics::*;
use crate::grammar::*;
use crate::validators::{ValidationChain, Validator};

pub fn enum_validators() -> ValidationChain {
    vec![
        Validator::Enums(backing_type_bounds),
        Validator::Enums(allowed_underlying_types),
        Validator::Enums(enumerators_are_unique),
        Validator::Enums(underlying_type_cannot_be_optional),
        Validator::Enums(nonempty_if_checked),
    ]
}

/// Validate that the enumerators are within the bounds of the specified underlying type.
fn backing_type_bounds(enum_def: &Enum, diagnostic_reporter: &mut DiagnosticReporter) {
    if enum_def.supported_encodings().supports(&Encoding::Slice1) {
        // Enum was defined in a Slice1 file.
        // Slice1 does not allow negative numbers.
        enum_def
            .enumerators()
            .iter()
            .filter(|enumerator| enumerator.value < 0)
            .for_each(|enumerator| {
                let error = LogicKind::MustBePositive("enumerator values".to_owned());
                diagnostic_reporter.report_error(Error::new(error, Some(enumerator.span())));
            });
        // Enums in Slice1 always have an underlying type of int32.
        enum_def
            .enumerators()
            .iter()
            .filter(|enumerator| enumerator.value > i32::MAX as i64)
            .for_each(|enumerator| {
                let error = LogicKind::EnumeratorValueOutOfBounds(
                    enumerator.identifier().to_owned(),
                    enumerator.value,
                    0,
                    i32::MAX as i64,
                );
                diagnostic_reporter.report_error(Error::new(error, Some(enumerator.span())));
            });
    } else {
        // Enum was defined in a Slice2 file.
        // Non-integrals are handled by `allowed_underlying_types`
        fn check_bounds(enum_def: &Enum, underlying_type: &Primitive, diagnostic_reporter: &mut DiagnosticReporter) {
            let (min, max) = underlying_type.numeric_bounds().unwrap();
            enum_def
                .enumerators()
                .iter()
                .filter(|enumerator| enumerator.value < min || enumerator.value > max)
                .for_each(|enumerator| {
                    let error = LogicKind::EnumeratorValueOutOfBounds(
                        enumerator.identifier().to_owned(),
                        enumerator.value,
                        min,
                        max,
                    );
                    diagnostic_reporter.report_error(Error::new(error, Some(enumerator.span())));
                });
        }
        match &enum_def.underlying {
            Some(underlying_type) => {
                if underlying_type.is_integral() {
                    check_bounds(enum_def, underlying_type, diagnostic_reporter);
                }
            }
            None => {
                // No underlying type, the default is varint32 for Slice2.
                check_bounds(enum_def, &Primitive::VarInt32, diagnostic_reporter);
            }
        }
    }
}

/// Validate that the backing type specified for a Slice2 enums is an integral type.
fn allowed_underlying_types(enum_def: &Enum, diagnostic_reporter: &mut DiagnosticReporter) {
    if enum_def.supported_encodings().supports(&Encoding::Slice1) {
        return;
    }
    match &enum_def.underlying {
        Some(underlying_type) => {
            if !underlying_type.is_integral() {
                let error = LogicKind::UnderlyingTypeMustBeIntegral(
                    enum_def.identifier().to_owned(),
                    underlying_type.definition().kind().to_owned(),
                );
                diagnostic_reporter.report_error(Error::new(error, Some(enum_def.span())));
            }
        }
        None => (), // No underlying type, the default is varint32 for Slice2 which is integral.
    }
}

/// Validate that the enumerators for an enum are unique.
fn enumerators_are_unique(enum_def: &Enum, diagnostic_reporter: &mut DiagnosticReporter) {
    // The enumerators must be sorted by value first as we are using windowing to check the
    // n + 1 enumerator against the n enumerator. If the enumerators are sorted by value then
    // the windowing will reveal any duplicate enumerators.
    let enumerators = enum_def.enumerators();
    let mut sorted_enumerators = enumerators.clone();
    sorted_enumerators.sort_by_key(|m| m.value);
    sorted_enumerators.windows(2).for_each(|window| {
        if window[0].value == window[1].value {
            let diagnostic = Error::new_with_notes(
                LogicKind::CannotHaveDuplicateEnumerators(window[1].identifier().to_owned()),
                Some(window[1].span()),
                vec![Note::new(
                    format!(
                        "The enumerator `{}` has previous used the value `{}`",
                        window[0].identifier(),
                        window[0].value,
                    ),
                    Some(window[0].span()),
                )],
            );
            diagnostic_reporter.report_error(diagnostic);
        }
    });
}

/// Validate the the underlying type of an enum is not optional.
fn underlying_type_cannot_be_optional(enum_def: &Enum, diagnostic_reporter: &mut DiagnosticReporter) {
    if let Some(ref typeref) = enum_def.underlying {
        if typeref.is_optional {
            diagnostic_reporter.report_error(Error::new(
                LogicKind::CannotUseOptionalUnderlyingType(enum_def.identifier().to_owned()),
                Some(enum_def.span()),
            ));
        }
    }
}

/// Validate that a checked enum must not be empty.
fn nonempty_if_checked(enum_def: &Enum, diagnostic_reporter: &mut DiagnosticReporter) {
    if !enum_def.is_unchecked && enum_def.enumerators.is_empty() {
        diagnostic_reporter.report_error(Error::new(
            LogicKind::MustContainEnumerators(enum_def.identifier().to_owned()),
            Some(enum_def.span()),
        ));
    }
}
