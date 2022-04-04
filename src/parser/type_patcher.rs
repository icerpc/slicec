// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::upcast_weak_as;

use crate::ast::Ast;
use crate::grammar::*;
use crate::ptr_util::{OwnedPtr, WeakPtr};
use crate::ptr_visitor::PtrVisitor;
use std::collections::HashMap;

pub(super) fn patch_types(ast: &mut Ast) {
    let mut patcher = TypePatcher {
        primitive_cache: &ast.primitive_cache,
        module_scoped_lookup_table: &ast.module_scoped_lookup_table,
    };

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
    primitive_cache: &'ast HashMap<&'static str, OwnedPtr<Primitive>>,
    module_scoped_lookup_table: &'ast HashMap<String, WeakPtr<dyn Entity>>,
}

impl<'ast> TypePatcher<'ast> {
    fn resolve_definition(&self, type_ref: &mut TypeRef<dyn Type>) {
        // Skip if the reference has already been resolved and doesn't need patching.
        if type_ref.definition.is_initialized() {
            return;
        }

        // Lookup the definition in the AST's lookup tables, and if it exists, try to patch it in.
        // Since only user-defined types need to be patched, we lookup by entity instead of by type.
        let lookup = Ast::lookup_module_scoped_entity(
            self.module_scoped_lookup_table, &type_ref.type_string, &type_ref.scope,
        );
        if let Some(definition) = lookup {
            match definition.borrow().concrete_entity() {
                Entities::Struct(_) => {
                    type_ref.definition = upcast_weak_as!(
                        definition.clone().downcast::<Struct>().ok().unwrap(), dyn Type
                    );
                    return;
                }
                Entities::Class(_) => {
                    type_ref.definition = upcast_weak_as!(
                        definition.clone().downcast::<Class>().ok().unwrap(), dyn Type
                    );
                    return;
                }
                Entities::Exception(_) => {
                    type_ref.definition = upcast_weak_as!(
                        definition.clone().downcast::<Exception>().ok().unwrap(), dyn Type
                    );
                    return;
                }
                Entities::Interface(_) => {
                    type_ref.definition = upcast_weak_as!(
                        definition.clone().downcast::<Interface>().ok().unwrap(), dyn Type
                    );
                    return;
                }
                Entities::Enum(_) => {
                    type_ref.definition = upcast_weak_as!(
                        definition.clone().downcast::<Enum>().ok().unwrap(), dyn Type
                    );
                    return;
                }
                Entities::Trait(_) => {
                    type_ref.definition = upcast_weak_as!(
                        definition.clone().downcast::<Trait>().ok().unwrap(), dyn Type
                    );
                    return;
                }
                Entities::CustomType(_) => {
                    type_ref.definition = upcast_weak_as!(
                        definition.clone().downcast::<CustomType>().ok().unwrap(), dyn Type
                    );
                    return;
                }
                Entities::TypeAlias(type_alias) => {
                    // TODO this can probably be simplified into a single loop.
                    let alias_ref = &type_alias.underlying;
                    type_ref.attributes.extend_from_slice(alias_ref.attributes());

                    if alias_ref.definition.is_initialized() {
                        type_ref.definition = alias_ref.definition.clone();
                        return;
                    }

                    let mut alias_lookup = Ast::lookup_type(
                        self.module_scoped_lookup_table,
                        self.primitive_cache,
                        &alias_ref.type_string,
                        &alias_ref.scope,
                    );

                    while let Ok(underlying) = &alias_lookup {
                        if let Ok(underlying_alias) = underlying.clone().downcast::<TypeAlias>() {
                            let underlying_ref = &underlying_alias.borrow().underlying;
                            type_ref.attributes.extend_from_slice(underlying_ref.attributes());

                            if underlying_ref.definition.is_initialized() {
                                type_ref.definition = underlying_ref.definition.clone();
                                return;
                            }

                            alias_lookup = Ast::lookup_type(
                                self.module_scoped_lookup_table,
                                self.primitive_cache,
                                &underlying_ref.type_string,
                                &underlying_ref.scope,
                            );
                        } else {
                            type_ref.definition = underlying.clone();
                            return;
                        }
                    }
                },
                _ => panic!("Encountered unpatchable type: {}", definition.borrow().kind())
            }
        }

        crate::report_error(format!(
            "No entity with the identifier '{}' could be found in this scope.",
            &type_ref.type_string,
        ), Some(type_ref.location()));
    }

    fn resolve_typed_definition<T: Element + 'static>(&self, type_ref: &mut TypeRef<T>) {
        // Skip if the reference has already been resolved and doesn't need patching.
        if type_ref.definition.is_initialized() {
            return;
        }

        // Lookup the definition in the AST's lookup tables, and if it exists, try to patch it in.
        let lookup = Ast::lookup_module_scoped_entity(
            self.module_scoped_lookup_table, &type_ref.type_string, &type_ref.scope
        );

        if let Some(definition) = lookup {
            if let Ok(converted) = definition.clone().downcast::<T>() {
                type_ref.definition = converted;
            } else {
                crate::report_error(format!(
                    "The Entity '{}' is not a valid type for this definition.",
                    &type_ref.type_string,
                ), Some(type_ref.location()));
            }
        } else {
            crate::report_error(format!(
                "No entity with the identifier '{}' could be found in this scope.",
                &type_ref.type_string,
            ), Some(type_ref.location()));
        }
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
