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
            let error = Error::new_with_notes(
                ErrorKind::CannotHaveDuplicateTag(window[1].identifier().to_owned()),
                Some(window[1].span()),
                vec![Note::new(
                    format!(
                        "The data member `{}` has previous used the tag value `{}`",
                        &window[0].identifier(),
                        window[0].tag().unwrap()
                    ),
                    Some(window[0].span()),
                )],
            );
            diagnostic_reporter.report_error(error);
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
            let error = ErrorKind::RequiredMustPrecedeOptional(parameter.identifier().to_owned());
            diagnostic_reporter.report_error(Error::new(error, Some(parameter.data_type.span())));
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
                let error = Error::new_with_notes(
                    ErrorKind::CompactStructCannotContainTaggedMembers,
                    Some(member.span()),
                    vec![Note::new(
                        format!("struct '{}' is declared compact here", struct_def.identifier()),
                        Some(struct_def.span()),
                    )],
                );
                diagnostic_reporter.report_error(error);
            }
        }
    }
}

/// Validate that the data type of the tagged member is optional.
fn tags_have_optional_types(members: Vec<&dyn Member>, diagnostic_reporter: &mut DiagnosticReporter) {
    let tagged_members = members
        .iter()
        .filter(|member| member.tag().is_some())
        .clone()
        .collect::<Vec<_>>();

    // Validate that tagged members are optional.
    for member in tagged_members {
        if !member.data_type().is_optional {
            diagnostic_reporter.report_error(Error::new(
                ErrorKind::TaggedMemberMustBeOptional(member.identifier().to_owned()),
                Some(member.span()),
            ));
        }
    }
}

fn tagged_members_cannot_use_classes(members: Vec<&dyn Member>, diagnostic_reporter: &mut DiagnosticReporter) {
    for member in members {
        if member.is_tagged() && member.data_type().uses_classes() {
            let identifier = member.identifier().to_owned();
            let error_kind = if member.data_type().is_class_type() {
                ErrorKind::CannotTagClass(identifier)
            } else {
                ErrorKind::CannotTagContainingClass(identifier)
            };
            diagnostic_reporter.report_error(Error::new(error_kind, Some(member.span())));
        }
    }
}
