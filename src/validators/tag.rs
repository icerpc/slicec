// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::diagnostics::*;
use crate::grammar::*;
use crate::validators::{ValidationChain, Validator};

pub fn tag_validators() -> ValidationChain {
    vec![
        Validator::Members(tags_have_optional_types),
        Validator::Members(tagged_members_cannot_use_classes),
        Validator::Members(tags_are_unique),
        Validator::Struct(compact_structs_cannot_contain_tags),
        Validator::Parameters(parameter_order),
    ]
}

/// Validates that the tags are unique.
fn tags_are_unique(members: Vec<&dyn Member>, diagnostic_reporter: &mut DiagnosticReporter) {
    // The tagged members must be sorted by value first as we are using windowing to check the
    // n + 1 tagged member against the n tagged member. If the tags are sorted by value then
    // the windowing will reveal any duplicate tags.
    let mut tagged_members = members
        .iter()
        .filter(|member| member.is_tagged())
        .cloned()
        .collect::<Vec<_>>();
    tagged_members.sort_by_key(|member| member.tag().unwrap());
    tagged_members.windows(2).for_each(|window| {
        if window[0].tag() == window[1].tag() {
            Error::new(ErrorKind::CannotHaveDuplicateTag {
                member_identifier: window[1].identifier().to_owned(),
            })
            .set_span(window[1].span())
            .add_note(
                format!(
                    "The data member `{}` has previous used the tag value `{}`",
                    &window[0].identifier(),
                    window[0].tag().unwrap(),
                ),
                Some(window[0].span()),
            )
            .report(diagnostic_reporter);
        };
    });
}

/// Validate that tagged parameters must follow the required parameters.
fn parameter_order(parameters: &[&Parameter], diagnostic_reporter: &mut DiagnosticReporter) {
    // Folding is used to have an accumulator called `seen` that is set to true once a tagged
    // parameter is found. If `seen` is true on a successive iteration and the parameter has
    // no tag then we have a required parameter after a tagged parameter.
    parameters.iter().fold(false, |seen, parameter| match parameter.tag {
        Some(_) => true,
        None if seen => {
            let error = ErrorKind::RequiredMustPrecedeOptional {
                parameter_identifier: parameter.identifier().to_owned(),
            };
            Error::new(error)
                .set_span(parameter.data_type.span())
                .report(diagnostic_reporter);
            true
        }
        None => false,
    });
}

/// Validate that tags cannot be used in compact structs.
fn compact_structs_cannot_contain_tags(struct_def: &Struct, diagnostic_reporter: &mut DiagnosticReporter) {
    // Compact structs must be non-empty.
    if struct_def.is_compact && !struct_def.members.is_empty() {
        // Compact structs cannot have tagged data members.
        for member in struct_def.members() {
            if member.tag.is_some() {
                Error::new(ErrorKind::CompactStructCannotContainTaggedMembers)
                    .set_span(member.span())
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
    let tagged_members = members
        .iter()
        .filter(|member| member.tag().is_some())
        .collect::<Vec<_>>();

    // Validate that tagged members are optional.
    for member in tagged_members {
        if !member.data_type().is_optional {
            Error::new(ErrorKind::TaggedMemberMustBeOptional {
                member_identifier: member.identifier().to_owned(),
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
            Types::Struct(struct_def) => struct_def.members().iter().any(|m| uses_classes(&m.data_type)),
            Types::Class(_) => true,
            Types::Exception(exception_def) => exception_def.all_members().iter().any(|m| uses_classes(&m.data_type)),
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
            let error_kind = if member.data_type().is_class_type() {
                ErrorKind::CannotTagClass {
                    member_identifier: identifier,
                }
            } else {
                ErrorKind::CannotTagContainingClass {
                    member_identifier: identifier,
                }
            };
            Error::new(error_kind)
                .set_span(member.span())
                .report(diagnostic_reporter);
        }
    }
}
