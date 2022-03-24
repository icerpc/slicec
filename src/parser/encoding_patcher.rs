// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::ast::Ast;
use crate::grammar::*;
use crate::ptr_util::OwnedPtr;
use crate::ptr_visitor::PtrVisitor;
use crate::slice_file::SliceFile;
use crate::supported_encodings::SupportedEncodings;
use crate::visitor::Visitor;
use std::collections::HashMap;

pub(super) fn patch_encodings(slice_files: &HashMap<String, SliceFile>, ast: &mut Ast) {
    let mut patcher = EncodingPatcher {
        slice_files,
        dependency_stack: Vec::new(),
        supported_encodings: HashMap::new(),
    };

    // First, visit everything immutably to check for cycles and compute the supported encodings.
    for slice_file in slice_files.values() {
        slice_file.visit_with(&mut patcher);
    }

    // Then directly visit everything mutably to actually patch in the supported encodings.
    for module in &mut ast.ast {
        unsafe { module.visit_ptr_with(&mut patcher); }
    }
}

fn get_encodings_supported_by(file_encoding: &SliceEncoding) -> SupportedEncodings {
    SupportedEncodings::new(match file_encoding {
        SliceEncoding::Slice11 => vec![SliceEncoding::Slice11, SliceEncoding::Slice2],
        SliceEncoding::Slice2 => vec![SliceEncoding::Slice2],
    })
}

struct EncodingPatcher<'files> {
    slice_files: &'files HashMap<String, SliceFile>,
    // Stack of all the types we've currently seen in the dependency chain we're currently checking.
    dependency_stack: Vec<String>,
    // Map of all the encodings supported by a type (key is the type's type-id).
    supported_encodings: HashMap<String, SupportedEncodings>,
}

impl<'files> EncodingPatcher<'files> {
    fn add_supported_encodings_entry(
        &mut self,
        type_def: &(impl Entity + Type),
        type_id: String,
        supported_encodings: SupportedEncodings,
    ) {
        if supported_encodings.is_empty() {
            let message = format!(
                "type '{}' isn't supportable by any Slice encoding",
                type_id,
            );
            crate::report_error(message, Some(type_def.location()));
        }
        self.supported_encodings.insert(type_id, supported_encodings);
    }

    fn check_for_cycle<T: Entity + Type>(&mut self, type_def: &T, type_id: &String) -> bool {
        // Check if the type is self-referential and not a class (we allow cycles in classes).
        if let Some(i) = self.dependency_stack.iter().position(|x| x == type_id) {
            let message = format!(
                "self-referential type {type_id} has infinite size.\n{cycle_string}",
                type_id = &type_id,
                cycle_string = &self.dependency_stack[i..].join(" -> "),
            );
            crate::report_error(message, Some(type_def.location()));

            // If the type is cyclic, we say it's supported by all encodings even though it's
            // supported by non. Otherwise, types using this one would also support no encodings,
            // leading to many spurious error messages.
            self.supported_encodings.insert(
                type_id.to_owned(),
                SupportedEncodings::dummy(),
            );

            true
        } else {
            false
        }
    }

    fn get_file_encoding_for(&self, symbol: &impl Symbol) -> SliceEncoding {
        let file_name = &symbol.location().file;
        let slice_file = self.slice_files.get(file_name).unwrap();
        slice_file.encoding()
    }

    fn resolve_encodings_supported_by_type(
        &mut self,
        file_encoding: &SliceEncoding,
        type_ref: &TypeRef,
        is_tagged: bool,
    ) -> SupportedEncodings {
        // True for proxies and classes, false for everything else.
        let mut is_nullable = false;

        let mut supported_encodings = match type_ref.concrete_typeref() {
            TypeRefs::Struct(struct_ref) => {
                let type_id = struct_ref.module_scoped_identifier();
                // Compute the type's supported encodings if they haven't been computed yet.
                if let Some(encodings) = self.supported_encodings.get(&type_id) {
                    encodings
                } else {
                    struct_ref.visit_with(self);
                    self.supported_encodings.get(&type_id).unwrap()
                }.clone()
            }
            TypeRefs::Class(_) => {
                is_nullable = true;
                // Classes can only be declared in a 1.1 encoded file. A 1.1 encoded file can only
                // reference entities from other 1.1 encoded files. So it's impossible for a class
                // to contain non-1.1 things in it. So classes are always supported by (only) 1.1.
                SupportedEncodings::new(vec![SliceEncoding::Slice11])
            }
            TypeRefs::Exception(exception_ref) => {
                let type_id = exception_ref.module_scoped_identifier();
                // Compute the type's supported encodings if they haven't been computed yet.
                if let Some(encodings) = self.supported_encodings.get(&type_id) {
                    encodings
                } else {
                    exception_ref.visit_with(self);
                    self.supported_encodings.get(&type_id).unwrap()
                }.clone()
            }
            TypeRefs::Interface(interface_ref) => {
                is_nullable = true;

                let type_id = interface_ref.module_scoped_identifier();
                // Compute the type's supported encodings if they haven't been computed yet.
                if let Some(encodings) = self.supported_encodings.get(&type_id) {
                    encodings
                } else {
                    interface_ref.visit_with(self);
                    self.supported_encodings.get(&type_id).unwrap()
                }.clone()
            }
            TypeRefs::Enum(enum_ref) => {
                let type_id = enum_ref.module_scoped_identifier();
                // Compute the type's supported encodings if they haven't been computed yet.
                if let Some(encodings) = self.supported_encodings.get(&type_id) {
                    encodings
                } else {
                    enum_ref.visit_with(self);
                    self.supported_encodings.get(&type_id).unwrap()
                }.clone()
            }
            TypeRefs::Trait(trait_ref) => {
                let type_id = trait_ref.module_scoped_identifier();
                // Compute the type's supported encodings if they haven't been computed yet.
                if let Some(encodings) = self.supported_encodings.get(&type_id) {
                    encodings
                } else {
                    trait_ref.visit_with(self);
                    self.supported_encodings.get(&type_id).unwrap()
                }.clone()
            }
            TypeRefs::Sequence(sequence_ref) => {
                self.resolve_encodings_supported_by_type(
                    file_encoding,
                    &sequence_ref.element_type,
                    false
                )
            }
            TypeRefs::Dictionary(dictionary_ref) => {
                let mut key_encodings = self.resolve_encodings_supported_by_type(
                    file_encoding,
                    &dictionary_ref.key_type,
                    false,
                );
                let value_encodings = self.resolve_encodings_supported_by_type(
                    file_encoding,
                    &dictionary_ref.value_type,
                    false,
                );
                key_encodings.intersect_with(&value_encodings);
                key_encodings
            }
            TypeRefs::Primitive(primitive_ref) => {
                let primitive_def = primitive_ref.definition();

                // Check that the primitive is supported by the
                // file's encoding in which it is being used.
                let encodings = primitive_def.supported_encodings();
                if !encodings.supports(&file_encoding) {
                    let message = format!(
                        "'{}' is not supported by the Slice {} encoding",
                        primitive_def.kind(),
                        file_encoding
                    );
                    crate::report_error(message, Some(type_ref.location()));
                    self.print_file_encoding_note(type_ref);
                    SupportedEncodings::dummy()
                } else {
                    encodings
                }
            }
        };

        // Non-tagged optional types aren't supported by the Slice 1.1 encoding.
        if !is_tagged && !is_nullable && type_ref.is_optional {
            supported_encodings.disable_11();
            if *file_encoding == SliceEncoding::Slice11 {
                crate::report_error(
                    "optional types can only be used with tags in the Slice 1.1 encoding".to_owned(),
                    Some(type_ref.location())
                );
                self.print_file_encoding_note(type_ref);
                supported_encodings = SupportedEncodings::dummy();
            }
        }

        supported_encodings
    }

    fn print_file_encoding_note(&self, symbol: &impl Symbol) {
        let file_name = &symbol.location().file;
        let slice_file = self.slice_files.get(file_name).unwrap();

        if let Some(file_encoding) = &slice_file.encoding {
            let message = format!(
                "file encoding was set to the Slice {} encoding here:",
                &file_encoding.version,
            );
            crate::report_note(message, Some(file_encoding.location()));
        } else {
            let message = format!(
                "file is using the Slice {} encoding by default",
                SliceEncoding::default(),
            );
            crate::report_note(message, None);

            crate::report_note(
                r#"to use a different encoding, specify it at the top of the slice file
ex: 'encoding = 1.1;'"#.to_owned(),
                None,
            )
        }
    }
}

impl<'files> Visitor for EncodingPatcher<'files> {
    fn visit_struct_start(&mut self, struct_def: &Struct) {
        let type_id = struct_def.module_scoped_identifier();
        let file_encoding = self.get_file_encoding_for(struct_def);
        let mut supported_encodings = get_encodings_supported_by(&file_encoding);

        if self.check_for_cycle(struct_def, &type_id) {
            // If the type is cyclic, return early to avoid an infinite loop.
            // `check_for_cycle` will already of reported an error message.
            return;
        }

        // Resolve the supported encodings for the data members.
        self.dependency_stack.push(type_id);
        for member in struct_def.members() {
            let member_supported_encodings = self.resolve_encodings_supported_by_type(
                &file_encoding,
                &member.data_type,
                member.tag.is_some(),
            );
            supported_encodings.intersect_with(&member_supported_encodings);
        }

        // Non-compact structs are not supported by the 1.1 encoding.
        if !struct_def.is_compact {
            supported_encodings.disable_11();
            if file_encoding == SliceEncoding::Slice11 {
                crate::report_error(
                    "non-compact structs are not supported by the Slice 1.1 encoding".to_owned(),
                    Some(struct_def.location()),
                );
                self.print_file_encoding_note(struct_def);
                supported_encodings = SupportedEncodings::dummy();
            }
        }

        // Pop the type-id off the stack, and store this struct's supported encodings.
        let type_id = self.dependency_stack.pop().unwrap();
        self.add_supported_encodings_entry(struct_def, type_id, supported_encodings);
    }

    fn visit_class_start(&mut self, class_def: &Class) {
        let type_id = class_def.module_scoped_identifier();
        let file_encoding = self.get_file_encoding_for(class_def);
        let mut supported_encodings = get_encodings_supported_by(&file_encoding);

        // We allow cycles with classes, so we don't check for them here.

        // Resolve the supported encodings for the data members.
        self.dependency_stack.push(type_id);
        for member in class_def.members() {
            let member_supported_encodings = self.resolve_encodings_supported_by_type(
                &file_encoding,
                &member.data_type,
                member.tag.is_some(),
            );
            supported_encodings.intersect_with(&member_supported_encodings);
        }

        // Classes are only supported by the 1.1 encoding.
        supported_encodings.disable_2();
        if file_encoding == SliceEncoding::Slice2 {
            crate::report_error(
                "classes are only supported by the Slice 1.1 encoding".to_owned(),
                Some(class_def.location()),
            );
            self.print_file_encoding_note(class_def);
            supported_encodings = SupportedEncodings::dummy();
        }

        // Pop the type-id off the stack, and store this class's supported encodings.
        let type_id = self.dependency_stack.pop().unwrap();
        self.add_supported_encodings_entry(class_def, type_id, supported_encodings);
    }

    fn visit_exception_start(&mut self, exception_def: &Exception) {
        let type_id = exception_def.module_scoped_identifier();
        let file_encoding = self.get_file_encoding_for(exception_def);
        let mut supported_encodings = get_encodings_supported_by(&file_encoding);

        if self.check_for_cycle(exception_def, &type_id) {
            // If the type is cyclic, return early to avoid an infinite loop.
            // `check_for_cycle` will already of reported an error message.
            return;
        }

        // Resolve the supported encodings for the data members.
        self.dependency_stack.push(type_id);
        for member in exception_def.members() {
            let member_supported_encodings = self.resolve_encodings_supported_by_type(
                &file_encoding,
                &member.data_type,
                member.tag.is_some(),
            );
            supported_encodings.intersect_with(&member_supported_encodings);
        }

        // Exception inheritance is only supported by the 1.1 encoding.
        if exception_def.base.is_some() {
            supported_encodings.disable_2();
            if file_encoding == SliceEncoding::Slice2 {
                crate::report_error(
                    "exception inheritance is only supported by the Slice 1.1 encoding".to_owned(),
                    Some(exception_def.location()),
                );
                self.print_file_encoding_note(exception_def);
                supported_encodings = SupportedEncodings::dummy();
            }
        }

        // Pop the type-id off the stack, and store this exceptions's supported encodings.
        let type_id = self.dependency_stack.pop().unwrap();
        self.add_supported_encodings_entry(exception_def, type_id, supported_encodings);
    }

    fn visit_interface_start(&mut self, interface_def: &Interface) {
        let type_id = interface_def.module_scoped_identifier();
        let file_encoding = self.get_file_encoding_for(interface_def);
        let supported_encodings = get_encodings_supported_by(&file_encoding);

        self.add_supported_encodings_entry(interface_def, type_id, supported_encodings);
    }

    fn visit_operation_start(&mut self, operation_def: &Operation) {
        fn check_operation_members(
            members: Vec<&impl Member>,
            operation_def: &Operation,
            patcher: &mut EncodingPatcher,
        ) {
            let operation_encoding = &operation_def.encoding;
            for member in members {
                let member_supported_encodings = patcher.resolve_encodings_supported_by_type(
                    operation_encoding,
                    member.data_type(),
                    member.tag().is_some(),
                );
                if !member_supported_encodings.supports(operation_encoding) {
                    let message = format!(
                        "operation '{}' contains members that are not compatible with its encoding (Slice {}).",
                        operation_def.identifier(),
                        operation_encoding
                    );
                    crate::report_error(message, Some(member.location()));
                    patcher.print_file_encoding_note(member);
                }
            }
        }

        // Ensure the operation's parameters and return type are supported by its encoding.
        check_operation_members(operation_def.parameters(), operation_def, self);
        check_operation_members(operation_def.return_members(), operation_def, self);
    }

    fn visit_enum_start(&mut self, enum_def: &Enum) {
        let type_id = enum_def.module_scoped_identifier();
        let file_encoding = self.get_file_encoding_for(enum_def);
        let mut supported_encodings = get_encodings_supported_by(&file_encoding);

        // Enums with underlying types are not supported by the Slice 1.1 encoding.
        if enum_def.underlying.is_some() {
            supported_encodings.disable_11();
            if file_encoding == SliceEncoding::Slice11 {
                crate::report_error(
                    "enums with underlying types are not supported by the Slice 1.1 encoding".to_owned(),
                    Some(enum_def.location())
                );
                self.print_file_encoding_note(enum_def);
                supported_encodings = SupportedEncodings::dummy();
            }
        }

        self.add_supported_encodings_entry(enum_def, type_id, supported_encodings);
    }

    fn visit_trait(&mut self, trait_def: &Trait) {
        let type_id = trait_def.module_scoped_identifier();
        let file_encoding = self.get_file_encoding_for(trait_def);
        let mut supported_encodings = get_encodings_supported_by(&file_encoding);

        // Traits are not supported by the Slice 1.1 encoding.
        supported_encodings.disable_11();
        if file_encoding == SliceEncoding::Slice11 {
            crate::report_error(
                "traits are not supported by the Slice 1.1 encoding".to_owned(),
                Some(trait_def.location()),
            );
            self.print_file_encoding_note(trait_def);
            supported_encodings = SupportedEncodings::dummy();
        }

        self.add_supported_encodings_entry(trait_def, type_id, supported_encodings);
    }
}

// Then we visit through everything mutably to patch in the supported encodings.
impl<'files> PtrVisitor for EncodingPatcher<'files> {
    unsafe fn visit_struct_start(&mut self, struct_ptr: &mut OwnedPtr<Struct>) {
        let struct_def = struct_ptr.borrow_mut();
        let type_id = struct_def.module_scoped_identifier();
        struct_def.supported_encodings =
            Some(self.supported_encodings.get(&type_id).unwrap().clone());
    }

    unsafe fn visit_class_start(&mut self, class_ptr: &mut OwnedPtr<Class>) {
        let class_def = class_ptr.borrow_mut();
        let type_id = class_def.module_scoped_identifier();
        class_def.supported_encodings =
            Some(self.supported_encodings.get(&type_id).unwrap().clone());
    }

    unsafe fn visit_exception_start(&mut self, exception_ptr: &mut OwnedPtr<Exception>) {
        let exception_def = exception_ptr.borrow_mut();
        let type_id = exception_def.module_scoped_identifier();
        exception_def.supported_encodings =
            Some(self.supported_encodings.get(&type_id).unwrap().clone());
    }

    unsafe fn visit_interface_start(&mut self, interface_ptr: &mut OwnedPtr<Interface>) {
        let interface_def = interface_ptr.borrow_mut();
        let type_id = interface_def.module_scoped_identifier();
        interface_def.supported_encodings =
            Some(self.supported_encodings.get(&type_id).unwrap().clone());
    }

    unsafe fn visit_enum_start(&mut self, enum_ptr: &mut OwnedPtr<Enum>) {
        let enum_def = enum_ptr.borrow_mut();
        let type_id = enum_def.module_scoped_identifier();
        enum_def.supported_encodings =
            Some(self.supported_encodings.get(&type_id).unwrap().clone());
    }

    unsafe fn visit_trait(&mut self, trait_ptr: &mut OwnedPtr<Trait>) {
        let trait_def = trait_ptr.borrow_mut();
        let type_id = trait_def.module_scoped_identifier();
        trait_def.supported_encodings =
            Some(self.supported_encodings.get(&type_id).unwrap().clone());
    }
}
