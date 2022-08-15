// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::diagnostics::*;
use crate::grammar::*;
use crate::validators::{ValidationChain, Validator};

pub fn dictionary_validators() -> ValidationChain {
    vec![Validator::Dictionaries(has_allowed_key_type)]
}

pub fn has_allowed_key_type(dictionaries: &[&Dictionary], diagnostic_reporter: &mut DiagnosticsReporter) {
    for dictionary in dictionaries {
        check_dictionary_key_type(&dictionary.key_type, diagnostic_reporter);
    }
}

fn check_dictionary_key_type(type_ref: &TypeRef, diagnostic_reporter: &mut DiagnosticsReporter) -> bool {
    // Optional types cannot be used as dictionary keys.
    if type_ref.is_optional {
        diagnostic_reporter.report(LogicKind::KeyMustBeNonOptional, Some(type_ref.span()));
        return false;
    }

    let definition = type_ref.definition();
    let (is_valid, named_symbol): (bool, Option<&dyn NamedSymbol>) = match definition.concrete_type() {
        Types::Struct(struct_def) => {
            // Only compact structs can be used for dictionary keys.
            if !struct_def.is_compact {
                diagnostic_reporter.report(LogicKind::StructKeyMustBeCompact, Some(type_ref.span()));
                diagnostic_reporter.report(
                    DiagnosticKind::new_note(format!("struct '{}' is defined here:", struct_def.identifier())),
                    Some(struct_def.span()),
                );
                return false;
            }

            // Check that all the data members of the struct are also valid key types.
            let mut contains_invalid_key_types = false;
            for member in struct_def.members() {
                if !check_dictionary_key_type(member.data_type(), diagnostic_reporter) {
                    let diagnostic = LogicKind::KeyTypeNotSupported(member.identifier().to_owned());
                    diagnostic_reporter.report(diagnostic, Some(member.span()));
                    contains_invalid_key_types = true;
                }
            }

            if contains_invalid_key_types {
                let diagnostic = LogicKind::StructKeyContainsDisallowedType(struct_def.identifier().to_owned());
                diagnostic_reporter.report(diagnostic, Some(type_ref.span()));
                diagnostic_reporter.report(
                    DiagnosticKind::new_note(format!("struct '{}' is defined here:", struct_def.identifier())),
                    Some(struct_def.span()),
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

        diagnostic_reporter.report(LogicKind::KeyTypeNotSupported(pluralized_kind), Some(type_ref.span()));

        // If the key type is a user-defined type, point to where it was defined.
        if let Some(named_symbol_def) = named_symbol {
            diagnostic_reporter.report(
                DiagnosticKind::new_note(format!(
                    "{} '{}' is defined here:",
                    named_symbol_def.kind(),
                    named_symbol_def.identifier(),
                )),
                Some(named_symbol_def.span()),
            );
        }
    }
    is_valid
}
