// Copyright (c) ZeroC, Inc.

use crate::ast::{Ast, LookupError, Node};
use crate::compilation_result::{CompilationData, CompilationResult};
use crate::diagnostics::*;
use crate::grammar::*;
use crate::utils::ptr_util::{OwnedPtr, WeakPtr};

pub unsafe fn patch_ast(mut compilation_data: CompilationData) -> CompilationResult {
    let mut patcher = TypeRefPatcher {
        type_ref_patches: Vec::new(),
        diagnostic_reporter: &mut compilation_data.diagnostic_reporter,
    };

    // TODO why explain we split this logic so that we can for sure have an immutable AST.
    patcher.compute_patches(&compilation_data.ast);
    patcher.apply_patches(&mut compilation_data.ast);

    compilation_data.into()
}

struct TypeRefPatcher<'a> {
    type_ref_patches: Vec<PatchKind>,
    diagnostic_reporter: &'a mut DiagnosticReporter,
}

impl TypeRefPatcher<'_> {
    fn compute_patches(&mut self, ast: &Ast) {
        for node in ast.as_slice() {
            let patch = match node {
                Node::Class(class_ptr) => class_ptr
                    .borrow()
                    .base
                    .as_ref()
                    .and_then(|type_ref| self.resolve_definition(type_ref, ast))
                    .map(PatchKind::BaseClass),
                Node::Exception(exception_ptr) => exception_ptr
                    .borrow()
                    .base
                    .as_ref()
                    .and_then(|type_ref| self.resolve_definition(type_ref, ast))
                    .map(PatchKind::BaseException),
                Node::Field(field_ptr) => {
                    let type_ref = &field_ptr.borrow().data_type;
                    self.resolve_definition(type_ref, ast).map(PatchKind::FieldType)
                }
                Node::Interface(interface_ptr) => {
                    interface_ptr.borrow().bases.iter()
                        .map(|type_ref| self.resolve_definition(type_ref, ast))
                        .collect::<Option<Vec<_>>>() // None if any of the bases couldn't be resolved.
                        .map(PatchKind::BaseInterfaces)
                }
                Node::Operation(operation_ptr) => {
                    if let Throws::Specific(type_ref) = &operation_ptr.borrow().throws {
                        self.resolve_definition(type_ref, ast).map(PatchKind::ThrowsType)
                    } else {
                        None
                    }
                }
                Node::Parameter(parameter_ptr) => {
                    let type_ref = &parameter_ptr.borrow().data_type;
                    self.resolve_definition(type_ref, ast).map(PatchKind::ParameterType)
                }
                Node::Enum(enum_ptr) => enum_ptr
                    .borrow()
                    .underlying
                    .as_ref()
                    .and_then(|type_ref| self.resolve_definition(type_ref, ast))
                    .map(PatchKind::EnumUnderlyingType),
                Node::TypeAlias(type_alias_ptr) => {
                    let type_ref = &type_alias_ptr.borrow().underlying;
                    self.resolve_definition(type_ref, ast)
                        .map(PatchKind::TypeAliasUnderlyingType)
                }
                Node::Sequence(sequence_ptr) => {
                    let type_ref = &sequence_ptr.borrow().element_type;
                    self.resolve_definition(type_ref, ast).map(PatchKind::SequenceType)
                }
                Node::Dictionary(dictionary_ptr) => {
                    let dictionary_def = dictionary_ptr.borrow();
                    let key_patch = self.resolve_definition(&dictionary_def.key_type, ast);
                    let value_patch = self.resolve_definition(&dictionary_def.value_type, ast);
                    Some(PatchKind::DictionaryTypes(key_patch, value_patch))
                }
                _ => None,
            };
            self.type_ref_patches.push(patch.unwrap_or_default());
        }
    }

    unsafe fn apply_patches(self, ast: &mut Ast) {
        let elements = ast.as_mut_slice();

        // There should be 1 patch per AST node.
        debug_assert_eq!(elements.len(), self.type_ref_patches.len());

        // Simultaneously iterate through patches and AST nodes, and apply each patch to its corresponding node.
        //
        // Each match arm is broken into 2 steps, separated by a comment. First we navigate to the TypeRefs that needs
        // patching, then we patch in its definition and any attributes it might of picked up from type aliases.
        for (i, patch) in self.type_ref_patches.into_iter().enumerate() {
            match patch {
                PatchKind::BaseClass((base_class_ptr, attributes)) => {
                    let class_ptr: &mut OwnedPtr<Class> = (&mut elements[i]).try_into().unwrap();
                    let base_class_ref = class_ptr.borrow_mut().base.as_mut().unwrap();
                    base_class_ref.patch(base_class_ptr, attributes);
                }
                PatchKind::BaseException((base_exception_ptr, attributes)) => {
                    let exception_ptr: &mut OwnedPtr<Exception> = (&mut elements[i]).try_into().unwrap();
                    let base_exception_ref = exception_ptr.borrow_mut().base.as_mut().unwrap();
                    base_exception_ref.patch(base_exception_ptr, attributes);
                }
                PatchKind::BaseInterfaces(base_interface_patches) => {
                    let interface_ptr: &mut OwnedPtr<Interface> = (&mut elements[i]).try_into().unwrap();
                    // Ensure the number of patches is equal to the number of base interfaces.
                    debug_assert_eq!(interface_ptr.borrow().bases.len(), base_interface_patches.len());

                    // Iterate through and patch each base interface.
                    for (j, patch) in base_interface_patches.into_iter().enumerate() {
                        let (base_interface_ptr, attributes) = patch;
                        let base_interface_ref = &mut interface_ptr.borrow_mut().bases[j];
                        base_interface_ref.patch(base_interface_ptr, attributes);
                    }
                }
                PatchKind::FieldType((field_type_ptr, attributes)) => {
                    let field_ptr: &mut OwnedPtr<Field> = (&mut elements[i]).try_into().unwrap();
                    let field_type_ref = &mut field_ptr.borrow_mut().data_type;
                    field_type_ref.patch(field_type_ptr, attributes);
                }
                PatchKind::ParameterType((parameter_type_ptr, attributes)) => {
                    let parameter_ptr: &mut OwnedPtr<Parameter> = (&mut elements[i]).try_into().unwrap();
                    let parameter_type_ref = &mut parameter_ptr.borrow_mut().data_type;
                    parameter_type_ref.patch(parameter_type_ptr, attributes);
                }
                PatchKind::ThrowsType((exception_type_ptr, attributes)) => {
                    let operation_ptr: &mut OwnedPtr<Operation> = (&mut elements[i]).try_into().unwrap();
                    if let Throws::Specific(throws_type_ref) = &mut operation_ptr.borrow_mut().throws {
                        throws_type_ref.patch(exception_type_ptr, attributes);
                    } else {
                        unreachable!() // If a patch exists, there must of been a type_ref to patch.
                    }
                }
                PatchKind::EnumUnderlyingType((enum_underlying_type_ptr, attributes)) => {
                    let enum_ptr: &mut OwnedPtr<Enum> = (&mut elements[i]).try_into().unwrap();
                    let enum_underlying_type_ref = enum_ptr.borrow_mut().underlying.as_mut().unwrap();
                    enum_underlying_type_ref.patch(enum_underlying_type_ptr, attributes);
                }
                PatchKind::TypeAliasUnderlyingType((type_alias_underlying_type_ptr, attributes)) => {
                    let type_alias_ptr: &mut OwnedPtr<TypeAlias> = (&mut elements[i]).try_into().unwrap();
                    let type_alias_underlying_type_ref = &mut type_alias_ptr.borrow_mut().underlying;
                    type_alias_underlying_type_ref.patch(type_alias_underlying_type_ptr, attributes);
                }
                PatchKind::SequenceType((element_type_ptr, attributes)) => {
                    let sequence_ptr: &mut OwnedPtr<Sequence> = (&mut elements[i]).try_into().unwrap();
                    let element_type_ref = &mut sequence_ptr.borrow_mut().element_type;
                    element_type_ref.patch(element_type_ptr, attributes);
                }
                PatchKind::DictionaryTypes(key_patch, value_patch) => {
                    let dictionary_ptr: &mut OwnedPtr<Dictionary> = (&mut elements[i]).try_into().unwrap();
                    if let Some((key_type_ptr, key_attributes)) = key_patch {
                        dictionary_ptr.borrow_mut().key_type.patch(key_type_ptr, key_attributes);
                    }
                    if let Some((value_type_ptr, value_attributes)) = value_patch {
                        dictionary_ptr
                            .borrow_mut()
                            .value_type
                            .patch(value_type_ptr, value_attributes);
                    }
                }
                PatchKind::None => {}
            }
        }
    }

    fn resolve_definition<'a, T>(&mut self, type_ref: &TypeRef<T>, ast: &'a Ast) -> Option<Patch<T>>
    where
        T: Element + ?Sized,
        &'a Node: TryInto<WeakPtr<T>, Error = LookupError>,
    {
        // If the definition is already patched, we skip the function and return `None` immediately.
        // Otherwise we retrieve the type string and try to resolve it in the ast.
        let TypeRefDefinition::Unpatched(type_string) = &type_ref.definition else { return None; };

        // There are 3 steps to type resolution.
        // First, lookup the type as a node in the AST.
        // Second, handle the case where the type is an alias (by resolving down to its concrete underlying type).
        // Third, get the type's pointer from its node and attempt to cast it to `T` (the required Slice type).
        let lookup_result = ast
            .find_node_with_scope(type_string, type_ref.module_scope())
            .and_then(|node| {
                // We perform the deprecation check here instead of the validators since we need to check type-aliases
                // which are resolved and erased after TypeRef patching is completed.
                self.check_for_deprecated_type(type_ref, node);

                if let Node::TypeAlias(type_alias) = node {
                    self.resolve_type_alias(type_alias.borrow(), ast)
                } else {
                    try_into_patch(node, Vec::new())
                }
            });

        // If we resolved a definition for the type reference, return it, otherwise report what went wrong.
        match lookup_result {
            Ok(definition) => Some(definition),
            Err(err) => {
                let mapped_error: Error = err.into();
                mapped_error.set_span(type_ref.span()).report(self.diagnostic_reporter);
                None
            }
        }
    }

    fn check_for_deprecated_type<T: Element + ?Sized>(&mut self, type_ref: &TypeRef<T>, node: &Node) {
        // Check if the type is an entity, and if so, check if it has the `deprecated` attribute.
        // Only entities can be deprecated, so this check is sufficient.
        if let Ok(entity) = <&dyn Entity>::try_from(node) {
            if let Some(argument) = entity.get_deprecation(true) {
                // Compute the warning message. The `deprecated` attribute can have either 0 or 1 arguments, so we
                // only check the first argument. If it's present, we attach it to the warning message we emit.
                Warning::new(WarningKind::UseOfDeprecatedEntity {
                    identifier: entity.identifier().to_owned(),
                    deprecation_reason: argument.map_or_else(String::new, |arg| ": ".to_owned() + &arg),
                })
                .set_span(type_ref.span())
                .set_scope(type_ref.parser_scope())
                .add_note(
                    format!("{} was deprecated here:", entity.identifier()),
                    Some(entity.span()),
                )
                .report(self.diagnostic_reporter);
            }
        }
    }

    fn resolve_type_alias<'a, T>(&mut self, type_alias: &'a TypeAlias, ast: &'a Ast) -> Result<Patch<T>, LookupError>
    where
        T: Element + ?Sized,
        &'a Node: TryInto<WeakPtr<T>, Error = LookupError>,
    {
        // In case there's a chain of type aliases, we maintain a stack of all the ones we've seen.
        // While resolving the chain, if we see a type alias already in this vector, a cycle is present.
        let mut type_alias_chain = Vec::new();

        let mut attributes: Vec<Attribute> = Vec::new();
        let mut current_type_alias = type_alias;
        loop {
            type_alias_chain.push(current_type_alias);
            attributes.extend(
                current_type_alias
                    .attributes(false)
                    .into_iter()
                    .cloned()
                    .collect::<Vec<Attribute>>(),
            );
            let underlying_type = &current_type_alias.underlying;

            // If we hit a type alias that is already patched, we immediately return its underlying type.
            // Otherwise we retrieve the alias' type string and try to resolve it in the ast.
            let type_string = match &underlying_type.definition {
                TypeRefDefinition::Patched(ptr) => {
                    // Lookup the node that is being aliased in the AST, and convert it into a patch.
                    // TODO: when `T = dyn Type` we can skip this, and use `ptr.clone()` directly.
                    let nodes = ast.as_slice();
                    let node = nodes.iter().find(|node| ptr == &<&dyn Element>::from(*node));
                    return try_into_patch(node.unwrap(), attributes);
                }
                TypeRefDefinition::Unpatched(s) => s,
            };

            // TODO this will lead to duplicate errors, if there's a broken type alias and multiple things use it!
            let node = ast.find_node_with_scope(type_string, underlying_type.module_scope())?;
            // If the node is another type alias, push it onto the chain and continue iterating, otherwise return it.
            if let Node::TypeAlias(next_type_alias) = node {
                current_type_alias = next_type_alias.borrow();
            } else {
                return try_into_patch(node, attributes);
            }

            // Check if we've already seen the next type alias before continuing the loop; if so, it's cyclic and we
            // emit a detailed error message showing each chain in the cycle before returning with an error.
            let lookup_result = type_alias_chain
                .iter()
                .position(|&other| std::ptr::eq(other, current_type_alias));
            if let Some(i) = lookup_result {
                type_alias_chain.push(current_type_alias);
                let notes = type_alias_chain[i..]
                    .windows(2)
                    .map(|window| {
                        let identifier = window[0].identifier();
                        let identifier_original = window[1].identifier();
                        Note {
                            message: format!("type alias '{identifier}' uses type alias '{identifier_original}' here:"),
                            span: Some(window[0].underlying.span().clone()),
                        }
                    })
                    .collect::<Vec<Note>>();

                Error::new(ErrorKind::SelfReferentialTypeAliasNeedsConcreteType {
                    identifier: current_type_alias.module_scoped_identifier(),
                })
                .set_span(current_type_alias.span())
                .add_note("failed to resolve type due to a cycle in its definition", None)
                .add_notes(notes)
                .report(self.diagnostic_reporter);

                return Err(LookupError::DoesNotExist {
                    identifier: current_type_alias.module_scoped_identifier(),
                });
            }
        }
    }
}

type Patch<T> = (WeakPtr<T>, Vec<Attribute>);

#[derive(Default)]
enum PatchKind {
    #[default]
    None,
    BaseClass(Patch<Class>),
    BaseException(Patch<Exception>),
    BaseInterfaces(Vec<Patch<Interface>>),
    FieldType(Patch<dyn Type>),
    ParameterType(Patch<dyn Type>),
    ThrowsType(Patch<Exception>),
    EnumUnderlyingType(Patch<Primitive>),
    TypeAliasUnderlyingType(Patch<dyn Type>),
    SequenceType(Patch<dyn Type>),
    DictionaryTypes(Option<Patch<dyn Type>>, Option<Patch<dyn Type>>),
}

fn try_into_patch<'a, T: ?Sized>(node: &'a Node, attributes: Vec<Attribute>) -> Result<Patch<T>, LookupError>
where
    &'a Node: TryInto<WeakPtr<T>, Error = LookupError>,
{
    node.try_into().map(|ptr| (ptr, attributes))
}

impl From<LookupError> for Error {
    fn from(error: LookupError) -> Self {
        Error::new(match error {
            LookupError::DoesNotExist { identifier } => ErrorKind::DoesNotExist { identifier },
            LookupError::TypeMismatch { expected, actual, is_concrete } => {
                ErrorKind::TypeMismatch { expected, actual, is_concrete }
            }
        })
    }
}
