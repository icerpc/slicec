// Copyright (c) ZeroC, Inc.

use crate::diagnostics::{Diagnostic, DiagnosticReporter, Error};
use crate::grammar::*;

pub fn validate_parameters(members: &[&Parameter], diagnostic_reporter: &mut DiagnosticReporter) {
    stream_parameter_is_last(members, diagnostic_reporter);
    at_most_one_stream_parameter(members, diagnostic_reporter);
}

fn at_most_one_stream_parameter(members: &[&Parameter], diagnostic_reporter: &mut DiagnosticReporter) {
    let streamed_members = members.iter().filter(|member| member.is_streamed).collect::<Vec<_>>();
    if streamed_members.len() > 1 {
        streamed_members
        .split_last() // Split at the last element, which is the one we do not want to report an error for.
        .unwrap().1 // All members before the split. Safe to unwrap since we know there are at least two members.
        .iter()
        .for_each(|m| Diagnostic::new(Error::MultipleStreamedMembers).set_span(m.span()).report(diagnostic_reporter));
    }
}

fn stream_parameter_is_last(members: &[&Parameter], diagnostic_reporter: &mut DiagnosticReporter) {
    members
        .split_last() // Returns None if members is empty.
        .map_or(vec![], |(_, remaining)| remaining.to_vec())
        .into_iter()
        .filter(|m| m.is_streamed)
        .for_each(|m| {
           Diagnostic::new(Error::StreamedMembersMustBeLast { parameter_identifier: m.identifier().to_owned() })
                .set_span(m.span())
                .report(diagnostic_reporter);
        });
}
