// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::diagnostics::{DiagnosticReporter, *};
use crate::grammar::*;
use crate::validators::{ValidationChain, Validator};

pub fn miscellaneous_validators() -> ValidationChain {
    vec![
        Validator::Parameters(stream_parameter_is_last),
        Validator::Parameters(at_most_one_stream_parameter),
        Validator::Struct(validate_compact_struct_not_empty),
        Validator::Module(file_scoped_modules_cannot_contain_sub_modules),
    ]
}

fn file_scoped_modules_cannot_contain_sub_modules(module_def: &Module, diagnostic_reporter: &mut DiagnosticReporter) {
    if module_def.is_file_scoped {
        module_def.submodules().iter().for_each(|submodule| {
            ErrorBuilder::new(ErrorKind::FileScopedModuleCannotContainSubModules(
                module_def.identifier().to_owned(),
            ))
            .span(submodule.span())
            .report(diagnostic_reporter);
        });
    }
}

fn at_most_one_stream_parameter(members: &[&Parameter], diagnostic_reporter: &mut DiagnosticReporter) {
    let streamed_members = members.iter().filter(|member| member.is_streamed).collect::<Vec<_>>();
    if streamed_members.len() > 1 {
        streamed_members
        .split_last() // Split at the last element, which is the one we do not want to report an error for.
        .unwrap().1 // All members before the split. Safe to unwrap since we know there are at least two members.
        .iter()
        .for_each(|m| ErrorBuilder::new(ErrorKind::MultipleStreamedMembers).span(m.span()).report(diagnostic_reporter));
    }
}

fn stream_parameter_is_last(members: &[&Parameter], diagnostic_reporter: &mut DiagnosticReporter) {
    members
        .split_last() // Returns None if members is empty.
        .map_or(vec![], |(_, remaining)| remaining.to_vec())
        .into_iter()
        .filter(|m| m.is_streamed)
        .for_each(|m| {
           ErrorBuilder::new(ErrorKind::StreamedMembersMustBeLast(m.identifier().to_owned()))
                .span(m.span())
                .report(diagnostic_reporter);
        });
}

fn validate_compact_struct_not_empty(struct_def: &Struct, diagnostic_reporter: &mut DiagnosticReporter) {
    // Compact structs must be non-empty.
    if struct_def.is_compact && struct_def.members().is_empty() {
        ErrorBuilder::new(ErrorKind::CompactStructCannotBeEmpty)
            .span(struct_def.span())
            .report(diagnostic_reporter);
    }
}
