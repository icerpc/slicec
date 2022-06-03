// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::code_gen_util::get_sorted_members;
use crate::error::Error;
use crate::grammar::*;
use crate::validators::{Validate, ValidationChain, ValidationResult};

pub fn tag_validators() -> ValidationChain {
    vec![
        Validate::Exception(have_optional_types),
        Validate::Exception(tagged_containers_cannot_contain_classes),
        Validate::Exception(cannot_tag_classes),
        Validate::Struct(compact_structs_cannot_contain_tags),
        Validate::DataMembers(check_tags_uniqueness),
        Validate::DataMembers(have_optional_types),
        Validate::Parameters(parameter_order),
        Validate::Parameters(have_optional_types),
        Validate::Parameters(check_tags_uniqueness),
        Validate::Parameters(tagged_containers_cannot_contain_classes),
        Validate::Parameters(cannot_tag_classes),
    ]
}

/// Validates that the tags are unique.
fn check_tags_uniqueness(members: &[&impl Member]) -> ValidationResult {
    // The tagged members must be sorted by value first as we are using windowing to check the
    // n + 1 tagged member against the n tagged member. If the tags are sorted by value then
    // the windowing will reveal any duplicate tags.
    let (_, tagged_members) = get_sorted_members(members);
    let mut errors = vec![];
    tagged_members.windows(2).for_each(|window| {
        if window[0].tag() == window[1].tag() {
            errors.push(Error {
                message: format!(
                    "invalid tag on member `{}`: tags must be unique",
                    &window[1].identifier()
                ),
                location: Some(window[1].location().clone()),
                severity: crate::error::ErrorLevel::Error,
            });
            errors.push(Error {
                message: format!(
                    "The data member `{}` has previous used the tag value `{}`",
                    &window[0].identifier(),
                    window[0].tag().unwrap()
                ),
                location: Some(window[0].location().clone()),
                severity: crate::error::ErrorLevel::Error,
            });
        };
    });
    match errors.is_empty() {
        true => Ok(()),
        false => Err(errors),
    }
}

/// Validate that tagged parameters must follow the required parameters.
fn parameter_order(parameters: &[&Parameter]) -> ValidationResult {
    // Folding is used to have an accumulator called `seen` that is set to true once a tagged
    // parameter is found. If `seen` is true on a successive iteration and the parameter has
    // no tag then we have a required parameter after a tagged parameter.
    let mut errors = vec![];
    parameters.iter().fold(false, |seen, parameter| {
            match parameter.tag {
                Some(_) => true,
                None if seen => {
                    errors.push(Error {
                        message: format!(
                            "invalid parameter `{}`: required parameters must precede tagged parameters",
                            parameter.identifier(),
                        ),
                        location: Some(parameter.data_type.location.clone()),
                        severity: crate::error::ErrorLevel::Error,
                    });
                    true
                },
                None => false
            }
        });

    match errors.is_empty() {
        true => Ok(()),
        false => Err(errors),
    }
}

/// Validate that tags cannot be used in compact structs.
fn compact_structs_cannot_contain_tags(struct_def: &Struct) -> ValidationResult {
    // Compact structs must be non-empty.
    let mut errors = vec![];
    if struct_def.is_compact && !struct_def.members.is_empty() {
        // Compact structs cannot have tagged data members.
        let mut has_tags = false;
        for member in struct_def.members() {
            if member.tag.is_some() {
                errors.push(Error {
                    message: "tagged data members are not supported in compact structs\nconsider removing the tag, or making the struct non-compact"
                        .to_owned(),
                    location: Some(member.location().clone()),
                    severity: crate::error::ErrorLevel::Error,
                });
                has_tags = true;
            }
        }

        if has_tags {
            errors.push(Error {
                message: format!(
                    "struct '{}' is declared compact here",
                    struct_def.identifier(),
                ),
                location: Some(struct_def.location.clone()),
                severity: crate::error::ErrorLevel::Error,
            });
        }
    }
    match errors.is_empty() {
        true => Ok(()),
        false => Err(errors),
    }
}

/// Validate that the data type of the tagged member is optional.
fn have_optional_types(members: &[&impl Member]) -> ValidationResult {
    let mut errors = vec![];
    let tagged_members = members
        .iter()
        .filter(|member| member.tag().is_some())
        .clone()
        .collect::<Vec<_>>();

    // Validate that tagged members are optional.
    for member in tagged_members {
        if !member.data_type().is_optional {
            errors.push(Error {
                message: format!(
                    "invalid member `{}`: tagged members must be optional",
                    member.identifier()
                ),
                location: Some(member.location().clone()),
                severity: crate::error::ErrorLevel::Error,
            });
        }
    }

    match errors.is_empty() {
        true => Ok(()),
        false => Err(errors),
    }
}

/// Validate that classes cannot be tagged.
fn cannot_tag_classes(members: &[&impl Member]) -> ValidationResult {
    let mut errors = vec![];
    let tagged_members = members
        .iter()
        .filter(|member| member.tag().is_some())
        .clone()
        .collect::<Vec<_>>();

    for member in tagged_members {
        if member.data_type().definition().is_class_type() {
            errors.push(Error {
                message: format!(
                    "invalid member `{}`: tagged members cannot be classes",
                    member.identifier()
                ),
                location: Some(member.location().clone()),
                severity: crate::error::ErrorLevel::Error,
            });
        }
    }
    match errors.is_empty() {
        true => Ok(()),
        false => Err(errors),
    }
}

/// Validate that tagged container types cannot contain class members.
fn tagged_containers_cannot_contain_classes(members: &[&impl Member]) -> ValidationResult {
    let mut errors = vec![];
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
                    !c.members()
                        .iter()
                        .any(|m| m.data_type().definition().uses_classes())
                }
            }
            _ => member.data_type().definition().uses_classes(),
        } {
            errors.push(Error {
                message: format!(
                    "invalid type `{}`: tagged members cannot contain classes",
                    member.identifier()
                ),
                location: Some(member.location().clone()),
                severity: crate::error::ErrorLevel::Error,
            });
        }
    }
    match errors.is_empty() {
        true => Ok(()),
        false => Err(errors),
    }
}
