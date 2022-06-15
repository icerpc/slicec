// Copyright (c) ZeroC, Inc. All rights reserved.

/// TODO EVERYTHING IN HERE NEEDS COMMENTS!!!

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
        entity_def.compute_supported_encodings(
            self,
            &mut supported_encodings,
            &file_encoding,
        );

        // Ensure the entity is supported by its file's Slice encoding.
        if !supported_encodings.supports(&file_encoding) {
            let message = format!(
                "`{}` is not supported by the Slice {} encoding",
                entity_def.identifier(),
                file_encoding,
            );
            self.emit_file_encoding_mismatch_error(message, entity_def);

            // Replace the supported encodings with a dummy that supports all encodings.
            // Otherwise everything that uses this type will also not be supported by the file's
            // encoding, causing a cascade of unhelpful error messages.
            supported_encodings = SupportedEncodings::dummy();
        }

        // Cache and return this entity's supported encodings.
        self.supported_encodings_cache.insert(type_id, supported_encodings.clone());
        supported_encodings
    }

    fn get_supported_encodings_for_type_ref(
        &mut self,
        type_ref: &TypeRef<impl Type + ?Sized>,
        file_encoding: &Encoding,
        mut allow_nullable_with_slice_1: bool,
    ) -> SupportedEncodings {
        println!("{:?}", type_ref);
        let mut supported_encodings = match type_ref.concrete_type() {
            Types::Struct(struct_def) => self.get_supported_encodings_for(struct_def),
            Types::Exception(exception_def) => self.get_supported_encodings_for(exception_def),
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
                self.get_supported_encodings_for_type_ref(
                    &sequence.element_type,
                    file_encoding,
                    false,
                )
            }
            Types::Dictionary(dictionary) => {
                // Dictionaries are supported by any encoding that supports their keys and values.
                let key_encodings = self.get_supported_encodings_for_type_ref(
                    &dictionary.key_type,
                    file_encoding,
                    false,
                );
                let value_encodings = self.get_supported_encodings_for_type_ref(
                    &dictionary.value_type,
                    file_encoding,
                    false,
                );
                let mut supported_encodings = key_encodings;
                supported_encodings.intersect_with(&value_encodings);
                supported_encodings
            }
            Types::Primitive(primitive) => primitive.supported_encodings(),
        };

        // Optional types aren't supported by the Slice 1 encoding (with some exceptions).
        if !allow_nullable_with_slice_1 && type_ref.is_optional {
            supported_encodings.disable(Encoding::Slice1);
            if *file_encoding == Encoding::Slice1 {
                let m = "optional types are not supported by the Slice 1 encoding (except for classes and proxies)";
                self.error_reporter.report_error(m, None);
            }
        }

        // Ensure the Slice encoding of the file where the type is being used supports the type.
        if supported_encodings.supports(file_encoding) {
            supported_encodings
        } else {
            let message = format!(
                "the type `{}` is not supported by the Slice {} encoding",
                &type_ref.type_string,
                file_encoding,
            );
            self.emit_file_encoding_mismatch_error(message, type_ref);

            // Return a dummy value that supports all encodings, instead of the real result.
            // Otherwise everything that uses this type will also not be supported by the file's
            // encoding, causing a cascade of unhelpful error messages.
            SupportedEncodings::dummy()
        }
    }

    fn emit_file_encoding_mismatch_error(&mut self, message: String, symbol: &impl Symbol) {
        // Report the actual error message.
        self.error_reporter.report_error(message, Some(symbol.location()));

        let file_name = &symbol.location().file;
        let slice_file = self.slice_files.get(file_name).unwrap();

        // Emit a note explaining why the file has the slice encoding it does.
        if let Some(file_encoding) = &slice_file.encoding {
            let encoding_message = format!(
                "file encoding was set to the Slice {} encoding here:",
                &file_encoding.version,
            );
            self.error_reporter.report_note(
                encoding_message,
                Some(file_encoding.location()),
            )
        } else {
            let encoding_message = format!(
                "file is using the Slice {} encoding by default",
                Encoding::default(),
            );
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
    );
}

impl ComputeSupportedEncodings for Struct {
    fn compute_supported_encodings(
        &self,
        patcher: &mut EncodingPatcher,
        supported_encodings: &mut SupportedEncodings,
        file_encoding: &Encoding,
    ) {
        // Non-compact structs are not supported by the Slice 1 encoding.
        if !self.is_compact {
            supported_encodings.disable(Encoding::Slice1);
            if *file_encoding == Encoding::Slice1 {
                let message = "structs must be `compact` to be supported by the Slice 1 encoding";
                patcher.error_reporter.report_error(message, None);
            }
        }

        // Insert a dummy entry for the struct into the cache to prevent infinite lookup cycles.
        // If a cycle is encountered, the encodings will be computed incorrectly, but it's an
        // error for structs to be cyclic, so it's fine if the supported encodings are bogus.
        patcher.supported_encodings_cache.insert(
            self.parser_scoped_identifier(),
            SupportedEncodings::dummy(),
        );
        // Structs only support encodings that all its data members also support.
        for member in self.members() {
            supported_encodings.intersect_with(
                &patcher.get_supported_encodings_for_type_ref(
                    member.data_type(),
                    file_encoding,
                    member.is_tagged(),
                )
            );
        }
    }
}

impl ComputeSupportedEncodings for Exception {
    fn compute_supported_encodings(
        &self,
        patcher: &mut EncodingPatcher,
        supported_encodings: &mut SupportedEncodings,
        file_encoding: &Encoding,
    ) {
        // Exception inheritance is only supported by the Slice 1 encoding.
        if self.base_exception().is_some() {
            supported_encodings.disable(Encoding::Slice2);
            if *file_encoding != Encoding::Slice1 {
                let message = "exception inheritance is only supported by the Slice 1 encoding";
                patcher.error_reporter.report_error(message, None);
            }
        }

        // Insert a dummy entry for the exception into the cache to prevent infinite lookup cycles.
        // If a cycle is encountered, the encodings will be computed incorrectly, but it's an
        // error for exceptions to be cyclic, so it's fine if the supported encodings are bogus.
        patcher.supported_encodings_cache.insert(
            self.parser_scoped_identifier(),
            SupportedEncodings::dummy(),
        );
        // Exceptions only support encodings that all its data members also support
        // (including inherited ones).
        for member in self.all_members() {
            supported_encodings.intersect_with(
                &patcher.get_supported_encodings_for_type_ref(
                    member.data_type(),
                    file_encoding,
                    member.is_tagged(),
                )
            );
        }
    }
}

impl ComputeSupportedEncodings for Class {
    fn compute_supported_encodings(
        &self,
        patcher: &mut EncodingPatcher,
        supported_encodings: &mut SupportedEncodings,
        file_encoding: &Encoding,
    ) {
        // Classes are only supported by the Slice 1 encoding.
        supported_encodings.disable(Encoding::Slice2);
        if *file_encoding != Encoding::Slice1 {
            let message = "classes are only supported by the Slice 1 encoding";
            patcher.error_reporter.report_error(message, None);
        }

        // Insert a dummy entry for the class into the cache to prevent infinite lookup cycles.
        // Cycles are allowed with classes, but the only encoding that supports classes is Slice1,
        // so using this approach to break cycles will still yield the correct supported encodings.
        patcher.supported_encodings_cache.insert(
            self.parser_scoped_identifier(),
            SupportedEncodings::dummy(),
        );
        // Classes only support encodings that all its data members also support
        // (including inherited ones).
        for member in self.all_members() {
            supported_encodings.intersect_with(
                &patcher.get_supported_encodings_for_type_ref(
                    member.data_type(),
                    file_encoding,
                    member.is_tagged(),
                )
            );
        }
    }
}

impl ComputeSupportedEncodings for Interface {
    fn compute_supported_encodings(
        &self,
        patcher: &mut EncodingPatcher,
        _: &mut SupportedEncodings,
        file_encoding: &Encoding,
    ) {
        // Interfaces have no restrictions apart from those imposed by its file's encoding.
        // However, all the operations in an interface must support its file's encoding too.
        for operation in self.all_operations() {
            for member in operation.parameters_and_return_members() {
                // This method automatically emits errors for encoding mismatches.
                patcher.get_supported_encodings_for_type_ref(
                    member.data_type(),
                    file_encoding,
                    member.is_tagged(),
                );

                // Streamed parameters are not supported by the Slice 1 encoding.
                if member.is_streamed && *file_encoding == Encoding::Slice1 {
                    let message = "streamed parameters are not supported by the Slice 1 encoding";
                    patcher.emit_file_encoding_mismatch_error(message.to_owned(), member);
                }
            }
        }
    }
}

impl ComputeSupportedEncodings for Enum {
    fn compute_supported_encodings(
        &self,
        patcher: &mut EncodingPatcher,
        supported_encodings: &mut SupportedEncodings,
        file_encoding: &Encoding,
    ) {
        if let Some(underlying_type) = &self.underlying {
            // Enums with underlying types are not supported by the Slice 1 encoding.
            supported_encodings.disable(Encoding::Slice1);
            if *file_encoding == Encoding::Slice1 {
                let message = "enums with underlying types are not supported by the Slice 1 encoding";
                patcher.error_reporter.report_error(message, None);
            }

            // Enums only support encodings that its underlying type also supports.
            supported_encodings.intersect_with(
                &patcher.get_supported_encodings_for_type_ref(
                    underlying_type,
                    file_encoding,
                    false,
                )
            )
        }
    }
}

impl ComputeSupportedEncodings for Trait {
    fn compute_supported_encodings(
        &self,
        patcher: &mut EncodingPatcher,
        supported_encodings: &mut SupportedEncodings,
        file_encoding: &Encoding,
    ) {
        // Traits are not supported by the Slice 1 encoding.
        supported_encodings.disable(Encoding::Slice1);
        if *file_encoding == Encoding::Slice1 {
            let message = "traits are not supported by the Slice 1 encoding";
            patcher.error_reporter.report_error(message, None);
        }
    }
}

impl ComputeSupportedEncodings for CustomType {
    fn compute_supported_encodings(
        &self,
        patcher: &mut EncodingPatcher,
        supported_encodings: &mut SupportedEncodings,
        file_encoding: &Encoding,
    ) {
        // Custom types are not supported by the Slice 1 encoding.
        supported_encodings.disable(Encoding::Slice1);
        if *file_encoding == Encoding::Slice1 {
            let message = "custom types are not supported by the Slice 1 encoding";
            patcher.error_reporter.report_error(message, None);
        }
    }
}
