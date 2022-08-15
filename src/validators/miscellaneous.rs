// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::diagnostics::{DiagnosticsReporter, *};
use crate::grammar::*;
use crate::validators::{ValidationChain, Validator};

pub fn miscellaneous_validators() -> ValidationChain {
    vec![
        Validator::Parameters(stream_parameter_is_last),
        Validator::Struct(validate_compact_struct_not_empty),
    ]
}

fn stream_parameter_is_last(members: &[&Parameter], diagnostic_reporter: &mut DiagnosticsReporter) {
    // If members is empty, `split_last` returns None, and this check is skipped,
    // otherwise it returns all the members, except for the last one. None of these members
    // can be streamed, since only the last member can be.
    if let Some((_, nonstreamed_members)) = members.split_last() {
        for member in nonstreamed_members {
            if member.is_streamed {
                diagnostic_reporter.report(
                    LogicErrorKind::StreamedMembersMustBeLast(member.identifier().to_owned()),
                    Some(member.span()),
                );
            }
        }
    }
}

fn validate_compact_struct_not_empty(struct_def: &Struct, diagnostic_reporter: &mut DiagnosticsReporter) {
    // Compact structs must be non-empty.
    if struct_def.is_compact && struct_def.members().is_empty() {
        diagnostic_reporter.report(LogicErrorKind::CompactStructCannotBeEmpty, Some(struct_def.span()));
    }
}
