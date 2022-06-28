// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::error::ErrorReporter;
use crate::grammar::*;
use crate::validators::{ValidationChain, Validator};

pub fn tag_validators() -> ValidationChain {
    vec![
        Validator::Members(tags_have_optional_types),
        Validator::Members(tagged_containers_cannot_contain_classes),
        Validator::Members(cannot_tag_classes),
        Validator::Members(tags_are_unique),
        Validator::Struct(compact_structs_cannot_contain_tags),
        Validator::Parameters(parameter_order),
    ]
}

/// Validates that the tags are unique.
fn tags_are_unique(members: Vec<&dyn Member>, error_reporter: &mut ErrorReporter) {
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
            error_reporter.report_error(
                format!(
                    "invalid tag on member `{}`: tags must be unique",
                    &window[1].identifier(),
                ),
                Some(window[1].location()),
            );
            error_reporter.report_note(
                format!(
                    "The data member `{}` has previous used the tag value `{}`",
                    &window[0].identifier(),
                    window[0].tag().unwrap()
                ),
                Some(window[0].location()),
            );
        };
    });
}

/// Validate that tagged parameters must follow the required parameters.
fn parameter_order(parameters: &[&Parameter], error_reporter: &mut ErrorReporter) {
    // Folding is used to have an accumulator called `seen` that is set to true once a tagged
    // parameter is found. If `seen` is true on a successive iteration and the parameter has
    // no tag then we have a required parameter after a tagged parameter.
    parameters.iter().fold(false, |seen, parameter| match parameter.tag {
        Some(_) => true,
        None if seen => {
            error_reporter.report_error(
                format!(
                    "invalid parameter `{}`: required parameters must precede tagged parameters",
                    parameter.identifier(),
                ),
                Some(parameter.data_type.location()),
            );
            true
        }
        None => false,
    });
}

/// Validate that tags cannot be used in compact structs.
fn compact_structs_cannot_contain_tags(struct_def: &Struct, error_reporter: &mut ErrorReporter) {
    // Compact structs must be non-empty.
    if struct_def.is_compact && !struct_def.members.is_empty() {
        // Compact structs cannot have tagged data members.
        let mut has_tags = false;
        for member in struct_def.members() {
            if member.tag.is_some() {
                error_reporter.report_error(
                    "tagged data members are not supported in compact structs\nconsider removing the tag, or making the struct non-compact",
                    Some(member.location()),
                );
                has_tags = true;
            }
        }

        if has_tags {
            error_reporter.report_note(
                format!("struct '{}' is declared compact here", struct_def.identifier()),
                Some(struct_def.location()),
            );
        }
    }
}

/// Validate that the data type of the tagged member is optional.
fn tags_have_optional_types(members: Vec<&dyn Member>, error_reporter: &mut ErrorReporter) {
    let tagged_members = members
        .iter()
        .filter(|member| member.tag().is_some())
        .clone()
        .collect::<Vec<_>>();

    // Validate that tagged members are optional.
    for member in tagged_members {
        if !member.data_type().is_optional {
            error_reporter.report_error(
                format!("invalid member `{}`: tagged members must be optional", member.identifier()),
                Some(member.location()),
            );
        }
    }
}

/// Validate that classes cannot be tagged.
fn cannot_tag_classes(members: Vec<&dyn Member>, error_reporter: &mut ErrorReporter) {
    let tagged_members = members
        .iter()
        .filter(|member| member.tag().is_some())
        .clone()
        .collect::<Vec<_>>();

    for member in tagged_members {
        if member.data_type().definition().is_class_type() {
            error_reporter.report_error(
                format!("invalid member `{}`: tagged members cannot be classes", member.identifier()),
                Some(member.location()),
            );
        }
    }
}

/// Validate that tagged container types cannot contain class members.
fn tagged_containers_cannot_contain_classes(members: Vec<&dyn Member>, error_reporter: &mut ErrorReporter) {
    let tagged_members = members
        .iter()
        .filter(|member| member.tag().is_some())
        .clone()
        .collect::<Vec<_>>();

    for member in tagged_members {
        // TODO: This works but the uses_classes method is not intuitive. Should be renamed
        // or changed so that if a class contains no members it returns false.
        if match member.data_type().concrete_type() {
            Types::Class(c) => {
                if c.members().is_empty() {
                    false
                } else {
                    !c.members().iter().any(|m| m.data_type().definition().uses_classes())
                }
            }
            _ => member.data_type().definition().uses_classes(),
        } {
            error_reporter.report_error(
                format!("invalid type `{}`: tagged members cannot contain classes", member.identifier()),
                Some(member.location()),
            );
        }
    }
}
