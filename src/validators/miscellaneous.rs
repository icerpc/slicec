// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::diagnostics::{DiagnosticReporter, *};
use crate::grammar::*;
use crate::validators::{ValidationChain, Validator};

pub fn miscellaneous_validators() -> ValidationChain {
    vec![
        Validator::Parameters(stream_parameter_is_last),
        Validator::Parameters(at_most_one_stream_parameter),
        Validator::Struct(validate_compact_struct_not_empty),
    ]
}

fn at_most_one_stream_parameter(members: &[&Parameter], diagnostic_reporter: &mut DiagnosticReporter) {
    let streamed_members = members.iter().filter(|member| member.is_streamed).collect::<Vec<_>>();
    if streamed_members.len() > 1 {
        streamed_members
        .split_last() // Split at the last element, which is the one we do not want to report an error for.
        .unwrap().1 // All members before the split. Safe to unwrap since we know there are at least two members.
        .iter()
        .for_each(|m| diagnostic_reporter.report_error(Error::new(ErrorKind::MultipleStreamedMembers, Some(m.span()))));
    }
}

fn stream_parameter_is_last(members: &[&Parameter], diagnostic_reporter: &mut DiagnosticReporter) {
    // If members is empty, `split_last` returns None, and this check is skipped,
    // otherwise it returns all the members, except for the last one. None of these members
    // can be streamed, since only the last member can be.
    if let Some((_, remaining)) = members.split_last() {
        // Check that only the last parameter is streamed.
        remaining.iter().filter(|member| member.is_streamed).for_each(|member| {
            diagnostic_reporter.report_error(Error::new(
                ErrorKind::StreamedMembersMustBeLast(member.identifier().to_owned()),
                Some(member.span()),
            ));
        });
    }
}

fn validate_compact_struct_not_empty(struct_def: &Struct, diagnostic_reporter: &mut DiagnosticReporter) {
    // Compact structs must be non-empty.
    if struct_def.is_compact && struct_def.members().is_empty() {
        diagnostic_reporter.report_error(Error::new(
            ErrorKind::CompactStructCannotBeEmpty,
            Some(struct_def.span()),
        ));
    }
}
