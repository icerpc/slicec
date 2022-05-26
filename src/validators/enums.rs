// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::error::ErrorReporter;
use crate::grammar::*;
use crate::visitor::Visitor;

#[derive(Debug)]
pub struct EnumValidator<'a> {
    pub error_reporter: &'a mut ErrorReporter,
    pub encoding: Encoding,
}

impl EnumValidator<'_> {
    /// Validate that the enumerators are within the bounds of the specified underlying type.
    fn backing_type_bounds(&mut self, enum_def: &Enum) {
        match self.encoding {
            Encoding::Slice1 => {
                // Slice1 does not allow negative numbers.
                enum_def
                    .enumerators()
                    .iter()
                    .filter(|enumerator| enumerator.value < 0)
                    .for_each(|enumerator| {
                        self.error_reporter.report_error(
                            format!(
                            "invalid enumerator value on enumerator `{}`: enumerators must be non-negative",
                            &enumerator.identifier()
                        ),
                            Some(enumerator.location()),
                        );
                    });
                // Enums in Slice1 always have an underlying type of int32.
                enum_def
                .enumerators()
                .iter()
                .filter(|enumerator| enumerator.value > i32::MAX as i64)
                .for_each(|enumerator| {
                    self.error_reporter.report_error(
                        format!(
                            "invalid enumerator value on enumerator `{identifier}`: must be smaller than than {max}",
                            identifier = &enumerator.identifier(),
                            max = i32::MAX,

                        ),
                        Some(enumerator.location()),
                    );
                });
            }
            Encoding::Slice2 => {
                // Non-integrals are handled by `allowed_underlying_types`
                fn check_bounds(
                    enum_def: &Enum,
                    underlying_type: &Primitive,
                    error_reporter: &mut ErrorReporter,
                ) {
                    let (min, max) = underlying_type.numeric_bounds().unwrap();
                    enum_def
                    .enumerators()
                    .iter()
                    .map(|enumerator| enumerator.value)
                    .filter(|value| *value < min || *value > max)
                    .for_each(|value| {
                        error_reporter.report_error(
                            format!(
                                "enumerator value '{value}' is out of bounds. The value must be between `{min}..{max}`, inclusive, for the underlying type `{underlying}`",
                                value = value,
                                underlying=underlying_type.kind(),
                                min = min,
                                max = max,
                            ),
                            Some(&enum_def.location),
                        );
                    });
                }

                match &enum_def.underlying {
                    Some(underlying_type) => {
                        if underlying_type.is_integral() {
                            check_bounds(enum_def, underlying_type, self.error_reporter);
                        }
                    }
                    None => {
                        // No underlying type, the default is varint32 for Slice2.
                        check_bounds(enum_def, &Primitive::VarInt32, self.error_reporter);
                    }
                }
            }
        }
    }

    /// Validate that the backing type specified for a Slice2 enums is an integral type.
    fn allowed_underlying_types(&mut self, enum_def: &Enum) {
        if self.encoding == Encoding::Slice1 {
            return;
        }
        match &enum_def.underlying {
            Some(underlying_type) => {
                if !underlying_type.is_integral() {
                    self.error_reporter.report_error(
                        format!(
                            "underlying type '{underlying}' is not allowed for enums",
                            underlying = underlying_type.definition().kind(),
                        ),
                        Some(&enum_def.location),
                    );
                }
            }
            None => (), // No underlying type, the default is varint32 for Slice2 which is integral.
        }
    }

    /// Validate that the enumerators for an enum are unique.
    fn enumerators_are_unique(&mut self, enumerators: Vec<&Enumerator>) {
        // The enumerators must be sorted by value first as we are using windowing to check the
        // n + 1 enumerator against the n enumerator. If the enumerators are sorted by value then
        // the windowing will reveal any duplicate enumerators.
        let mut sorted_enumerators = enumerators.clone();
        sorted_enumerators.sort_by_key(|m| m.value);
        sorted_enumerators.windows(2).for_each(|window| {
            if window[0].value == window[1].value {
                self.error_reporter.report_error(
                    format!(
                        "invalid enumerator value on enumerator `{}`: enumerators must be unique",
                        &window[1].identifier()
                    ),
                    Some(window[1].location()),
                );
                self.error_reporter.report_error(
                    format!(
                        "The enumerator `{}` has previous used the value `{}`",
                        &window[0].identifier(),
                        window[0].value
                    ),
                    Some(window[0].location()),
                );
            }
        })
    }

    /// Validate the the underlying type of an enum is not optional.
    fn underlying_type_cannot_be_optional(&mut self, enum_def: &Enum) {
        if let Some(ref typeref) = enum_def.underlying {
            if typeref.is_optional {
                self.error_reporter.report_error(
                        format!("underlying type '{}' cannot be optional: enums cannot have optional underlying types", typeref.type_string),
                        Some(&enum_def.location),
                    );
            }
        }
    }

    /// Validate that a checked enum must not be empty.
    fn nonempty_if_checked(&mut self, enum_def: &Enum) {
        if !enum_def.is_unchecked && enum_def.enumerators.is_empty() {
            self.error_reporter.report_error(
                "enums must contain at least one enumerator",
                Some(&enum_def.location),
            );
        }
    }
}

impl<'a> Visitor for EnumValidator<'a> {
    fn visit_enum_start(&mut self, enum_def: &Enum) {
        self.allowed_underlying_types(enum_def);
        self.backing_type_bounds(enum_def);
        self.enumerators_are_unique(enum_def.enumerators());
        self.underlying_type_cannot_be_optional(enum_def);
        self.nonempty_if_checked(enum_def);
    }
}
