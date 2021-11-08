// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::ast::Ast;
use crate::grammar::*;
use crate::ptr_visitor::PtrVisitor;
use crate::ptr_util::{OwnedPtr, WeakPtr};
use std::collections::HashMap;

pub(super) fn patch_types(ast: &mut Ast) {
    let mut patcher = TypePatcher { lookup_table: &ast.type_lookup_table };

    for module in &mut ast.ast {
        unsafe { module.visit_ptr_with(&mut patcher); }
    }

    for type_ptr in &mut ast.anonymous_types {
        unsafe {
            // Sequences and dictionaries are the only anonymous types that need patching.
            match type_ptr.borrow_mut().concrete_type_mut() {
                TypesMut::Sequence(sequence) => {
                    patcher.resolve_definition(&mut sequence.element_type);
                }
                TypesMut::Dictionary(dictionary) => {
                    patcher.resolve_definition(&mut dictionary.key_type);
                    patcher.resolve_definition(&mut dictionary.value_type);
                }
                _ => {}
            }
        }
    }
}

struct TypePatcher<'ast> {
    lookup_table: &'ast HashMap<String, WeakPtr<dyn Type>>,
}

impl<'ast> TypePatcher<'ast> {
    fn resolve_definition(&self, type_ref: &mut TypeRef<dyn Type>) {
        // Skip if the reference has already been resolved and doesn't need patching.
        if type_ref.definition.is_initialized() {
            return;
        }

        // Lookup the type in the AST's lookup tables, and if it exists, patch it in.
        let mut lookup = Ast::lookup_type(self.lookup_table, &type_ref.type_string, &type_ref.scope);
        while let Some(definition) = lookup {
            // If the type is an alias, clone the alias' attributes to the type_ref being patched.
            // Then unwrap the alias, and continue the loop on its underlying definition.
            // TODO this entire process is not great: recursive aliases will loop forever!!!
            // This also violates rust's borrowing rules for self-referential types!
            if let Elements::TypeAlias(type_alias) = definition.borrow().concrete_element() {
                let alias_ref = &type_alias.underlying;
                type_ref.attributes.extend_from_slice(alias_ref.attributes());
                lookup = Ast::lookup_type(self.lookup_table, &alias_ref.type_string, &alias_ref.scope);
            } else {
                type_ref.definition = definition.clone();
                return;
            }
        }

        // Reaching this code means that the type lookup failed.
        // TODO report an error here!
    }

    fn resolve_typed_definition<T: Element + 'static>(&self, type_ref: &mut TypeRef<T>) {
        // Skip if the reference has already been resolved and doesn't need patching.
        if type_ref.definition.is_initialized() {
            return;
        }

        // Lookup the type in the AST's lookup tables, and if it exists, try to patch it in.
        let mut lookup = Ast::lookup_type(self.lookup_table, &type_ref.type_string, &type_ref.scope);
        while let Some(definition) = lookup {
            // If the type is an alias, clone the alias' attributes to the type_ref being patched.
            // Then unwrap the alias, and continue the loop on its underlying definition.
            // TODO this entire process is not great: recursive aliases will loop forever!!!
            // This also violates rust's borrowing rules for self-referential types!
            if let Elements::TypeAlias(type_alias) = definition.borrow().concrete_element() {
                let alias_ref = &type_alias.underlying;
                type_ref.attributes.extend_from_slice(alias_ref.attributes());
                lookup = Ast::lookup_type(self.lookup_table, &alias_ref.type_string, &alias_ref.scope);
            } else {
                // Make sure the definition's type is the correct type for the reference.
                if let Ok(converted) = definition.clone().downcast::<T>() {
                    type_ref.definition = converted;
                } else {
                    // The definition exists, but is the incorrect type.
                    // TODO throw an error here.
                }
                return;
            }
        }

        // Reaching this code means that the type lookup failed.
        // TODO report an error here!
    }
}

impl<'ast> PtrVisitor for TypePatcher<'ast> {
    unsafe fn visit_class_start(&mut self, class_ptr: &mut OwnedPtr<Class>) {
        if let Some(base_class) = &mut class_ptr.borrow_mut().base {
            self.resolve_typed_definition(base_class);
        }
    }

    unsafe fn visit_exception_start(&mut self, exception_ptr: &mut OwnedPtr<Exception>) {
        if let Some(base_exception) = &mut exception_ptr.borrow_mut().base {
            self.resolve_typed_definition(base_exception);
        }
    }

    unsafe fn visit_interface_start(&mut self, interface_ptr: &mut OwnedPtr<Interface>) {
        for base_interface in &mut interface_ptr.borrow_mut().bases {
            self.resolve_typed_definition(base_interface);
        }
    }

    unsafe fn visit_enum_start(&mut self, enum_ptr: &mut OwnedPtr<Enum>) {
        if let Some(underlying_type) = &mut enum_ptr.borrow_mut().underlying {
            self.resolve_typed_definition(underlying_type);
        }
    }

    unsafe fn visit_type_alias(&mut self, type_alias_ptr: &mut OwnedPtr<TypeAlias>) {
        self.resolve_definition(&mut type_alias_ptr.borrow_mut().underlying);
    }

    unsafe fn visit_data_member(&mut self, data_member_ptr: &mut OwnedPtr<DataMember>) {
        self.resolve_definition(&mut data_member_ptr.borrow_mut().data_type);
    }

    unsafe fn visit_parameter(&mut self, parameter_ptr: &mut OwnedPtr<Parameter>) {
        self.resolve_definition(&mut parameter_ptr.borrow_mut().data_type);
    }

    unsafe fn visit_return_member(&mut self, parameter_ptr: &mut OwnedPtr<Parameter>) {
        self.resolve_definition(&mut parameter_ptr.borrow_mut().data_type);
    }
}
