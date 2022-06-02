// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::error::Error;
use crate::grammar::*;
use crate::validators::{Validate, ValidationChain, ValidationResult};

pub fn miscellaneous_validators() -> ValidationChain {
    vec![
        Validate::ParametersAndReturnMember(validate_stream_member),
        Validate::Struct(validate_compact_struct_not_empty),
    ]
}

fn validate_stream_member(members: &[&Parameter]) -> ValidationResult {
    let mut errors = vec![];
    // If members is empty, `split_last` returns None, and this check is skipped,
    // otherwise it returns all the members, except for the last one. None of these members
    // can be streamed, since only the last member can be.
    if let Some((_, nonstreamed_members)) = members.split_last() {
        for member in nonstreamed_members {
            if member.is_streamed {
                errors.push(Error {
                    message: "only the last parameter in an operation can use the stream modifier"
                        .to_owned(),
                    location: Some(member.location.clone()),
                    severity: crate::error::ErrorLevel::Error,
                });
            }
        }
    }
    match errors.is_empty() {
        true => Ok(()),
        false => Err(errors),
    }
}

fn validate_compact_struct_not_empty(struct_def: &Struct) -> ValidationResult {
    let mut errors = vec![];
    if struct_def.is_compact {
        // Compact structs must be non-empty.
        if struct_def.members().is_empty() {
            errors.push(Error {
                message: "compact structs must be non-empty".to_owned(),
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
