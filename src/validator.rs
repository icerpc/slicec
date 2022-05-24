// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::ast::Ast;
use crate::code_gen_util::get_sorted_members;
use crate::error::ErrorReporter;
use crate::grammar::*;
use crate::slice_file::SliceFile;
use crate::visitor::Visitor;
use std::borrow::Borrow;
use std::collections::HashMap;

#[derive(Debug)]
pub(crate) struct Validator<'a> {
    pub error_reporter: &'a mut ErrorReporter,
    pub ast: &'a Ast,
}

impl Validator<'_> {
    pub fn validate(&mut self, slice_files: &HashMap<String, SliceFile>) {
        for slice_file in slice_files.values() {
            slice_file.visit_with(self);
            slice_file.visit_with(&mut TagValidator { error_reporter: self.error_reporter });
            slice_file.visit_with(&mut EnumValidator {
                error_reporter: self.error_reporter,
                encoding: slice_file.encoding(),
            });
            slice_file.visit_with(&mut AttributeValidator { error_reporter: self.error_reporter })
        }
        self.validate_dictionary_key_types();
    }

    fn validate_dictionary_key_types(&mut self) {
        for type_ptr in &self.ast.anonymous_types {
            if let Types::Dictionary(dictionary) = type_ptr.borrow().concrete_type() {
                self.check_dictionary_key_type(&dictionary.key_type);
            }
        }
    }

    fn check_dictionary_key_type(&mut self, type_ref: &TypeRef) -> bool {
        // Optional types cannot be used as dictionary keys.
        if type_ref.is_optional {
            self.error_reporter.report_error(
                "invalid dictionary key type: optional types cannot be used as a dictionary key type".to_owned(),
                Some(&type_ref.location),
            );
            return false;
        }

        let definition = type_ref.definition();
        let (is_valid, named_symbol): (bool, Option<&dyn NamedSymbol>) = match definition
            .concrete_type()
        {
            Types::Struct(struct_def) => {
                // Only compact structs can be used for dictionary keys.
                if !struct_def.is_compact {
                    self.error_reporter.report_error(
                        "invalid dictionary key type: structs must be compact to be used as a dictionary key type".to_owned(),
                        Some(&type_ref.location),
                    );
                    self.error_reporter.report_note(
                        format!("struct '{}' is defined here:", struct_def.identifier()),
                        Some(&struct_def.location),
                    );
                    return false;
                }

                // Check that all the data members of the struct are also valid key types.
                let mut contains_invalid_key_types = false;
                for member in struct_def.members() {
                    if !self.check_dictionary_key_type(member.data_type()) {
                        self.error_reporter.report_error(
                            format!(
                                "data member '{}' cannot be used as a dictionary key type",
                                member.identifier(),
                            ),
                            Some(&member.location),
                        );
                        contains_invalid_key_types = true;
                    }
                }

                if contains_invalid_key_types {
                    self.error_reporter.report_error(
                        format!(
                            "invalid dictionary key type: struct '{}' contains members that cannot be used as a dictionary key type",
                            struct_def.identifier(),
                        ),
                        Some(&type_ref.location),
                    );
                    self.error_reporter.report_note(
                        format!("struct '{}' is defined here:", struct_def.identifier()),
                        Some(&struct_def.location),
                    );
                    return false;
                }
                return true;
            }
            Types::Class(class_def) => (false, Some(class_def)),
            Types::Exception(exception_def) => (false, Some(exception_def)),
            Types::Interface(interface_def) => (false, Some(interface_def)),
            Types::Enum(_) => (true, None),
            Types::Trait(trait_def) => (false, Some(trait_def)),
            Types::CustomType(_) => (true, None),
            Types::Sequence(_) => (false, None),
            Types::Dictionary(_) => (false, None),
            Types::Primitive(primitive) => (
                !matches!(
                    primitive,
                    Primitive::Float32 | Primitive::Float64 | Primitive::AnyClass
                ),
                None,
            ),
        };

        if !is_valid {
            let pluralized_kind = match definition.concrete_type() {
                Types::Primitive(_) => definition.kind().to_owned(),
                Types::Class(_) => "classes".to_owned(),
                Types::Dictionary(_) => "dictionaries".to_owned(),
                _ => definition.kind().to_owned() + "s",
            };

            self.error_reporter.report_error(
                format!(
                    "invalid dictionary key type: {} cannot be used as a dictionary key type",
                    pluralized_kind,
                ),
                Some(&type_ref.location),
            );

            // If the key type is a user-defined type, point to where it was defined.
            if let Some(named_symbol_def) = named_symbol {
                self.error_reporter.report_note(
                    format!(
                        "{} '{}' is defined here:",
                        named_symbol_def.kind(),
                        named_symbol_def.identifier(),
                    ),
                    Some(named_symbol_def.location()),
                );
            }
        }
        is_valid
    }

    fn validate_stream_member(&mut self, members: Vec<&Parameter>) {
        // If members is empty, `split_last` returns None, and this check is skipped,
        // otherwise it returns all the members, except for the last one. None of these members
        // can be streamed, since only the last member can be.
        if let Some((_, nonstreamed_members)) = members.split_last() {
            for member in nonstreamed_members {
                if member.is_streamed {
                    self.error_reporter.report_error(
                        "only the last parameter in an operation can use the stream modifier"
                            .to_owned(),
                        Some(&member.location),
                    );
                }
            }
        }
    }
}

impl<'a> Visitor for Validator<'a> {
    fn visit_struct_start(&mut self, struct_def: &Struct) {
        if struct_def.is_compact {
            // Compact structs must be non-empty.
            if struct_def.members().is_empty() {
                self.error_reporter.report_error(
                    "compact structs must be non-empty".to_owned(),
                    Some(&struct_def.location),
                )
            }
        }
    }

    fn visit_operation_start(&mut self, operation_def: &Operation) {
        self.validate_stream_member(operation_def.parameters());
        self.validate_stream_member(operation_def.return_members());
    }
}

#[derive(Debug)]
struct AttributeValidator<'a> {
    pub error_reporter: &'a mut ErrorReporter,
}

impl AttributeValidator<'_> {
    /// Validates that each argument does not contain spaces or tabs unless it is a string literal.
    fn validate_format_attribute(&mut self, attributes: &[Attribute]) {
        attributes.iter().for_each(|attribute| {
            if attribute.directive.as_str() == "format" {
                match attribute.arguments.len() {
                    // The format attribute must have arguments
                    0 => self.error_reporter.report_error(
                        "format attribute arguments cannot be empty".to_owned(),
                        Some(&attribute.location),
                    ),
                    _ => {
                        // Validate format attributes are allowed ones.
                        let options = ["Compact".to_owned(), "Sliced".to_owned()];
                        attribute
                            .arguments
                            .iter()
                            .filter(|arg| options.contains(arg))
                            .for_each(|arg| {
                                self.error_reporter.report_error(
                                    format!("invalid format attribute argument: {}", arg),
                                    Some(&attribute.location),
                                )
                            });
                    }
                }
            }
        })
    }
}

impl<'a> Visitor for AttributeValidator<'a> {
    fn visit_operation_start(&mut self, operation: &Operation) {
        self.validate_format_attribute(&operation.attributes);
    }
}

#[derive(Debug)]
struct EnumValidator<'a> {
    pub error_reporter: &'a mut ErrorReporter,
    pub encoding: Encoding,
}

impl EnumValidator<'_> {
    /// Validate that the enumerators are within the bounds of the specified underlying type.
    fn backing_type_bounds(&mut self, enum_def: &Enum) {
        match self.encoding {
            Encoding::Slice1 => {
                // Slice1 does not allow negative numbers.
                enum_def
                    .enumerators()
                    .iter()
                    .filter(|enumerator| enumerator.value < 0)
                    .for_each(|enumerator| {
                        self.error_reporter.report_error(
                            format!(
                            "invalid enumerator value on enumerator `{}`: enumerators must be non-negative",
                            &enumerator.identifier()
                        ),
                            Some(enumerator.location()),
                        );
                    });
                // Enums in Slice1 always have an underlying type of int32.
                enum_def
                .enumerators()
                .iter()
                .filter(|enumerator| enumerator.value > i32::MAX as i64)
                .for_each(|enumerator| {
                    self.error_reporter.report_error(
                        format!(
                            "invalid enumerator value on enumerator `{identifier}`: must be smaller than than {max}",
                            identifier = &enumerator.identifier(),
                            max = i32::MAX,

                        ),
                        Some(enumerator.location()),
                    );
                });
            }
            Encoding::Slice2 => {
                // Non-integrals are handled by `allowed_underlying_types`
                if enum_def.underlying_type(self.encoding).is_integral() {
                    let (min, max) = enum_def
                        .underlying_type(self.encoding)
                        .numeric_bounds()
                        .unwrap();
                    enum_def
                    .enumerators()
                    .iter()
                    .map(|enumerator| enumerator.value)
                    .filter(|value| *value <= min || *value >= max)
                    .for_each(|value| {
                        self.error_reporter.report_error(
                            format!(
                                "enumerator value '{value}' is out of bounds. The value must be between `{min}..{max}`, inclusive, for the underlying type `{underlying}`",
                                value = value,
                                underlying=enum_def.underlying_type(self.encoding).kind(),
                                min = min,
                                max = max,
                            ),
                            Some(&enum_def.location),
                        );
                    });
                }
            }
        }
    }

    /// Validate that the backing type specified for a Slice2 enums is an integral type.
    fn allowed_underlying_types(&mut self, enum_def: &Enum) {
        if self.encoding == Encoding::Slice2
            && !enum_def.underlying_type(self.encoding).is_integral()
        {
            self.error_reporter.report_error(
                format!(
                    "underlying type '{underlying}' is not allowed for enums",
                    underlying = enum_def.underlying_type(self.encoding).kind(),
                ),
                Some(&enum_def.location),
            );
        }
    }

    /// Validate that the enumerators for an enum are unique.
    fn enumerators_are_unique(&mut self, enumerators: Vec<&Enumerator>) {
        // The enumerators must be sorted by value first as we are using windowing to check the
        // n + 1 enumerator against the n enumerator. If the enumerators are sorted by value then
        // the windowing will reveal any duplicate enumerators.
        let mut sorted_enumerators = enumerators.clone();
        sorted_enumerators.sort_by_key(|m| m.value);
        sorted_enumerators.windows(2).for_each(|window| {
            if window[0].value == window[1].value {
                self.error_reporter.report_error(
                    format!(
                        "invalid enumerator value on enumerator `{}`: enumerators must be unique",
                        &window[1].identifier()
                    ),
                    Some(window[1].location()),
                );
                self.error_reporter.report_error(
                    format!(
                        "The enumerator `{}` has previous used the value `{}`",
                        &window[0].identifier(),
                        window[0].value
                    ),
                    Some(window[0].location()),
                );
            }
        })
    }

    /// Validate the the underlying type of an enum is not optional.
    fn underlying_type_cannot_be_optional(&mut self, enum_def: &Enum) {
        if let Some(ref typeref) = enum_def.underlying {
            if typeref.is_optional {
                self.error_reporter.report_error(
                    format!("underlying type '{}' cannot be optional: enums cannot have optional underlying types", typeref.type_string),
                    Some(&enum_def.location),
                );
            }
        }
    }
}

impl<'a> Visitor for EnumValidator<'a> {
    fn visit_enum_start(&mut self, enum_def: &Enum) {
        self.allowed_underlying_types(enum_def);
        self.backing_type_bounds(enum_def);
        self.enumerators_are_unique(enum_def.enumerators());
        self.underlying_type_cannot_be_optional(enum_def);
    }
}

#[derive(Debug)]
struct TagValidator<'a> {
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
        // Compact structs must be non-empty.
        if !struct_def.members.is_empty() {
            // Compact structs cannot have tagged data members.
            let mut has_tags = false;
            for member in struct_def.members() {
                if member.tag.is_some() {
                    self.error_reporter.report_error(
                        "tagged data members are not supported in compact structs\n\
                            consider removing the tag, or making the struct non-compact"
                            .to_owned(),
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
    }

    /// Validates that the tags are unique.
    fn tags_are_unique<M>(&mut self, members: &[&M])
    where
        M: Member,
    {
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
    fn have_optional_types<M>(&mut self, members: &[&M])
    where
        M: Member + ?Sized,
    {
        let tagged_members = members
            .iter()
            .filter(|member| member.tag().is_some())
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
    fn cannot_tag_classes<M>(&mut self, members: &[&M])
    where
        M: Member + ?Sized,
    {
        let tagged_members = members
            .iter()
            .filter(|member| member.tag().is_some())
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
    fn tagged_containers_cannot_contain_classes<M>(&mut self, members: &[&M])
    where
        M: Member + ?Sized,
    {
        let tagged_members = members
            .iter()
            .filter(|member| member.tag().is_some())
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
        for member_list in [operation_def.parameters(), operation_def.return_members()].iter() {
            self.parameter_order(member_list);
            self.have_optional_types(member_list);
            self.tags_are_unique(member_list);
            self.tagged_containers_cannot_contain_classes(member_list);
            self.cannot_tag_classes(member_list);
        }
    }
}
