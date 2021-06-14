// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::ref_from_node;
use crate::ast::{Ast, Node};
use crate::error::{Error, ErrorHandler};
use crate::grammar::*;
use crate::visitor::Visitor;
use std::collections::HashMap;

/// Validator visits all the elements in a slice file to check for additional errors and warnings
/// not caught by previous phases of parsing and that are common to all slice compilers.
#[derive(Debug)]
pub(crate) struct Validator<'a> {
    /// Reference to the parser's error handler,
    error_handler: &'a mut ErrorHandler,
}

impl<'a> Validator<'a> {
    /// Creates a new validator.
    pub(crate) fn new(error_handler: &'a mut ErrorHandler) -> Self {
        Validator { error_handler }
    }

    /// Checks the underlying type of an enum, and computes the lower and upper bounds that its
    /// enumerator's values can be between. If the enum has an allowed type, it returns the bounds
    /// as a tuple of (lower, upper), but if the enum has a disallowed type it returns an error.
    fn get_enum_bounds(enum_def: &Enum, ast: &Ast) -> Result<(i64, i64), Error> {
        // By default, if an enum doesn't have an underlying type, its bounds are the same as u32.
        let mut lower_bound: i64 = 0i64;
        let mut upper_bound: i64 = 4_294_967_295i64;

        // Check if the enum has an underlying type.
        if let Some(underlying) = &enum_def.underlying {
            let underlying_type = ast.resolve_index(underlying.definition.unwrap());
            if let Node::Primitive(_, primitive) = underlying_type {
                match primitive {
                    Primitive::Byte => {
                        lower_bound = 0i64;
                        upper_bound = 255i64;
                    }
                    Primitive::Short => {
                        lower_bound = -32_768i64;
                        upper_bound =  32_767i64;
                    }
                    Primitive::UShort => {
                        lower_bound = 0i64;
                        upper_bound = 65_535i64;
                    }
                    Primitive::Int | Primitive::VarInt => {
                        lower_bound = -2_147_483_648i64;
                        upper_bound =  2_147_483_647i64;
                    }
                    Primitive::UInt | Primitive::VarUInt => {
                        // Nothing to do, this is already the default.
                    }
                    _ => {
                        let message = format!(
                            "Type '{}' cannot be used as an enum's underlying type.\n\
                             An enum's underlying type must be byte, short, ushort, int, uint, \
                             varint, or varuint.",
                            primitive.kind(),
                        );
                        return Err((message, underlying).into());
                    }
                }
            } else {
                let message = format!(
                    "'{}'s cannot be used as an enum's underlying type.\n\
                     An enum's underlying type must be byte, short, ushort, int, uint, varint, or \
                     varuint.",
                    underlying_type.as_element().kind(),
                );
                return Err((message, underlying).into());
            }
        }
        Ok((lower_bound, upper_bound))
    }

    fn check_enumerator_value(id: usize, lower: i64, upper: i64, ast: &Ast) -> Result<(), Error> {
        let enumerator = ref_from_node!(Node::Enumerator, ast, id);
        if (enumerator.value < lower) || (enumerator.value > upper) {
            let message = format!(
                "enumerator '{}'s value ({}) is outside the range of its enum: [{}...{}]",
                enumerator.identifier(),
                enumerator.value,
                lower,
                upper,
            );
            Err((message, enumerator).into())
        } else {
            Ok(())
        }
    }
}

impl<'a> Visitor for Validator<'a> {
    fn visit_enum_start(&mut self, enum_def: &Enum, _: usize, ast: &Ast) {
        // Checks if the underlying type is valid, and gets the type's lower and upper bounds.
        match Self::get_enum_bounds(enum_def, ast) {
            Ok((lower, upper)) => {
                // Iterate through the enumerators and check if each is within bounds.
                for id in &enum_def.contents {
                    if let Err(err) = Self::check_enumerator_value(*id, lower, upper, ast) {
                        self.error_handler.report_error(err);
                    }
                }
            }

            Err(error) => {
                self.error_handler.report_error(error);
            }
        }

        // Check if any of the enumerator values are repeated.
        let mut used_values = HashMap::new();
        for id in &enum_def.contents {
            let enumerator = ref_from_node!(Node::Enumerator, ast, *id);
            if used_values.contains_key(&enumerator.value) {
                let error_message = format!(
                    "cannot reuse the value '{}' for enumerator '{}'",
                    enumerator.value,
                    enumerator.identifier(),
                );
                self.error_handler.report_error((error_message, enumerator).into());

                let original_id = *used_values.get(&enumerator.value).unwrap();
                let original = ast.resolve_index(original_id).as_named_symbol().unwrap();
                let note_message = format!(
                    "the value {} was originally used by the enumerator '{}' here",
                    enumerator.value,
                    original.identifier(),
                );
                self.error_handler.report_note((note_message, original).into());
            } else {
                used_values.insert(enumerator.value, *id);
            }
        }
    }

    fn visit_operation_start(&mut self, operation: &Operation, _: usize, _: &Ast) {
        // Check that return tuples have more than 1 element in them.
        if let ReturnType::Tuple(tuple, _) = &operation.return_type {
            if tuple.len() < 2 {
                let error_message = format!(
                    "Operation '{}' has too few elements in its return tuple.\nReturn tuples must \
                     have at least 2 elements. Consider using a single return type instead.",
                    operation.identifier(),
                );
                self.error_handler.report_error((error_message, &operation.return_type).into());
            }
        }
    }
}
