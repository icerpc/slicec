// Copyright (c) ZeroC, Inc. All rights reserved.

use super::super::{Ast, Node};
use crate::error::ErrorReporter;
use crate::grammar::*;
use crate::slice_file::SliceFile;
use crate::supported_encodings::SupportedEncodings;
use std::collections::HashMap;

pub unsafe fn patch_ast(
    ast: &mut Ast,
    slice_files: &HashMap<String, SliceFile>,
    error_reporter: &mut ErrorReporter,
) -> Result<(), ()> {
    // Create a new encoding patcher.
    let mut patcher = EncodingPatcher {
        supported_encodings_cache: HashMap::new(),
        slice_files,
        error_reporter,
    };

    // Iterate through each node in the AST and patch any `supported_encodings` fields.
    // We only patch elements that internally cache what encodings they support, all other elements are skipped.
    //
    // For types where it's trivial to compute their encodings (primitives, sequences, etc.) we compute them on the fly
    // but other types that are computationally intensive (like containers) we compute it once (here) and cache it.
    for node in ast.as_mut_slice() {
        match node {
            Node::Struct(struct_ptr) => {
                let encodings = patcher.get_supported_encodings_for(struct_ptr.borrow());
                struct_ptr.borrow_mut().supported_encodings = Some(encodings);
            }
            Node::Exception(exception_ptr) => {
                let encodings = patcher.get_supported_encodings_for(exception_ptr.borrow());
                exception_ptr.borrow_mut().supported_encodings = Some(encodings);
            }
            Node::Class(class_ptr) => {
                let encodings = patcher.get_supported_encodings_for(class_ptr.borrow());
                class_ptr.borrow_mut().supported_encodings = Some(encodings);
            }
            Node::Interface(interface_ptr) => {
                let encodings = patcher.get_supported_encodings_for(interface_ptr.borrow());
                interface_ptr.borrow_mut().supported_encodings = Some(encodings);
            }
            Node::Enum(enum_ptr) => {
                let encodings = patcher.get_supported_encodings_for(enum_ptr.borrow());
                enum_ptr.borrow_mut().supported_encodings = Some(encodings);
            }
            Node::Trait(trait_ptr) => {
                let encodings = patcher.get_supported_encodings_for(trait_ptr.borrow());
                trait_ptr.borrow_mut().supported_encodings = Some(encodings);
            }
            Node::CustomType(custom_type_ptr) => {
                let encodings = patcher.get_supported_encodings_for(custom_type_ptr.borrow());
                custom_type_ptr.borrow_mut().supported_encodings = Some(encodings);
            }
            _ => {}
        }
    }

    error_reporter.get_state()
}

struct EncodingPatcher<'a> {
    supported_encodings_cache: HashMap<String, SupportedEncodings>,
    slice_files: &'a HashMap<String, SliceFile>,
    error_reporter: &'a mut ErrorReporter,
}

impl EncodingPatcher<'_> {
    fn get_supported_encodings_for<T>(&mut self, entity_def: &T) -> SupportedEncodings
    where
        T: Entity + ComputeSupportedEncodings,
    {
        // Check if the entity's supported encodings have already been computed.
        let type_id = entity_def.parser_scoped_identifier();
        if let Some(supported_encodings) = self.supported_encodings_cache.get(&type_id) {
            return supported_encodings.clone();
        }

        // Retrieve the encodings supported by the file that the entity is defined in.
        let file_name = &entity_def.location().file;
        let file_encoding = self.slice_files.get(file_name).unwrap().encoding();
        let mut supported_encodings = SupportedEncodings::new(match &file_encoding {
            Encoding::Slice1 => vec![Encoding::Slice1, Encoding::Slice2],
            Encoding::Slice2 => vec![Encoding::Slice2],
        });

        // Handle any type-specific encoding restrictions.
        //
        // This function can optionally return information to be emitted alongside a main error in specific cases.
        // Ex: Using a trait in a Slice1 file, we specifically say "traits are not supported by the Slice1 encoding".
        let additional_info = entity_def.compute_supported_encodings(self, &mut supported_encodings, &file_encoding);

        // Ensure the entity is supported by its file's Slice encoding.
        if !supported_encodings.supports(&file_encoding) {
            let message = format!(
                "{} `{}` is not supported by the Slice{} encoding",
                entity_def.kind(),
                entity_def.identifier(),
                file_encoding,
            );
            self.error_reporter.report_error(message, Some(entity_def.location()));
            self.emit_file_encoding_mismatch_error(entity_def);

            // Replace the supported encodings with a dummy that supports all encodings.
            // Otherwise everything that uses this type will also not be supported by the file's
            // encoding, causing a cascade of unhelpful error messages.
            supported_encodings = SupportedEncodings::dummy();
        }
        // Report any additional information as a note after reporting any errors above.
        if let Some(note) = additional_info {
            self.error_reporter.report_note(note, None);
        }

        // Cache and return this entity's supported encodings.
        self.supported_encodings_cache
            .insert(type_id, supported_encodings.clone());
        supported_encodings
    }

    fn get_supported_encodings_for_type_ref(
        &mut self,
        type_ref: &TypeRef<impl Type + ?Sized>,
        file_encoding: &Encoding,
        mut allow_nullable_with_slice_1: bool,
    ) -> SupportedEncodings {
        // If we encounter a type that isn't supported by its file's encodings, and we know a specific reason why, we
        // store an explanation in this variable. If it's empty, we report a generic message.
        let mut error_messages = Vec::new();

        let mut supported_encodings = match type_ref.concrete_type() {
            Types::Struct(struct_def) => self.get_supported_encodings_for(struct_def),
            Types::Exception(exception_def) => {
                let mut encodings = self.get_supported_encodings_for(exception_def);
                // Exceptions can't be used as a data type with Slice1.
                encodings.disable(Encoding::Slice1);
                if *file_encoding == Encoding::Slice1 {
                    error_messages.push("exceptions cannot be used as a data type with the Slice1 encoding".to_owned());
                }
                encodings
            }
            Types::Class(class_def) => {
                allow_nullable_with_slice_1 = true;
                self.get_supported_encodings_for(class_def)
            }
            Types::Interface(interface_def) => {
                allow_nullable_with_slice_1 = true;
                self.get_supported_encodings_for(interface_def)
            }
            Types::Enum(enum_def) => self.get_supported_encodings_for(enum_def),
            Types::Trait(trait_def) => self.get_supported_encodings_for(trait_def),
            Types::CustomType(custom_type) => self.get_supported_encodings_for(custom_type),
            Types::Sequence(sequence) => {
                // Sequences are supported by any encoding that supports their elements.
                self.get_supported_encodings_for_type_ref(&sequence.element_type, file_encoding, false)
            }
            Types::Dictionary(dictionary) => {
                // Dictionaries are supported by any encoding that supports their keys and values.
                let key_encodings =
                    self.get_supported_encodings_for_type_ref(&dictionary.key_type, file_encoding, false);
                let value_encodings =
                    self.get_supported_encodings_for_type_ref(&dictionary.value_type, file_encoding, false);

                let mut supported_encodings = key_encodings;
                supported_encodings.intersect_with(&value_encodings);
                supported_encodings
            }
            Types::Primitive(primitive) => primitive.supported_encodings(),
        };

        // Optional types aren't supported by the Slice1 encoding (with some exceptions).
        if !allow_nullable_with_slice_1 && type_ref.is_optional {
            supported_encodings.disable(Encoding::Slice1);
            if *file_encoding == Encoding::Slice1 {
                error_messages.push(
                    "optional types are not supported by the Slice1 encoding (except for classes, proxies, and with tags)".to_owned(),
                );
            }
        }

        // Ensure the Slice encoding of the file where the type is being used supports the type.
        if supported_encodings.supports(file_encoding) {
            supported_encodings
        } else {
            // If no specific reasons were given for the error, generate a generic one.
            if error_messages.is_empty() {
                error_messages.push(format!(
                    "the type `{}` is not supported by the Slice{} encoding",
                    &type_ref.type_string, file_encoding,
                ));
            }
            for message in error_messages {
                self.error_reporter.report_error(message, Some(type_ref.location()));
            }
            self.emit_file_encoding_mismatch_error(type_ref);

            // Return a dummy value that supports all encodings, instead of the real result.
            // Otherwise everything that uses this type will also not be supported by the file's
            // encoding, causing a cascade of unhelpful error messages.
            SupportedEncodings::dummy()
        }
    }

    fn emit_file_encoding_mismatch_error(&mut self, symbol: &impl Symbol) {
        let file_name = &symbol.location().file;
        let slice_file = self.slice_files.get(file_name).unwrap();

        // Emit a note explaining why the file has the slice encoding it does.
        if let Some(file_encoding) = &slice_file.encoding {
            let encoding_message = format!("file encoding was set to Slice{} here:", &file_encoding.version);
            self.error_reporter
                .report_note(encoding_message, Some(file_encoding.location()))
        } else {
            let encoding_message = format!("file is using the Slice{} encoding by default", Encoding::default());
            self.error_reporter.report_note(encoding_message, None);

            self.error_reporter.report_note(
                "to use a different encoding, specify it at the top of the slice file\nex: 'encoding = 1;'",
                None,
            )
        }
    }
}

trait ComputeSupportedEncodings {
    fn compute_supported_encodings(
        &self,
        patcher: &mut EncodingPatcher,
        supported_encodings: &mut SupportedEncodings,
        file_encoding: &Encoding,
    ) -> Option<&'static str>;
}

impl ComputeSupportedEncodings for Struct {
    fn compute_supported_encodings(
        &self,
        patcher: &mut EncodingPatcher,
        supported_encodings: &mut SupportedEncodings,
        file_encoding: &Encoding,
    ) -> Option<&'static str> {
        // Insert a dummy entry for the struct into the cache to prevent infinite lookup cycles.
        // If a cycle is encountered, the encodings will be computed incorrectly, but it's an
        // error for structs to be cyclic, so it's fine if the supported encodings are bogus.
        patcher
            .supported_encodings_cache
            .insert(self.parser_scoped_identifier(), SupportedEncodings::dummy());
        // Structs only support encodings that all its data members also support.
        for member in self.members() {
            supported_encodings.intersect_with(&patcher.get_supported_encodings_for_type_ref(
                member.data_type(),
                file_encoding,
                member.is_tagged(),
            ));
        }

        // Non-compact structs are not supported by the Slice1 encoding.
        if !self.is_compact {
            supported_encodings.disable(Encoding::Slice1);
            if *file_encoding == Encoding::Slice1 {
                return Some("structs must be `compact` to be supported by the Slice1 encoding");
            }
        }
        None
    }
}

impl ComputeSupportedEncodings for Exception {
    fn compute_supported_encodings(
        &self,
        patcher: &mut EncodingPatcher,
        supported_encodings: &mut SupportedEncodings,
        file_encoding: &Encoding,
    ) -> Option<&'static str> {
        // Insert a dummy entry for the exception into the cache to prevent infinite lookup cycles.
        // If a cycle is encountered, the encodings will be computed incorrectly, but it's an
        // error for exceptions to be cyclic, so it's fine if the supported encodings are bogus.
        patcher
            .supported_encodings_cache
            .insert(self.parser_scoped_identifier(), SupportedEncodings::dummy());
        // Exceptions only support encodings that all its data members also support
        // (including inherited ones).
        for member in self.all_members() {
            supported_encodings.intersect_with(&patcher.get_supported_encodings_for_type_ref(
                member.data_type(),
                file_encoding,
                member.is_tagged(),
            ));
        }

        // Exception inheritance is only supported by the Slice1 encoding.
        if self.base_exception().is_some() {
            supported_encodings.disable(Encoding::Slice2);
            if *file_encoding != Encoding::Slice1 {
                return Some("exception inheritance is only supported by the Slice1 encoding");
            }
        }
        None
    }
}

impl ComputeSupportedEncodings for Class {
    fn compute_supported_encodings(
        &self,
        patcher: &mut EncodingPatcher,
        supported_encodings: &mut SupportedEncodings,
        file_encoding: &Encoding,
    ) -> Option<&'static str> {
        // Insert a dummy entry for the class into the cache to prevent infinite lookup cycles.
        // Cycles are allowed with classes, but the only encoding that supports classes is Slice1,
        // so using this approach to break cycles will still yield the correct supported encodings.
        patcher
            .supported_encodings_cache
            .insert(self.parser_scoped_identifier(), SupportedEncodings::dummy());
        // Classes only support encodings that all its data members also support
        // (including inherited ones).
        for member in self.all_members() {
            supported_encodings.intersect_with(&patcher.get_supported_encodings_for_type_ref(
                member.data_type(),
                file_encoding,
                member.is_tagged(),
            ));
        }

        // Classes are only supported by the Slice1 encoding.
        supported_encodings.disable(Encoding::Slice2);
        if *file_encoding != Encoding::Slice1 {
            Some("classes are only supported by the Slice1 encoding")
        } else {
            None
        }
    }
}

impl ComputeSupportedEncodings for Interface {
    fn compute_supported_encodings(
        &self,
        patcher: &mut EncodingPatcher,
        _: &mut SupportedEncodings,
        file_encoding: &Encoding,
    ) -> Option<&'static str> {
        // Insert a dummy entry for the interface into the cache to prevent infinite lookup cycles.
        // The correct encoding is computed and inserted later.
        patcher
            .supported_encodings_cache
            .insert(self.parser_scoped_identifier(), SupportedEncodings::dummy());

        // Interfaces have no restrictions apart from those imposed by its file's encoding.
        // However, all the operations in an interface must support its file's encoding too.
        for operation in self.all_operations() {
            for member in operation.parameters_and_return_members() {
                // This method automatically emits errors for encoding mismatches.
                patcher.get_supported_encodings_for_type_ref(member.data_type(), file_encoding, member.is_tagged());

                // Streamed parameters are not supported by the Slice1 encoding.
                if member.is_streamed && *file_encoding == Encoding::Slice1 {
                    let message = "streamed parameters are not supported by the Slice1 encoding";
                    patcher.error_reporter.report_error(message, Some(member.location()));
                    patcher.emit_file_encoding_mismatch_error(member);
                }
            }
        }
        None
    }
}

impl ComputeSupportedEncodings for Enum {
    fn compute_supported_encodings(
        &self,
        patcher: &mut EncodingPatcher,
        supported_encodings: &mut SupportedEncodings,
        file_encoding: &Encoding,
    ) -> Option<&'static str> {
        if let Some(underlying_type) = &self.underlying {
            // Enums only support encodings that its underlying type also supports.
            supported_encodings.intersect_with(&patcher.get_supported_encodings_for_type_ref(
                underlying_type,
                file_encoding,
                false,
            ));

            // Enums with underlying types are not supported by the Slice1 encoding.
            supported_encodings.disable(Encoding::Slice1);
            if *file_encoding == Encoding::Slice1 {
                return Some("enums with underlying types are not supported by the Slice1 encoding");
            }
        }
        None
    }
}

impl ComputeSupportedEncodings for Trait {
    fn compute_supported_encodings(
        &self,
        _: &mut EncodingPatcher,
        supported_encodings: &mut SupportedEncodings,
        file_encoding: &Encoding,
    ) -> Option<&'static str> {
        // Traits are not supported by the Slice1 encoding.
        supported_encodings.disable(Encoding::Slice1);
        if *file_encoding == Encoding::Slice1 {
            Some("traits are not supported by the Slice1 encoding")
        } else {
            None
        }
    }
}

impl ComputeSupportedEncodings for CustomType {
    fn compute_supported_encodings(
        &self,
        _: &mut EncodingPatcher,
        _: &mut SupportedEncodings,
        _: &Encoding,
    ) -> Option<&'static str> {
        // Custom types are supported by all encodings.
        None
    }
}
