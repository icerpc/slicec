// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::error::ErrorReporter;
use crate::errors::*;
use crate::grammar::*;
use crate::validators::{ValidationChain, Validator};

pub fn dictionary_validators() -> ValidationChain {
    vec![Validator::Dictionaries(has_allowed_key_type)]
}

pub fn has_allowed_key_type(dictionaries: &[&Dictionary], error_reporter: &mut ErrorReporter) {
    for dictionary in dictionaries {
        check_dictionary_key_type(&dictionary.key_type, error_reporter);
    }
}

fn check_dictionary_key_type(type_ref: &TypeRef, error_reporter: &mut ErrorReporter) -> bool {
    // Optional types cannot be used as dictionary keys.
    if type_ref.is_optional {
        let rule_error = RuleKind::InvalidKey(InvalidKeyKind::CannotUseOptionalAsKey);
        error_reporter.report_rule_error(rule_error, Some(type_ref.location()));
        return false;
    }

    let definition = type_ref.definition();
    let (is_valid, named_symbol): (bool, Option<&dyn NamedSymbol>) = match definition.concrete_type() {
        Types::Struct(struct_def) => {
            // Only compact structs can be used for dictionary keys.
            if !struct_def.is_compact {
                let rule_error = RuleKind::InvalidKey(InvalidKeyKind::StructsMustBeCompactToBeAKey);
                error_reporter.report_rule_error(rule_error, Some(type_ref.location()));
                error_reporter.report_note(
                    format!("struct '{}' is defined here:", struct_def.identifier()),
                    Some(struct_def.location()),
                );
                return false;
            }

            // Check that all the data members of the struct are also valid key types.
            let mut contains_invalid_key_types = false;
            for member in struct_def.members() {
                if !check_dictionary_key_type(member.data_type(), error_reporter) {
                    let rule_error =
                        RuleKind::InvalidKey(InvalidKeyKind::TypeCannotBeUsedAsAKey(member.identifier().to_string()));
                    error_reporter.report_rule_error(rule_error, Some(member.location()));
                    contains_invalid_key_types = true;
                }
            }

            if contains_invalid_key_types {
                let rule_error = RuleKind::InvalidKey(InvalidKeyKind::StructContainsDisallowedType(
                    struct_def.identifier().to_string(),
                ));
                error_reporter.report_rule_error(rule_error, Some(type_ref.location()));
                error_reporter.report_note(
                    format!("struct '{}' is defined here:", struct_def.identifier()),
                    Some(struct_def.location()),
                );
                return false;
            }
            return true;
        }
        Types::Class(class_def) => (false, Some(class_def)),
        Types::Exception(exception_def) => (false, Some(exception_def)),
        Types::Interface(interface_def) => (false, Some(interface_def)),
        Types::Enum(_) => (true, None),
        Types::Trait(trait_def) => (false, Some(trait_def)),
        Types::CustomType(_) => (true, None),
        Types::Sequence(_) => (false, None),
        Types::Dictionary(_) => (false, None),
        Types::Primitive(primitive) => (
            !matches!(primitive, Primitive::Float32 | Primitive::Float64 | Primitive::AnyClass),
            None,
        ),
    };

    if !is_valid {
        let pluralized_kind = match definition.concrete_type() {
            Types::Primitive(_) => definition.kind().to_owned(),
            Types::Class(_) => "classes".to_owned(),
            Types::Dictionary(_) => "dictionaries".to_owned(),
            _ => definition.kind().to_owned() + "s",
        };
        let rule_error = RuleKind::InvalidKey(InvalidKeyKind::TypeCannotBeUsedAsAKey(pluralized_kind.to_string()));
        error_reporter.report_rule_error(rule_error, Some(type_ref.location()));

        // If the key type is a user-defined type, point to where it was defined.
        if let Some(named_symbol_def) = named_symbol {
            error_reporter.report_note(
                format!(
                    "{} '{}' is defined here:",
                    named_symbol_def.kind(),
                    named_symbol_def.identifier(),
                ),
                Some(named_symbol_def.location()),
            );
        }
    }
    is_valid
}
