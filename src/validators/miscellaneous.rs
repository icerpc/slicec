// Copyright (c) ZeroC, Inc. All rights reserved.

use super::ValidatorVisitor;
use crate::diagnostics::{Error, ErrorKind};
use crate::grammar::*;

impl ValidatorVisitor<'_> {
pub(super) fn file_scoped_modules_cannot_contain_sub_modules(&mut self, module_def: &Module) {
    if module_def.is_file_scoped {
        module_def.submodules().iter().for_each(|submodule| {
            Error::new(ErrorKind::FileScopedModuleCannotContainSubModules(
                module_def.identifier().to_owned(),
            ))
            .set_span(submodule.span())
            .report(self.diagnostic_reporter);
        });
    }
}

pub(super) fn at_most_one_stream_parameter(&mut self, members: &[&Parameter]) {
    let streamed_members = members.iter().filter(|member| member.is_streamed).collect::<Vec<_>>();
    if streamed_members.len() > 1 {
        streamed_members
        .split_last() // Split at the last element, which is the one we do not want to report an error for.
        .unwrap().1 // All members before the split. Safe to unwrap since we know there are at least two members.
        .iter()
        .for_each(|m| Error::new(ErrorKind::MultipleStreamedMembers).set_span(m.span()).report(self.diagnostic_reporter));
    }
}

pub(super) fn stream_parameter_is_last(&mut self, members: &[&Parameter]) {
    members
        .split_last() // Returns None if members is empty.
        .map_or(vec![], |(_, remaining)| remaining.to_vec())
        .into_iter()
        .filter(|m| m.is_streamed)
        .for_each(|m| {
           Error::new(ErrorKind::StreamedMembersMustBeLast(m.identifier().to_owned()))
                .set_span(m.span())
                .report(self.diagnostic_reporter);
        });
}

pub(super) fn validate_compact_struct_not_empty(&mut self, struct_def: &Struct) {
    // Compact structs must be non-empty.
    if struct_def.is_compact && struct_def.members().is_empty() {
        Error::new(ErrorKind::CompactStructCannotBeEmpty)
            .set_span(struct_def.span())
            .report(self.diagnostic_reporter);
    }
}
}
