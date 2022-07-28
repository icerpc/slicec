// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::errors::{ErrorReporter, *};
use crate::grammar::*;
use crate::validators::{ValidationChain, Validator};

pub fn miscellaneous_validators() -> ValidationChain {
    vec![
        Validator::Parameters(stream_parameter_is_last),
        Validator::Struct(validate_compact_struct_not_empty),
    ]
}

fn stream_parameter_is_last(members: &[&Parameter], error_reporter: &mut ErrorReporter) {
    // If members is empty, `split_last` returns None, and this check is skipped,
    // otherwise it returns all the members, except for the last one. None of these members
    // can be streamed, since only the last member can be.
    if let Some((_, nonstreamed_members)) = members.split_last() {
        for member in nonstreamed_members {
            if member.is_streamed {
                error_reporter.report(LogicKind::StreamsMustBeLast, Some(member.location()));
            }
        }
    }
}

fn validate_compact_struct_not_empty(struct_def: &Struct, error_reporter: &mut ErrorReporter) {
    // Compact structs must be non-empty.
    if struct_def.is_compact && struct_def.members().is_empty() {
        error_reporter.report(LogicKind::CompactStructIsEmpty, Some(struct_def.location()));
    }
}
