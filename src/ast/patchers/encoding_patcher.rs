// Copyright (c) ZeroC, Inc.

use super::super::Node;
use crate::compilation_state::CompilationState;
use crate::diagnostics::*;
use crate::grammar::*;
use crate::slice_file::SliceFile;
use crate::supported_encodings::SupportedEncodings;
use std::collections::HashMap;

pub unsafe fn patch_ast(compilation_state: &mut CompilationState) {
    // Create a new encoding patcher.
    let mut patcher = EncodingPatcher {
        supported_encodings_cache: HashMap::new(),
        slice_files: &mut compilation_state.files,
        diagnostic_reporter: &mut compilation_state.diagnostic_reporter,
    };

    // Iterate through each node in the AST and patch any `supported_encodings` fields.
    // We only patch elements that internally cache what encodings they support, all other elements are skipped.
    //
    // For types where it's trivial to compute their encodings (primitives, sequences, etc.) we compute them on the fly
    // but other types that are computationally intensive (like containers) we compute it once (here) and cache it.
    for node in compilation_state.ast.as_mut_slice() {
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
            Node::CustomType(custom_type_ptr) => {
                let encodings = patcher.get_supported_encodings_for(custom_type_ptr.borrow());
                custom_type_ptr.borrow_mut().supported_encodings = Some(encodings);
            }
            Node::TypeAlias(type_alias_ptr) => {
                let encodings = patcher.get_supported_encodings_for(type_alias_ptr.borrow());
                type_alias_ptr.borrow_mut().supported_encodings = Some(encodings);
            }
            _ => {}
        }
    }
}

struct EncodingPatcher<'a> {
    supported_encodings_cache: HashMap<String, SupportedEncodings>,
    slice_files: &'a HashMap<String, SliceFile>,
    diagnostic_reporter: &'a mut DiagnosticReporter,
}

impl EncodingPatcher<'_> {
    fn get_supported_encodings_for<T>(&mut self, entity_def: &T) -> SupportedEncodings
    where
        T: Entity + Type + ComputeSupportedEncodings,
    {
        // Check if the entity's supported encodings have already been computed.
        let type_id = entity_def.parser_scoped_identifier();
        if let Some(supported_encodings) = self.supported_encodings_cache.get(&type_id) {
            return supported_encodings.clone();
        }

        // Retrieve the encodings supported by the file that the entity is defined in.
        let file_name = &entity_def.span().file;
        let file_encoding = self.slice_files.get(file_name).unwrap().encoding();
        let mut supported_encodings = SupportedEncodings::new(match &file_encoding {
            Encoding::Slice1 => vec![Encoding::Slice1, Encoding::Slice2],
            Encoding::Slice2 => vec![Encoding::Slice2],
        });

        // Handle any type-specific encoding restrictions.
        //
        // This function can optionally return information to be emitted alongside a main error in specific cases.
        let additional_info = entity_def.compute_supported_encodings(self, &mut supported_encodings, &file_encoding);

        // Ensure the entity is supported by its file's Slice encoding.
        if !supported_encodings.supports(&file_encoding) {
            let error = Error::NotSupportedWithEncoding {
                kind: entity_def.kind().to_owned(),
                identifier: entity_def.identifier().to_owned(),
                encoding: file_encoding,
            };
            let mut notes = match additional_info {
                Some(message) => vec![Note {
                    message: message.to_owned(),
                    span: None,
                }],
                None => Vec::new(),
            };
            notes.extend(self.get_file_encoding_mismatch_note(entity_def));

            Diagnostic::new(error)
                .set_span(entity_def.span())
                .extend_notes(notes)
                .report(self.diagnostic_reporter);

            // Replace the supported encodings with a dummy that supports all encodings.
            // Otherwise everything that uses this type will also not be supported by the file's
            // encoding, causing a cascade of unhelpful error messages.
            supported_encodings = SupportedEncodings::dummy();
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
        container: Option<&dyn Entity>,
    ) -> SupportedEncodings {
        // If we encounter a type that isn't supported by its file's encodings, and we know a specific reason why, we
        // store an explanation in this variable. If it's empty, we report a generic message.
        let mut diagnostics = Vec::new();

        let mut supported_encodings = match type_ref.concrete_type() {
            Types::Struct(struct_def) => self.get_supported_encodings_for(struct_def),
            Types::Exception(exception_def) => {
                let mut encodings = self.get_supported_encodings_for(exception_def);
                // Exceptions can't be used as a data type with Slice1.
                encodings.disable(Encoding::Slice1);
                if *file_encoding == Encoding::Slice1 {
                    let diagnostic = Diagnostic::new(Error::ExceptionAsDataType).set_span(type_ref.span());
                    diagnostics.push(diagnostic);
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
            Types::CustomType(custom_type) => {
                allow_nullable_with_slice_1 = true;
                self.get_supported_encodings_for(custom_type)
            }
            Types::Sequence(sequence) => {
                // Sequences are supported by any encoding that supports their elements.
                self.get_supported_encodings_for_type_ref(&sequence.element_type, file_encoding, false, None)
            }
            Types::Dictionary(dictionary) => {
                // Dictionaries are supported by any encoding that supports their keys and values.
                let key_encodings =
                    self.get_supported_encodings_for_type_ref(&dictionary.key_type, file_encoding, false, None);
                let value_encodings =
                    self.get_supported_encodings_for_type_ref(&dictionary.value_type, file_encoding, false, None);

                let mut supported_encodings = key_encodings;
                supported_encodings.intersect_with(&value_encodings);
                supported_encodings
            }
            Types::Primitive(primitive) => {
                if *primitive == Primitive::AnyClass {
                    allow_nullable_with_slice_1 = true;
                }
                primitive.supported_encodings()
            }
        };

        // Optional types aren't supported by the Slice1 encoding (with some exceptions).
        if !allow_nullable_with_slice_1 && type_ref.is_optional {
            supported_encodings.disable(Encoding::Slice1);
            if *file_encoding == Encoding::Slice1 {
                let diagnostic = Diagnostic::new(Error::OptionalsNotSupported {
                    kind: type_ref.definition().kind().to_owned(),
                })
                .set_span(type_ref.span())
                .extend_notes(disallowed_optional_suggestion(type_ref, container));

                diagnostics.push(diagnostic);
            }
        }

        // Ensure the Slice encoding of the file where the type is being used supports the type.
        if supported_encodings.supports(file_encoding) {
            supported_encodings
        } else {
            // If no specific reasons were given for the error, generate a generic one.
            if diagnostics.is_empty() {
                let diagnostic = Diagnostic::new(Error::UnsupportedType {
                    kind: type_ref.type_string(),
                    encoding: *file_encoding,
                })
                .set_span(type_ref.span())
                .extend_notes(self.get_file_encoding_mismatch_note(type_ref));

                diagnostics.push(diagnostic);
            }

            for diagnostic in diagnostics {
                diagnostic.report(self.diagnostic_reporter);
            }

            // Return a dummy value that supports all encodings, instead of the real result.
            // Otherwise everything that uses this type will also not be supported by the file's
            // encoding, causing a cascade of unhelpful error messages.
            SupportedEncodings::dummy()
        }
    }

    fn get_file_encoding_mismatch_note(&self, symbol: &impl Symbol) -> Option<Note> {
        let file_name = &symbol.span().file;
        let slice_file = self.slice_files.get(file_name).unwrap();

        // Emit a note if the file is using the default encoding.

        // slice_file.encoding.map_or(f)

        match slice_file.encoding.as_ref() {
            Some(_) => None,
            None => Some(Note {
                message: format!("file is using the {} encoding by default", Encoding::default()),
                span: None,
            }),
        }
    }
}

fn disallowed_optional_suggestion(
    type_ref: &TypeRef<impl Type + ?Sized>,
    container: Option<&dyn Entity>,
) -> Option<Note> {
    let Some(container) = container else {
        return None;
    };

    let identifier = match container.concrete_entity() {
        Entities::Field(field) => match field.parent().unwrap().concrete_entity() {
            // If the field's parent is a class or exception, recommend using a tag.
            Entities::Class(..) | Entities::Exception(..) => Some(field.identifier()),
            _ => None,
        },
        // If container is an operation parameter, recommend using a tag.
        Entities::Parameter(parameter) => Some(parameter.identifier()),
        _ => None,
    };

    identifier.map(|identifier| Note {
        message: format!(
            "consider using a tag, e.g. 'tag(n) {}: {}'",
            identifier,
            type_ref.type_string()
        ),
        span: None,
    })
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
        // Structs only support encodings that all its fields also support.
        for field in self.fields() {
            supported_encodings.intersect_with(&patcher.get_supported_encodings_for_type_ref(
                field.data_type(),
                file_encoding,
                field.is_tagged(),
                Some(field),
            ));
        }

        // Non-compact structs are not supported by the Slice1 encoding.
        if !self.is_compact {
            supported_encodings.disable(Encoding::Slice1);
            if *file_encoding == Encoding::Slice1 {
                return Some("structs must be 'compact' to be supported by the Slice1 encoding");
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
        // Exceptions only support encodings that all its fields also support
        // (including inherited ones).
        for field in self.all_fields() {
            supported_encodings.intersect_with(&patcher.get_supported_encodings_for_type_ref(
                field.data_type(),
                file_encoding,
                field.is_tagged(),
                Some(field),
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
        // Classes only support encodings that all its fields also support
        // (including inherited ones).
        for field in self.all_fields() {
            supported_encodings.intersect_with(&patcher.get_supported_encodings_for_type_ref(
                field.data_type(),
                file_encoding,
                field.is_tagged(),
                Some(field),
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
                patcher.get_supported_encodings_for_type_ref(
                    member.data_type(),
                    file_encoding,
                    member.is_tagged(),
                    Some(member),
                );

                // Streamed parameters are not supported by the Slice1 encoding.
                if member.is_streamed && *file_encoding == Encoding::Slice1 {
                    Diagnostic::new(Error::StreamedParametersNotSupported)
                        .set_span(member.span())
                        .report(patcher.diagnostic_reporter)
                }
            }

            match &operation.throws {
                Throws::None => {}
                Throws::Specific(exception_type) => {
                    // Ensure the exception is supported by the operation's (file's) encoding.
                    let supported_encodings = patcher.get_supported_encodings_for(exception_type.definition());
                    if !supported_encodings.supports(file_encoding) {
                        Diagnostic::new(Error::UnsupportedType {
                            kind: exception_type.type_string(),
                            encoding: *file_encoding,
                        })
                        .set_span(exception_type.span())
                        .extend_notes(patcher.get_file_encoding_mismatch_note(exception_type))
                        .report(patcher.diagnostic_reporter)
                    }
                }
                Throws::AnyException => {
                    if *file_encoding != Encoding::Slice1 {
                        Diagnostic::new(Error::AnyExceptionNotSupported)
                            .set_span(operation.span())
                            .report(patcher.diagnostic_reporter)
                    }
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
        // TODO: rework all of this when we add enums with associated types.
        if let Some(underlying_type) = &self.underlying {
            // Enums only support encodings that its underlying type also supports.
            supported_encodings.intersect_with(&patcher.get_supported_encodings_for_type_ref(
                underlying_type,
                file_encoding,
                false,
                Some(self),
            ));

            // Enums with underlying types are not supported by the Slice1 encoding.
            supported_encodings.disable(Encoding::Slice1);
            if *file_encoding == Encoding::Slice1 {
                return Some("enums with underlying types are not supported by the Slice1 encoding");
            }
        } else {
            // Enums defined in a file using Slice2 must have an explicit underlying type.
            if *file_encoding == Encoding::Slice2 {
                // TODO: this isn't the correct error to emit, remove this when we add enums with associated values.
                Diagnostic::new(Error::EnumUnderlyingTypeNotSupported {
                    enum_identifier: self.identifier().to_owned(),
                    kind: None,
                })
                .set_span(self.span())
                .add_note(
                    format!(
                        "Slice2 enums must have an underlying type. e.g. 'enum {} : uint8'",
                        self.identifier(),
                    ),
                    None,
                )
                .report(patcher.diagnostic_reporter)
            }
        }
        None
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

impl ComputeSupportedEncodings for TypeAlias {
    fn compute_supported_encodings(
        &self,
        patcher: &mut EncodingPatcher,
        supported_encodings: &mut SupportedEncodings,
        file_encoding: &Encoding,
    ) -> Option<&'static str> {
        // Type aliases only support encodings that its underlying type also supports.
        supported_encodings.intersect_with(&patcher.get_supported_encodings_for_type_ref(
            &self.underlying,
            file_encoding,
            false,
            Some(self),
        ));
        None
    }
}
