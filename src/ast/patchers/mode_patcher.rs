// Copyright (c) ZeroC, Inc.

use super::super::Node;
use crate::compilation_state::CompilationState;
use crate::diagnostics::*;
use crate::grammar::*;
use crate::slice_file::SliceFile;
use crate::supported_modes::SupportedModes;
use std::collections::HashMap;

pub unsafe fn patch_ast(compilation_state: &mut CompilationState) {
    // Create a new encoding patcher.
    let mut patcher = ModePatcher {
        supported_modes_cache: HashMap::new(),
        slice_files: &mut compilation_state.files,
        diagnostic_reporter: &mut compilation_state.diagnostic_reporter,
    };

    // Iterate through each node in the AST and patch any `supported_modes` fields.
    // We only patch elements that internally cache what modes they support, all other elements are skipped.
    //
    // For types where it's trivial to compute their modes (primitives, sequences, etc.) we compute them on the fly
    // but other types that are computationally intensive (like containers) we compute it once (here) and cache it.
    for node in compilation_state.ast.as_mut_slice() {
        match node {
            Node::Struct(struct_ptr) => {
                let modes = patcher.get_supported_modes_for(struct_ptr.borrow());
                struct_ptr.borrow_mut().supported_modes = Some(modes);
            }
            Node::Exception(exception_ptr) => {
                let modes = patcher.get_supported_modes_for(exception_ptr.borrow());
                exception_ptr.borrow_mut().supported_modes = Some(modes);
            }
            Node::Class(class_ptr) => {
                let modes = patcher.get_supported_modes_for(class_ptr.borrow());
                class_ptr.borrow_mut().supported_modes = Some(modes);
            }
            Node::Interface(interface_ptr) => {
                let modes = patcher.get_supported_modes_for(interface_ptr.borrow());
                interface_ptr.borrow_mut().supported_modes = Some(modes);
            }
            Node::Enum(enum_ptr) => {
                let modes = patcher.get_supported_modes_for(enum_ptr.borrow());
                enum_ptr.borrow_mut().supported_modes = Some(modes);
            }
            Node::CustomType(custom_type_ptr) => {
                let modes = patcher.get_supported_modes_for(custom_type_ptr.borrow());
                custom_type_ptr.borrow_mut().supported_modes = Some(modes);
            }
            Node::TypeAlias(type_alias_ptr) => {
                let modes = patcher.get_supported_modes_for(type_alias_ptr.borrow());
                type_alias_ptr.borrow_mut().supported_modes = Some(modes);
            }
            _ => {}
        }
    }
}

struct ModePatcher<'a> {
    supported_modes_cache: HashMap<String, SupportedModes>,
    slice_files: &'a HashMap<String, SliceFile>,
    diagnostic_reporter: &'a mut DiagnosticReporter,
}

impl ModePatcher<'_> {
    fn get_supported_modes_for<T>(&mut self, entity_def: &T) -> SupportedModes
    where
        T: Entity + Type + ComputeSupportedModes,
    {
        // Check if the entity's supported modes have already been computed.
        let type_id = entity_def.parser_scoped_identifier();
        if let Some(supported_modes) = self.supported_modes_cache.get(&type_id) {
            return supported_modes.clone();
        }

        // Retrieve the modes supported by the file that the entity is defined in.
        let file_name = &entity_def.span().file;
        let file_mode = self.slice_files.get(file_name).unwrap().mode();
        let mut supported_modes = SupportedModes::new(match &file_mode {
            Mode::Slice1 => vec![Mode::Slice1, Mode::Slice2],
            Mode::Slice2 => vec![Mode::Slice2],
        });

        // Handle any type-specific encoding restrictions.
        //
        // This function can optionally return information to be emitted alongside a main error in specific cases.
        let additional_info = entity_def.compute_supported_modes(self, &mut supported_modes, &file_mode);

        // Ensure the entity is supported by its file's Slice encoding.
        if !supported_modes.supports(&file_mode) {
            let error = Error::NotSupportedWithMode {
                kind: entity_def.kind().to_owned(),
                identifier: entity_def.identifier().to_owned(),
                mode: file_mode.to_string(),
            };
            let mut notes = match additional_info {
                Some(message) => vec![Note {
                    message: message.to_owned(),
                    span: None,
                }],
                None => Vec::new(),
            };
            notes.extend(self.get_file_mode_mismatch_note(entity_def));

            Diagnostic::new(error)
                .set_span(entity_def.span())
                .extend_notes(notes)
                .report(self.diagnostic_reporter);

            // Replace the supported modes with a dummy that supports all modes.
            // Otherwise everything that uses this type will also not be supported by the file's
            // mode, causing a cascade of unhelpful error messages.
            supported_modes = SupportedModes::dummy();
        }

        // Cache and return this entity's supported modes.
        self.supported_modes_cache.insert(type_id, supported_modes.clone());
        supported_modes
    }

    fn get_supported_modes_for_type_ref(
        &mut self,
        type_ref: &TypeRef<impl Type + ?Sized>,
        file_mode: &Mode,
        mut allow_nullable_with_slice_1: bool,
        container: Option<&dyn Entity>,
    ) -> SupportedModes {
        // If we encounter a type that isn't supported by its file's modes, and we know a specific reason why, we
        // store an explanation in this variable. If it's empty, we report a generic message.
        let mut diagnostics = Vec::new();

        let mut supported_modes = match type_ref.concrete_type() {
            Types::Struct(struct_def) => self.get_supported_modes_for(struct_def),
            Types::Exception(exception_def) => {
                let mut modes = self.get_supported_modes_for(exception_def);
                // Exceptions can't be used as a data type with Slice1.
                modes.disable(Mode::Slice1);
                if *file_mode == Mode::Slice1 {
                    let diagnostic = Diagnostic::new(Error::ExceptionAsDataType).set_span(type_ref.span());
                    diagnostics.push(diagnostic);
                }
                modes
            }
            Types::Class(class_def) => {
                allow_nullable_with_slice_1 = true;
                self.get_supported_modes_for(class_def)
            }
            Types::Interface(interface_def) => {
                allow_nullable_with_slice_1 = true;
                self.get_supported_modes_for(interface_def)
            }
            Types::Enum(enum_def) => self.get_supported_modes_for(enum_def),
            Types::CustomType(custom_type) => {
                allow_nullable_with_slice_1 = true;
                self.get_supported_modes_for(custom_type)
            }
            Types::Sequence(sequence) => {
                // Sequences are supported by any mode that supports their elements.
                self.get_supported_modes_for_type_ref(&sequence.element_type, file_mode, false, None)
            }
            Types::Dictionary(dictionary) => {
                // Dictionaries are supported by any mode that supports their keys and values.
                let key_modes = self.get_supported_modes_for_type_ref(&dictionary.key_type, file_mode, false, None);
                let value_modes = self.get_supported_modes_for_type_ref(&dictionary.value_type, file_mode, false, None);

                let mut supported_modes = key_modes;
                supported_modes.intersect_with(&value_modes);
                supported_modes
            }
            Types::Primitive(primitive) => {
                if *primitive == Primitive::AnyClass {
                    allow_nullable_with_slice_1 = true;
                }
                primitive.supported_modes()
            }
        };

        // Optional types aren't supported by the Slice1 mode (with some exceptions).
        if !allow_nullable_with_slice_1 && type_ref.is_optional {
            supported_modes.disable(Mode::Slice1);
            if *file_mode == Mode::Slice1 {
                let diagnostic = Diagnostic::new(Error::OptionalsNotSupported {
                    kind: type_ref.definition().kind().to_owned(),
                })
                .set_span(type_ref.span())
                .extend_notes(disallowed_optional_suggestion(type_ref, container));

                diagnostics.push(diagnostic);
            }
        }

        // Ensure the Slice mode of the file where the type is being used supports the type.
        if supported_modes.supports(file_mode) {
            supported_modes
        } else {
            // If no specific reasons were given for the error, generate a generic one.
            if diagnostics.is_empty() {
                let diagnostic = Diagnostic::new(Error::UnsupportedType {
                    kind: type_ref.type_string(),
                    mode: file_mode.to_string(),
                })
                .set_span(type_ref.span())
                .extend_notes(self.get_file_mode_mismatch_note(type_ref));

                diagnostics.push(diagnostic);
            }

            for diagnostic in diagnostics {
                diagnostic.report(self.diagnostic_reporter);
            }

            // Return a dummy value that supports all encodings, instead of the real result.
            // Otherwise everything that uses this type will also not be supported by the file's
            // encoding, causing a cascade of unhelpful error messages.
            SupportedModes::dummy()
        }
    }

    fn get_file_mode_mismatch_note(&self, symbol: &impl Symbol) -> Option<Note> {
        let file_name = &symbol.span().file;
        let slice_file = self.slice_files.get(file_name).unwrap();

        // Emit a note if the file is using the default mode.

        match slice_file.mode.as_ref() {
            Some(_) => None,
            None => Some(Note {
                message: format!("file is using {} mode by default", Mode::default()),
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
        Entities::Field(field) => match field.parent().concrete_entity() {
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
            type_ref.type_string(),
        ),
        span: None,
    })
}

trait ComputeSupportedModes {
    fn compute_supported_modes(
        &self,
        patcher: &mut ModePatcher,
        supported_modes: &mut SupportedModes,
        file_mode: &Mode,
    ) -> Option<&'static str>;
}

impl ComputeSupportedModes for Struct {
    fn compute_supported_modes(
        &self,
        patcher: &mut ModePatcher,
        supported_modes: &mut SupportedModes,
        file_mode: &Mode,
    ) -> Option<&'static str> {
        // Insert a dummy entry for the struct into the cache to prevent infinite lookup cycles.
        // If a cycle is encountered, the modes will be computed incorrectly, but it's an
        // error for structs to be cyclic, so it's fine if the supported modes are bogus.
        patcher
            .supported_modes_cache
            .insert(self.parser_scoped_identifier(), SupportedModes::dummy());
        // Structs only support modes that all its fields also support.
        for field in self.fields() {
            supported_modes.intersect_with(&patcher.get_supported_modes_for_type_ref(
                field.data_type(),
                file_mode,
                field.is_tagged(),
                Some(field),
            ));
        }

        // Non-compact structs are not supported by the Slice1 mode.
        if !self.is_compact {
            supported_modes.disable(Mode::Slice1);
            if *file_mode == Mode::Slice1 {
                return Some("structs must be 'compact' to be supported by the Slice1 mode");
            }
        }
        None
    }
}

impl ComputeSupportedModes for Exception {
    fn compute_supported_modes(
        &self,
        patcher: &mut ModePatcher,
        supported_modes: &mut SupportedModes,
        file_mode: &Mode,
    ) -> Option<&'static str> {
        // Insert a dummy entry for the exception into the cache to prevent infinite lookup cycles.
        // If a cycle is encountered, the modes will be computed incorrectly, but it's an
        // error for exceptions to be cyclic, so it's fine if the supported modes are bogus.
        patcher
            .supported_modes_cache
            .insert(self.parser_scoped_identifier(), SupportedModes::dummy());
        // Exceptions only support modes that all its fields also support
        // (including inherited ones).
        for field in self.all_fields() {
            supported_modes.intersect_with(&patcher.get_supported_modes_for_type_ref(
                field.data_type(),
                file_mode,
                field.is_tagged(),
                Some(field),
            ));
        }

        // Exception inheritance is only supported by the Slice1 mode.
        if self.base_exception().is_some() {
            supported_modes.disable(Mode::Slice2);
            if *file_mode != Mode::Slice1 {
                return Some("exception inheritance is only supported by the Slice1 mode");
            }
        }
        None
    }
}

impl ComputeSupportedModes for Class {
    fn compute_supported_modes(
        &self,
        patcher: &mut ModePatcher,
        supported_modes: &mut SupportedModes,
        file_mode: &Mode,
    ) -> Option<&'static str> {
        // Insert a dummy entry for the class into the cache to prevent infinite lookup cycles.
        // Cycles are allowed with classes, but the only mode that supports classes is Slice1,
        // so using this approach to break cycles will still yield the correct supported modes.
        patcher
            .supported_modes_cache
            .insert(self.parser_scoped_identifier(), SupportedModes::dummy());
        // Classes only support modes that all its fields also support
        // (including inherited ones).
        for field in self.all_fields() {
            supported_modes.intersect_with(&patcher.get_supported_modes_for_type_ref(
                field.data_type(),
                file_mode,
                field.is_tagged(),
                Some(field),
            ));
        }

        // Classes are only supported by the Slice1 mode.
        supported_modes.disable(Mode::Slice2);
        if *file_mode != Mode::Slice1 {
            Some("classes are only supported by the Slice1 mode")
        } else {
            None
        }
    }
}

impl ComputeSupportedModes for Interface {
    fn compute_supported_modes(
        &self,
        patcher: &mut ModePatcher,
        _: &mut SupportedModes,
        file_mode: &Mode,
    ) -> Option<&'static str> {
        // Insert a dummy entry for the interface into the cache to prevent infinite lookup cycles.
        // The correct mode is computed and inserted later.
        patcher
            .supported_modes_cache
            .insert(self.parser_scoped_identifier(), SupportedModes::dummy());

        // Interfaces have no restrictions apart from those imposed by its file's mode.
        // However, all the operations in an interface must support its file's mode too.
        for operation in self.all_operations() {
            for member in operation.parameters_and_return_members() {
                // This method automatically emits errors for mode mismatches.
                patcher.get_supported_modes_for_type_ref(
                    member.data_type(),
                    file_mode,
                    member.is_tagged(),
                    Some(member),
                );

                // Streamed parameters are not supported by the Slice1 mode.
                if member.is_streamed && *file_mode == Mode::Slice1 {
                    Diagnostic::new(Error::StreamedParametersNotSupported)
                        .set_span(member.span())
                        .report(patcher.diagnostic_reporter)
                }
            }

            match &operation.throws {
                Throws::None => {}
                Throws::Specific(exception_type) => {
                    // Ensure the exception is supported by the operation's (file's) mode.
                    let supported_modes = patcher.get_supported_modes_for(exception_type.definition());
                    if !supported_modes.supports(file_mode) {
                        Diagnostic::new(Error::UnsupportedType {
                            kind: exception_type.type_string(),
                            mode: file_mode.to_string(),
                        })
                        .set_span(exception_type.span())
                        .extend_notes(patcher.get_file_mode_mismatch_note(exception_type))
                        .report(patcher.diagnostic_reporter)
                    }
                }
                Throws::AnyException => {
                    if *file_mode != Mode::Slice1 {
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

impl ComputeSupportedModes for Enum {
    fn compute_supported_modes(
        &self,
        patcher: &mut ModePatcher,
        supported_modes: &mut SupportedModes,
        file_mode: &Mode,
    ) -> Option<&'static str> {
        // TODO: rework all of this when we add enums with associated types.
        if let Some(underlying_type) = &self.underlying {
            // Enums only support modes that its underlying type also supports.
            supported_modes.intersect_with(&patcher.get_supported_modes_for_type_ref(
                underlying_type,
                file_mode,
                false,
                Some(self),
            ));

            // Enums with underlying types are not supported by the Slice1 mode.
            supported_modes.disable(Mode::Slice1);
            if *file_mode == Mode::Slice1 {
                return Some("enums with underlying types are not supported by the Slice1 mode");
            }
        } else {
            // Enums defined in a file using Slice2 must have an explicit underlying type.
            if *file_mode == Mode::Slice2 {
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

impl ComputeSupportedModes for CustomType {
    fn compute_supported_modes(&self, _: &mut ModePatcher, _: &mut SupportedModes, _: &Mode) -> Option<&'static str> {
        // Custom types are supported by all modes.
        None
    }
}

impl ComputeSupportedModes for TypeAlias {
    fn compute_supported_modes(
        &self,
        patcher: &mut ModePatcher,
        supported_modes: &mut SupportedModes,
        file_mode: &Mode,
    ) -> Option<&'static str> {
        // Type aliases only support modes that its underlying type also supports.
        supported_modes.intersect_with(&patcher.get_supported_modes_for_type_ref(
            &self.underlying,
            file_mode,
            false,
            Some(self),
        ));
        None
    }
}
