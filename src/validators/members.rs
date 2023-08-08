// Copyright (c) ZeroC, Inc.

use crate::diagnostics::{Diagnostic, Diagnostics, Error};
use crate::grammar::*;

pub fn validate_members(members: Vec<&impl Member>, diagnostics: &mut Diagnostics) {
    tags_have_optional_types(members.clone(), diagnostics);
    tagged_members_cannot_use_classes(members.clone(), diagnostics);
    tags_are_unique(members.clone(), diagnostics);
}

/// Validates that the tags are unique.
fn tags_are_unique(members: Vec<&impl Member>, diagnostics: &mut Diagnostics) {
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
            .push_into(diagnostics);
        };
    });
}

/// Validate that the type of the tagged member is optional.
fn tags_have_optional_types(members: Vec<&impl Member>, diagnostics: &mut Diagnostics) {
    let tagged_members = members.into_iter().filter(|member| member.is_tagged());

    // Validate that tagged members are optional.
    for member in tagged_members {
        if !member.data_type().is_optional {
            Diagnostic::new(Error::TaggedMemberMustBeOptional {
                identifier: member.identifier().to_owned(),
            })
            .set_span(member.span())
            .push_into(diagnostics);
        }
    }
}

fn tagged_members_cannot_use_classes(members: Vec<&impl Member>, diagnostics: &mut Diagnostics) {
    // Helper function that recursively checks if a type is a class, or contains classes.
    // Infinite cycles are impossible because only classes can contain cycles, and we don't recurse on classes.
    fn uses_classes(typeref: &TypeRef) -> bool {
        match typeref.definition().concrete_type() {
            Types::Struct(struct_def) => struct_def.fields().iter().any(|m| uses_classes(&m.data_type)),
            Types::Class(_) => true,
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
            Diagnostic::new(error).set_span(member.span()).push_into(diagnostics);
        }
    }
}
