// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::error::ErrorReporter;
use crate::grammar::*;
use crate::visitor::Visitor;

#[derive(Debug)]
pub struct ShadowingValidator<'a> {
    pub error_reporter: &'a mut ErrorReporter,
}

impl Visitor for ShadowingValidator<'_> {
    fn visit_interface_start(&mut self, interface: &Interface) {
        let mut operations = interface.operations();
        let mut base_operations = interface.operations();
        let mut all_operations = interface.all_operations();

        all_operations.sort_by_key(|o| o.identifier());
        all_operations.windows(2).for_each(|window| {
            if window[0].identifier() == window[1].identifier() {
                self.error_reporter.report_error(
                    format!(
                        "operation `{}` is shadowed by another operation",
                        window[0].identifier()
                    ),
                    Some(&window[0].location),
                );
                // self.error_reporter.report_error(
                //     format!(
                //         "The operation `{}` was  previous used the tag value `{}`",
                //         &window[0].identifier(),
                //         window[0].tag().unwrap()
                //     ),
                //     Some(window[0].location()),
                // );
            }
        });
    }

    fn visit_exception_start(&mut self, exception: &Exception) {}

    fn visit_class_start(&mut self, class: &Class) {}
}
