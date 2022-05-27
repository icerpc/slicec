// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::error::ErrorReporter;
use crate::grammar::*;
use crate::visitor::Visitor;

#[derive(Debug)]
pub struct IdentifierValidator<'a> {
    pub error_reporter: &'a mut ErrorReporter,
}

impl IdentifierValidator<'_> {
    pub fn check_for_redefined(&mut self, symbols: &[&impl NamedSymbol]) {
        let mut identifiers = symbols
            .iter()
            .map(|s| s.raw_identifier())
            .collect::<Vec<_>>();

        // Sort first so that we can use windows to search for duplicates.
        identifiers.sort_by_key(|identifier| identifier.value.to_owned());
        identifiers.windows(2).for_each(|window| {
            if window[0].value == window[1].value {
                self.error_reporter.report_error(
                    format!("redefinition of {}", window[1].value),
                    Some(&window[1].location),
                );

                self.error_reporter.report_error(
                    format!("{} was previously defined here", window[0].value),
                    Some(&window[0].location),
                );
            }
        });
    }

    pub fn check_for_shadowing(
        &mut self,
        symbols: &[&impl NamedSymbol],
        inherited_symbols: &[&impl NamedSymbol],
    ) {
        let identifiers = symbols
            .iter()
            .map(|s| s.raw_identifier())
            .collect::<Vec<_>>();

        let inherited_symbols = inherited_symbols
            .iter()
            .map(|s| s.raw_identifier())
            .collect::<Vec<_>>();

        identifiers.iter().for_each(|identifier| {
            inherited_symbols
                .iter()
                .filter(|inherited_identifier| inherited_identifier.value == identifier.value)
                .for_each(|inherited_identifier| {
                    self.error_reporter.report_error(
                        format!("{} shadows another symbol", identifier.value),
                        Some(&identifier.location),
                    );

                    self.error_reporter.report_error(
                        format!("{} was previously defined here", inherited_identifier.value),
                        Some(&inherited_identifier.location),
                    );
                });
        });
    }
}

impl Visitor for IdentifierValidator<'_> {
    fn visit_interface_start(&mut self, interface: &Interface) {
        let operations = interface.operations();
        let inherited_operations = interface.all_inherited_operations();

        self.check_for_redefined(&operations);
        self.check_for_shadowing(&operations, &inherited_operations);
    }

    fn visit_exception_start(&mut self, exception: &Exception) {
        let members = exception.members();
        let inherited_members = exception.all_inherited_members();
        self.check_for_redefined(&members);
        self.check_for_shadowing(&members, &inherited_members);
    }

    fn visit_class_start(&mut self, class: &Class) {
        let members = class.members();
        let inherited_members = class.all_inherited_members();
        self.check_for_redefined(&members);
        self.check_for_shadowing(&members, &inherited_members);
    }

    fn visit_struct_start(&mut self, struct_def: &Struct) {
        self.check_for_redefined(&struct_def.members());
    }
}
