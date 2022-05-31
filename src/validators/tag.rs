// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::code_gen_util::get_sorted_members;
use crate::error::ErrorReporter;
use crate::grammar::*;
use crate::visitor::Visitor;

#[derive(Debug)]
pub struct TagValidator<'a> {
    pub error_reporter: &'a mut ErrorReporter,
}

impl TagValidator<'_> {
    // Validate that tagged parameters must follow the required parameters.
    fn parameter_order(&mut self, parameters: &[&Parameter]) {
        // Folding is used to have an accumulator called `seen` that is set to true once a tagged
        // parameter is found. If `seen` is true on a successive iteration and the parameter has
        // no tag then we have a required parameter after a tagged parameter.
        parameters.iter().fold(false, |seen, parameter| {
            match parameter.tag {
                Some(_) => true,
                None if seen => {
                    self.error_reporter.report_error(
                        format!(
                            "invalid parameter `{}`: required parameters must precede tagged parameters",
                            parameter.identifier()
                        ),
                        Some(&parameter.data_type.location)
                    );
                    true
                },
                None => false
            }
        });
    }

    /// Validate that tags cannot be used in compact structs.
    fn compact_structs_cannot_contain_tags(&mut self, struct_def: &Struct) {
        // Compact structs cannot have tagged data members.
        let mut has_tags = false;
        for member in struct_def.members() {
            if member.is_tagged() {
                self.error_reporter.report_error(
                    "tagged data members are not supported in compact structs\n\
                        consider removing the tag, or making the struct non-compact",
                    Some(member.location()),
                );
                has_tags = true;
            }
        }

        if has_tags {
            self.error_reporter.report_note(
                format!(
                    "struct '{}' is declared compact here",
                    struct_def.identifier()
                ),
                Some(&struct_def.location),
            );
        }
    }

    /// Validates that the tags are unique.
    fn tags_are_unique(&mut self, members: &[&impl Member]) {
        // The tagged members must be sorted by value first as we are using windowing to check the
        // n + 1 tagged member against the n tagged member. If the tags are sorted by value then
        // the windowing will reveal any duplicate tags.
        let (_, tagged_members) = get_sorted_members(members);
        tagged_members.windows(2).for_each(|window| {
            if window[0].tag() == window[1].tag() {
                self.error_reporter.report_error(
                    format!(
                        "invalid tag on member `{}`: tags must be unique",
                        &window[1].identifier()
                    ),
                    Some(window[1].location()),
                );
                self.error_reporter.report_error(
                    format!(
                        "The data member `{}` has previous used the tag value `{}`",
                        &window[0].identifier(),
                        window[0].tag().unwrap()
                    ),
                    Some(window[0].location()),
                );
            }
        });
    }

    /// Validate that the data type of the tagged member is optional.
    fn have_optional_types(&mut self, members: &[&impl Member]) {
        let tagged_members = members
            .iter()
            .filter(|member| member.is_tagged())
            .clone()
            .collect::<Vec<_>>();

        // Validate that tagged members are optional.
        for member in tagged_members {
            if !member.data_type().is_optional {
                self.error_reporter.report_error(
                    format!(
                        "invalid member `{}`: tagged members must be optional",
                        member.identifier()
                    )
                    .to_owned(),
                    Some(member.location()),
                );
            }
        }
    }

    /// Validate that classes cannot be tagged.
    fn cannot_tag_classes(&mut self, members: &[&impl Member]) {
        let tagged_members = members
            .iter()
            .filter(|member| member.is_tagged())
            .clone()
            .collect::<Vec<_>>();

        for member in tagged_members {
            if member.data_type().definition().is_class_type() {
                self.error_reporter.report_error(
                    format!(
                        "invalid member `{}`: tagged members cannot be classes",
                        member.identifier()
                    )
                    .to_owned(),
                    Some(member.location()),
                );
            }
        }
    }

    /// Validate that tagged container types cannot contain class members.
    fn tagged_containers_cannot_contain_classes(&mut self, members: &[&impl Member]) {
        let tagged_members = members
            .iter()
            .filter(|member| member.is_tagged())
            .clone()
            .collect::<Vec<_>>();

        for member in tagged_members {
            // TODO: This works but the uses_classes method is not intuitive. Should be renamed
            // or changed so that if a class contains no members it returns false.
            if match member.data_type().concrete_type() {
                Types::Class(c) => {
                    if c.members().is_empty() {
                        false
                    } else {
                        !c.members()
                            .iter()
                            .any(|m| m.data_type().definition().uses_classes())
                    }
                }
                _ => member.data_type().definition().uses_classes(),
            } {
                self.error_reporter.report_error(
                    format!(
                        "invalid type `{}`: tagged members cannot contain classes",
                        member.identifier()
                    )
                    .to_owned(),
                    Some(member.location()),
                );
            }
        }
    }
}

impl<'a> Visitor for TagValidator<'a> {
    fn visit_exception_start(&mut self, exception_def: &Exception) {
        self.tags_are_unique(&exception_def.members());
        self.have_optional_types(&exception_def.members());
        self.tagged_containers_cannot_contain_classes(&exception_def.members());
        self.cannot_tag_classes(&exception_def.members());
    }

    fn visit_struct_start(&mut self, struct_def: &Struct) {
        if struct_def.is_compact {
            self.compact_structs_cannot_contain_tags(struct_def)
        } else {
            // Tags can only exist on non compact structs.
            self.tags_are_unique(&struct_def.members());
            self.have_optional_types(&struct_def.members());
        }
    }

    fn visit_class_start(&mut self, class_def: &Class) {
        self.tags_are_unique(&class_def.members());
        self.have_optional_types(&class_def.members());
        self.tagged_containers_cannot_contain_classes(&class_def.members());
        self.cannot_tag_classes(&class_def.members());
    }

    fn visit_operation_start(&mut self, operation_def: &Operation) {
        let members = operation_def.all_members();
        self.parameter_order(&members);
        self.have_optional_types(&members);
        self.tags_are_unique(&members);
        self.tagged_containers_cannot_contain_classes(&members);
        self.cannot_tag_classes(&members);
    }
}
