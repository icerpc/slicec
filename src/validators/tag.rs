// Copyright (c) ZeroC, Inc.

use crate::diagnostics::{Diagnostic, DiagnosticReporter, Error};
use crate::grammar::*;
use crate::validators::{ValidationChain, Validator};

pub fn tag_validators() -> ValidationChain {
    vec![
        Validator::Members(tags_have_optional_types),
        Validator::Members(tagged_members_cannot_use_classes),
        Validator::Members(tags_are_unique),
        Validator::Struct(compact_structs_cannot_contain_tags),
    ]
}

/// Validates that the tags are unique.
fn tags_are_unique(members: Vec<&dyn Member>, diagnostic_reporter: &mut DiagnosticReporter) {
    // The tagged members must be sorted by value first as we are using windowing to check the
    // n + 1 tagged member against the n tagged member. If the tags are sorted by value then
    // the windowing will reveal any duplicate tags.
    let (_, sorted_tagged_members) = crate::utils::code_gen_util::get_sorted_members(&members);
    sorted_tagged_members.windows(2).for_each(|window| {
        if window[0].tag() == window[1].tag() {
            Diagnostic::new(Error::CannotHaveDuplicateTag {
                identifier: window[1].identifier().to_owned(),
            })
            .set_span(window[1].span())
            .add_note(
                format!(
                    "The tag '{}' is already being used by member '{}'",
                    window[0].tag().unwrap(),
                    &window[0].identifier(),
                ),
                Some(window[0].span()),
            )
            .report(diagnostic_reporter);
        };
    });
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

/// Validate that the data type of the tagged member is optional.
fn tags_have_optional_types(members: Vec<&dyn Member>, diagnostic_reporter: &mut DiagnosticReporter) {
    let tagged_members = members.into_iter().filter(|member| member.is_tagged());

    // Validate that tagged members are optional.
    for member in tagged_members {
        if !member.data_type().is_optional {
            Diagnostic::new(Error::TaggedMemberMustBeOptional {
                identifier: member.identifier().to_owned(),
            })
            .set_span(member.span())
            .report(diagnostic_reporter);
        }
    }
}

fn tagged_members_cannot_use_classes(members: Vec<&dyn Member>, diagnostic_reporter: &mut DiagnosticReporter) {
    // Helper function that recursively checks if a type is a class, or contains classes.
    // Infinite cycles are impossible because only classes can contain cycles, and we don't recurse on classes.
    fn uses_classes(typeref: &TypeRef) -> bool {
        match typeref.definition().concrete_type() {
            Types::Struct(struct_def) => struct_def.fields().iter().any(|m| uses_classes(&m.data_type)),
            Types::Class(_) => true,
            Types::Exception(exception_def) => exception_def.all_fields().iter().any(|m| uses_classes(&m.data_type)),
            Types::Interface(_) => false,
            Types::Enum(_) => false,
            Types::CustomType(_) => false,
            Types::Sequence(sequence) => uses_classes(&sequence.element_type),
            // It is disallowed for key types to use classes, so we only need to check the value type.
            Types::Dictionary(dictionary) => uses_classes(&dictionary.value_type),
            Types::Primitive(primitive) => matches!(primitive, Primitive::AnyClass),
        }
    }

    for member in members {
        if member.is_tagged() && uses_classes(member.data_type()) {
            let identifier = member.identifier().to_owned();
            let error = if member.data_type().is_class_type() {
                Error::CannotTagClass { identifier }
            } else {
                Error::CannotTagContainingClass { identifier }
            };
            Diagnostic::new(error)
                .set_span(member.span())
                .report(diagnostic_reporter);
        }
    }
}
