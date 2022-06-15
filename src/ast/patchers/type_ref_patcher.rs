// Copyright (c) ZeroC, Inc. All rights reserved.

/// TODO EVERYTHING IN HERE NEEDS COMMENTS!!!

use crate::ast::{Ast, Node};
use crate::error::ErrorReporter;
use crate::grammar::*;
use crate::ptr_util::{OwnedPtr, WeakPtr};
use std::convert::{TryFrom, TryInto};

pub unsafe fn patch_ast(ast: &mut Ast, error_reporter: &mut ErrorReporter) {
    let mut patcher = TypeRefPatcher {
        type_ref_patches: Vec::new(),
        error_reporter,
    };

    // TODO explain we split this logic so that we can for sure have an immutable AST.
    patcher.compute_patches(ast);
    patcher.apply_patches(ast);
}

struct TypeRefPatcher<'a> {
    type_ref_patches: Vec<PatchKind>,
    error_reporter: &'a mut ErrorReporter,
}

impl TypeRefPatcher<'_> {
    fn compute_patches(&mut self, ast: &Ast) {
        for node in ast.as_slice() {
            let patch = match node {
                Node::Class(class_ptr) => {
                    class_ptr.borrow().base.as_ref()
                        .and_then(|type_ref| self.resolve_definition(type_ref, ast))
                        .map(PatchKind::BaseClass)
                }
                Node::Exception(exception_ptr) => {
                    exception_ptr.borrow().base.as_ref()
                        .and_then(|type_ref| self.resolve_definition(type_ref, ast))
                        .map(PatchKind::BaseException)
                }
                Node::DataMember(data_member_ptr) => {
                    let type_ref = &data_member_ptr.borrow().data_type;
                    self.resolve_definition(type_ref, ast)
                        .map(PatchKind::DataMemberType)
                }
                Node::Interface(interface_ptr) => {
                    interface_ptr.borrow().bases.iter()
                        .map(|type_ref| self.resolve_definition(type_ref, ast))
                        .collect::<Option<Vec<_>>>() // None if any of the bases couldn't be resolved.
                        .map(PatchKind::BaseInterfaces)
                }
                Node::Parameter(parameter_ptr) => {
                    let type_ref = &parameter_ptr.borrow().data_type;
                    self.resolve_definition(type_ref, ast)
                        .map(PatchKind::ParameterType)
                }
                Node::Enum(enum_ptr) => {
                    enum_ptr.borrow().underlying.as_ref()
                        .and_then(|type_ref| self.resolve_definition(type_ref, ast))
                        .map(PatchKind::EnumUnderlyingType)
                }
                Node::TypeAlias(type_alias_ptr) => {
                    let type_ref = &type_alias_ptr.borrow().underlying;
                    self.resolve_definition(type_ref, ast)
                        .map(PatchKind::TypeAliasUnderlyingType)
                }
                Node::Sequence(sequence_ptr) => {
                    let type_ref = &sequence_ptr.borrow().element_type;
                    self.resolve_definition(type_ref, ast)
                        .map(PatchKind::SequenceType)
                }
                Node::Dictionary(dictionary_ptr) => {
                    let dictionary_def = dictionary_ptr.borrow();
                    let key_patch = self.resolve_definition(&dictionary_def.key_type, ast);
                    let value_patch = self.resolve_definition(&dictionary_def.value_type, ast);
                    match (key_patch, value_patch) {
                        (Some(key), Some(value)) => Some(PatchKind::DictionaryTypes(key, value)),
                        _ => None,
                    }
                }
                _ => None,
            };
            self.type_ref_patches.push(patch.unwrap_or_default());
        }
    }

    unsafe fn apply_patches(self, ast: &mut Ast) {
        let elements = ast.as_mut_slice();

        // There should 1 patch per AST node.
        debug_assert_eq!(elements.len(), self.type_ref_patches.len());

        // Simultaneously iterate through patches and AST nodes, and apply each patch to its corresponding node.
        //
        // Each match arm is broken into 2 steps, separated by a comment. First we navigate to the TypeRefs that needs
        // patching, then we patch in it's definition and any attributes it might of picked up from type aliases.
        for (i, patch) in self.type_ref_patches.into_iter().enumerate() {
            match patch {
                PatchKind::BaseClass((base_class_ptr, attributes)) => {
                    let class_ptr: &mut OwnedPtr<Class> = (&mut elements[i]).try_into().unwrap();
                    let base_class_ref = class_ptr.borrow_mut().base.as_mut().unwrap();
                    // Patch in the definition and any attributes picked up from type-aliases.
                    base_class_ref.definition = base_class_ptr;
                    base_class_ref.attributes.extend(attributes);
                }
                PatchKind::BaseException((base_exception_ptr, attributes)) => {
                    let exception_ptr: &mut OwnedPtr<Exception> = (&mut elements[i]).try_into().unwrap();
                    let base_exception_ref = exception_ptr.borrow_mut().base.as_mut().unwrap();
                    // Patch in the definition and any attributes picked up from type-aliases.
                    base_exception_ref.definition = base_exception_ptr;
                    base_exception_ref.attributes.extend(attributes);
                }
                PatchKind::BaseInterfaces(base_interface_patches) => {
                    let interface_ptr: &mut OwnedPtr<Interface> = (&mut elements[i]).try_into().unwrap();
                    // Ensure the number of patches is equal to the number of base interfaces.
                    debug_assert_eq!(interface_ptr.borrow().bases.len(), base_interface_patches.len());

                    // Iterate through and patch each base interface.
                    for (j, patch) in base_interface_patches.into_iter().enumerate() {
                        let (base_interface_ptr, attributes) = patch;
                        let base_interface_ref = &mut interface_ptr.borrow_mut().bases[j];
                        // Patch in the definition and any attributes picked up from type-aliases.
                        base_interface_ref.attributes.extend(attributes);
                        base_interface_ref.definition = base_interface_ptr;
                    }
                }
                PatchKind::DataMemberType((data_member_type_ptr, attributes)) => {
                    let data_member_ptr: &mut OwnedPtr<DataMember> = (&mut elements[i]).try_into().unwrap();
                    let data_member_type_ref = &mut data_member_ptr.borrow_mut().data_type;
                    // Patch in the definition and any attributes picked up from type-aliases.
                    data_member_type_ref.definition = data_member_type_ptr;
                    data_member_type_ref.attributes.extend(attributes);
                }
                PatchKind::ParameterType((parameter_type_ptr, attributes)) => {
                    let parameter_ptr: &mut OwnedPtr<Parameter> = (&mut elements[i]).try_into().unwrap();
                    let parameter_type_ref = &mut parameter_ptr.borrow_mut().data_type;
                    // Patch in the definition and any attributes picked up from type-aliases.
                    parameter_type_ref.definition = parameter_type_ptr;
                    parameter_type_ref.attributes.extend(attributes);
                }
                PatchKind::EnumUnderlyingType((enum_underlying_type_ptr, attributes)) => {
                    let enum_ptr: &mut OwnedPtr<Enum> = (&mut elements[i]).try_into().unwrap();
                    let enum_underlying_type_ref = enum_ptr.borrow_mut().underlying.as_mut().unwrap();
                    // Patch in the definition and any attributes picked up from type-aliases.
                    enum_underlying_type_ref.definition = enum_underlying_type_ptr;
                    enum_underlying_type_ref.attributes.extend(attributes);
                }
                PatchKind::TypeAliasUnderlyingType((type_alias_underlying_type_ptr, attributes)) => {
                    let type_alias_ptr: &mut OwnedPtr<TypeAlias> = (&mut elements[i]).try_into().unwrap();
                    let type_alias_underlying_type_ref = &mut type_alias_ptr.borrow_mut().underlying;
                    // Patch in the definition and any attributes picked up from type-aliases.
                    type_alias_underlying_type_ref.definition = type_alias_underlying_type_ptr;
                    type_alias_underlying_type_ref.attributes.extend(attributes);
                }
                PatchKind::SequenceType((element_type_ptr, attributes)) => {
                    let sequence_ptr: &mut OwnedPtr<Sequence> = (&mut elements[i]).try_into().unwrap();
                    let element_type_ref = &mut sequence_ptr.borrow_mut().element_type;
                    // Patch in the definition and any attributes picked up from type-aliases.
                    element_type_ref.definition = element_type_ptr;
                    element_type_ref.attributes.extend(attributes);
                }
                PatchKind::DictionaryTypes((key_type_ptr, key_attributes), (value_type_ptr, value_attributes)) => {
                    let dictionary_ptr: &mut OwnedPtr<Dictionary> = (&mut elements[i]).try_into().unwrap();
                    // Patch in the definition and any attributes picked up from type-aliases.
                    dictionary_ptr.borrow_mut().key_type.definition = key_type_ptr;
                    dictionary_ptr.borrow_mut().key_type.attributes.extend(key_attributes);
                    dictionary_ptr.borrow_mut().value_type.definition = value_type_ptr;
                    dictionary_ptr.borrow_mut().value_type.attributes.extend(value_attributes);
                }
                PatchKind::None => {}
            }
        }
    }

    fn resolve_definition<'a, T>(
        &mut self,
        type_ref: &TypeRef<T>,
        ast: &'a Ast,
    ) -> Option<Patch<T>>
    where
        T: Element + ?Sized,
        WeakPtr<T>: TryFrom<&'a Node, Error = String>,
    {
        // If the definition has already been patched return `None` immediately.
        if type_ref.definition.is_initialized() {
            return None;
        }

        // There are 3 steps to type resolution.
        // First, lookup the type as a node in the AST.
        // Second, Handle the case where the type is an alias (by resolving down to its concrete underlying type).
        // Third, get the type's pointer from its node and attempt to cast it to `T` (the required Slice type).
        let lookup_result: Result<Patch<T>, String> = ast
            .find_node_with_scope(&type_ref.type_string, type_ref.module_scope())
            .and_then(|node| {
                if let Node::TypeAlias(type_alias) = node {
                    self.resolve_type_alias(type_alias.borrow(), ast)
                } else {
                    Ok((node, Vec::new()))
                }
            })
            .and_then(|(node, attributes)| {
                node.try_into().map(|ptr| (ptr, attributes))
            });

        // If we resolved a definition for the type reference, return it, otherwise report what went wrong.
        match lookup_result {
            Ok(definition) => Some(definition),
            Err(message) => {
                self.error_reporter.report_error(message, Some(type_ref.location()));
                None
            }
        }
    }

    fn resolve_type_alias<'a>(
        &mut self,
        type_alias: &'a TypeAlias,
        ast: &'a Ast,
    ) -> Result<(&'a Node, Vec<Attribute>), String> {
        // In case there's a chain of type aliases, we maintain a stack of all the ones we've seen.
        // While resolving the chain, if we see a type alias already in this vector, an illegal cycle is present.
        let mut type_alias_chain = Vec::new();

        let mut attributes = Vec::new();
        let mut current_type_alias = type_alias;
        loop {
            type_alias_chain.push(current_type_alias);
            attributes.extend(current_type_alias.attributes().clone());
            let underlying_type = &current_type_alias.underlying;

            // TODO this will lead to duplicate errors, if there's a broken type alias and multiple things use it!
            let node = ast.find_node_with_scope(&underlying_type.type_string, underlying_type.module_scope())?;
            // If the node is another type alias, push it onto the chain and continue iterating, otherwise return it.
            if let Node::TypeAlias(next_type_alias) = node {
                current_type_alias = next_type_alias.borrow();
            } else {
                return Ok((node, attributes));
            }

            // Check if we've already seen the next type alias before continuing the loop; if so, it's cyclic and we
            // emit a detailed error message showing each chain in the cycle before returning with an error.
            let lookup_result = type_alias_chain.iter().position(|&other| std::ptr::eq(other, current_type_alias));
            if let Some(i) = lookup_result {
                type_alias_chain.push(current_type_alias);

                let message = format!(
                    "self-referential type alias '{}' has no concrete type.",
                    current_type_alias.module_scoped_identifier()
                );
                self.error_reporter.report_error(message, Some(current_type_alias.location()));

                for window in type_alias_chain[i..].windows(2) {
                    let message = format!(
                        "type alias '{}' uses type alias '{}' here:",
                        window[0].identifier(),
                        window[1].identifier(),
                    );
                    self.error_reporter.report_note(message, Some(window[0].underlying.location()));
                }

                return Err("Failed to resolve type due to a cycle in its definition".to_owned());
            }
        }
    }
}

type Patch<T> = (WeakPtr<T>, Vec<Attribute>);

enum PatchKind {
    None,
    BaseClass(Patch<Class>),
    BaseException(Patch<Exception>),
    BaseInterfaces(Vec<Patch<Interface>>),
    DataMemberType(Patch<dyn Type>),
    ParameterType(Patch<dyn Type>),
    EnumUnderlyingType(Patch<Primitive>),
    TypeAliasUnderlyingType(Patch<dyn Type>),
    SequenceType(Patch<dyn Type>),
    DictionaryTypes(Patch<dyn Type>, Patch<dyn Type>),
}

impl Default for PatchKind {
    fn default() -> Self { Self::None }
}
