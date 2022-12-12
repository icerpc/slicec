// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::diagnostics::{DiagnosticReporter, Error, ErrorKind};
use crate::grammar::*;
use crate::slice_file::SliceFile;
use crate::visitor::Visitor;
use std::collections::HashMap;

pub(super) fn detect_cycles(slice_files: &HashMap<String, SliceFile>, diagnostic_reporter: &mut DiagnosticReporter) {
    let mut cycle_detector = CycleDetector {
        dependency_stack: Vec::new(),
        diagnostic_reporter,
    };

    // First, visit everything immutably to check for cycles and compute the supported encodings.
    for slice_file in slice_files.values() {
        slice_file.visit_with(&mut cycle_detector);
    }
}

struct CycleDetector<'a> {
    // Stack of all the types we've seen in the dependency chain we're currently checking.
    dependency_stack: Vec<String>,
    diagnostic_reporter: &'a mut DiagnosticReporter,
}

impl<'a> CycleDetector<'a> {
    fn check_for_cycle<T: Entity + Type>(&mut self, type_def: &T, type_id: &str) -> bool {
        // Check if the type is self-referential by whether we've already seen it's type-id in
        // the dependency chain we're currently checking.
        if let Some(i) = self.dependency_stack.iter().position(|x| x == type_id) {
            let cycle_string = self.dependency_stack[i..].join(" -> ");
            Error::new(ErrorKind::InfiniteSizeCycle(type_id.to_string(), cycle_string))
                .set_span(type_def.span())
                .report(self.diagnostic_reporter);
            true
        } else {
            false
        }
    }
}

impl<'a> Visitor for CycleDetector<'a> {
    fn visit_struct_start(&mut self, struct_def: &Struct) {
        let type_id = struct_def.module_scoped_identifier();
        if self.check_for_cycle(struct_def, &type_id) {
            // If the type is cyclic, return early to avoid an infinite loop.
            // `check_for_cycle` will already have reported an error message.
            return;
        }

        // Push the struct's type-id on to the stack before its data members are visited.
        self.dependency_stack.push(type_id);
    }

    fn visit_struct_end(&mut self, _: &Struct) {
        self.dependency_stack.pop().unwrap();
    }

    fn visit_exception_start(&mut self, exception_def: &Exception) {
        let type_id = exception_def.module_scoped_identifier();
        if self.check_for_cycle(exception_def, &type_id) {
            // If the type is cyclic, return early to avoid an infinite loop.
            // `check_for_cycle` will already have reported an error message.
            return;
        }

        // Push the exception's type-id on to the stack before its data members are visited.
        self.dependency_stack.push(type_id);
    }

    fn visit_exception_end(&mut self, _: &Exception) {
        self.dependency_stack.pop().unwrap();
    }

    fn visit_data_member(&mut self, data_member: &DataMember) {
        match data_member.data_type().concrete_type() {
            // Only structs and exceptions can contain infinite cycles.
            // Classes are allowed to contain cycles since they use reference semantics.
            Types::Struct(struct_def) => struct_def.visit_with(self),
            Types::Exception(exception_def) => exception_def.visit_with(self),
            _ => {}
        }
    }
}
