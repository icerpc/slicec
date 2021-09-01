// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::ast::{Ast, Node};
use crate::error::ErrorHandler;
use crate::grammar::*;
use crate::ref_from_node;
use crate::slice_file::SliceFile;
use crate::visitor::Visitor;
use std::collections::HashMap;

#[derive(Debug)]
pub(crate) struct TableBuilder<'a> {
    /// This stack holds the identifiers of any enclosing scopes the builder is currently visiting.
    /// Except for the 1st element, which is always the empty string. This represents the global
    /// scope, and ensures we always have a leading '::' when joining identifiers together.
    current_scope: Vec<String>,
    /// Table of all the named elements parsed in the AST. Keys are the element's fully scoped
    /// identifier, with their indexes in the AST as values. This table is used for resolving
    /// elements by their identifiers in later parsing stages.
    lookup_table: HashMap<String, usize>,
    /// Pest parses bottom-up, so we can't know enclosing scopes while parsing. Instead this
    /// builder visits elements and wherever there's an empty scope field, it computes the scope
    /// and stores a patch for it here. Each element is a tuple of the element's AST index and its
    /// scope. The patches can't be applied in place since the builder visits elements immutably.
    scope_patches: Vec<(usize, String)>,
    /// Reference to the compiler's error handler so the builder can output errors.
    error_handler: &'a mut ErrorHandler,
}

impl<'a> TableBuilder<'a> {
    /// Creates a new `TableBuilder` with empty tables, and starting at global scope ("::").
    pub(crate) fn new(error_handler: &'a mut ErrorHandler) -> Self {
        TableBuilder {
            // We add an empty string so joining the vector gives a leading '::' for global scope.
            current_scope: vec!["".to_owned()],
            lookup_table: HashMap::new(),
            scope_patches: Vec::new(),
            error_handler,
        }
    }

    pub(crate) fn generate_tables(&mut self, slice_files: &HashMap<String, SliceFile>, ast: &Ast) {
        // Immutably visit over the slice files.
        for slice_file in slice_files.values() {
            slice_file.visit_with(self, ast);
        }
    }

    pub(crate) fn into_tables(self) -> (HashMap<String, usize>, Vec<(usize, String)>) {
        (self.lookup_table, self.scope_patches)
    }

    /// Computes the fully scoped identifier for the provided element and stores an entry for it
    /// (and its index in the AST) in the lookup table.
    fn add_table_entry(&mut self, element: &impl NamedSymbol, index: usize, ast: &Ast) {
        let scoped_identifier = self.current_scope.join("::") + "::" + element.identifier();

        // Report a redefinition error if there's already an entry for this identifier.
        if let Some(original_index) = self.lookup_table.get(&scoped_identifier) {
            let original = ast.resolve_index(*original_index).as_named_symbol().unwrap();
            let redefinition = ast.resolve_index(index).as_named_symbol().unwrap();

            self.error_handler.report_error((
                format!("cannot reuse identifier `{}` in this scope", redefinition.identifier()),
                redefinition,
            ).into());
            self.error_handler.report_note((
                format!("{} `{}` was originally defined here", original.kind(), original.identifier()),
                original,
            ).into());
        } else {
            self.lookup_table.insert(scoped_identifier, index);
        }
    }

    /// Computes the scope the current element resides in, and stores that with the element's index
    /// in the AST, so ScopePatcher can patch it later. We can't patch it now since TableBuilder
    /// has to immutably visit elements.
    fn add_scope_patch(&mut self, index: usize) {
        self.scope_patches.push(
            (index, self.current_scope.join("::"))
        );
    }
}

impl<'a> Visitor for TableBuilder<'a> {
    fn visit_module_start(&mut self, module_def: &Module, index: usize, ast: &Ast) {
        self.add_table_entry(module_def, index, ast);
        self.add_scope_patch(index);
        self.current_scope.push(module_def.identifier().to_owned());
    }

    fn visit_module_end(&mut self, _: &Module, _: usize, _: &Ast) {
        self.current_scope.pop();
    }

    fn visit_struct_start(&mut self, struct_def: &Struct, index: usize, ast: &Ast) {
        self.add_table_entry(struct_def, index, ast);
        self.add_scope_patch(index);
        self.current_scope.push(struct_def.identifier().to_owned());
    }

    fn visit_struct_end(&mut self, _: &Struct, _: usize, _: &Ast) {
        self.current_scope.pop();
    }

    fn visit_class_start(&mut self, class_def: &Class, index: usize, ast: &Ast) {
        self.add_table_entry(class_def, index, ast);
        self.add_scope_patch(index);
        self.current_scope.push(class_def.identifier().to_owned());
    }

    fn visit_class_end(&mut self, _: &Class, _: usize, _: &Ast) {
        self.current_scope.pop();
    }

    fn visit_exception_start(&mut self, exception_def: &Exception, index: usize, ast: &Ast) {
        self.add_table_entry(exception_def, index, ast);
        self.add_scope_patch(index);
        self.current_scope.push(exception_def.identifier().to_owned());
    }

    fn visit_exception_end(&mut self, _: &Exception, _: usize, _: &Ast) {
        self.current_scope.pop();
    }

    fn visit_interface_start(&mut self, interface_def: &Interface, index: usize, ast: &Ast) {
        self.add_table_entry(interface_def, index, ast);
        self.add_scope_patch(index);
        self.current_scope.push(
            interface_def.identifier().to_owned()
        );
    }

    fn visit_interface_end(&mut self, _: &Interface, _: usize, _: &Ast) {
        self.current_scope.pop();
    }

    fn visit_enum_start(&mut self, enum_def: &Enum, index: usize, ast: &Ast) {
        self.add_table_entry(enum_def, index, ast);
        self.add_scope_patch(index);
        self.current_scope.push(enum_def.identifier().to_owned());
    }

    fn visit_enum_end(&mut self, _: &Enum, _: usize, _: &Ast) {
        self.current_scope.pop();
    }

    fn visit_operation_start(&mut self, operation: &Operation, index: usize, ast: &Ast) {
        self.add_table_entry(operation, index, ast);
        self.add_scope_patch(index);
        self.current_scope.push(operation.identifier().to_owned());

        // Visit the operation's return type. Return types are placed in their own scope, to keep
        // the scopes for parameters and return-types separate.
        self.current_scope.push("_return".to_owned());
        match &operation.return_type {
            ReturnType::Void(_) => {}
            ReturnType::Single(return_type, _) => {
                return_type.visit_with(self, ast);
            }
            ReturnType::Tuple(return_tuple, _) => {
                for id in return_tuple.iter() {
                    let return_element = ref_from_node!(Node::Member, ast, *id);
                    self.add_table_entry(return_element, index, ast);
                    self.add_scope_patch(index);
                }
            }
        }
        self.current_scope.pop();
    }

    fn visit_operation_end(&mut self, _: &Operation, _: usize, _: &Ast) {
        self.current_scope.pop();
    }

    fn visit_data_member(&mut self, data_member: &Member, index: usize, ast: &Ast) {
        self.add_table_entry(data_member, index, ast);
        self.add_scope_patch(index);
    }

    fn visit_parameter(&mut self, parameter: &Member, index: usize, ast: &Ast) {
        self.add_table_entry(parameter, index, ast);
        self.add_scope_patch(index);
    }

    fn visit_enumerator(&mut self, enumerator: &Enumerator, index: usize, ast: &Ast) {
        self.add_table_entry(enumerator, index, ast);
        self.add_scope_patch(index);
    }

    fn visit_type_use(&mut self, type_use: &TypeRef, ast: &Ast) {
        if let Some(definition) = type_use.definition {
            let node = ast.resolve_index(definition);
            // Only sequences and dictionaries need patching.
            // Since they are the only builtin types that reference other types.
            match node {
                Node::Sequence(index, sequence) => {
                    self.add_scope_patch(*index);
                    // Visit the sequence's element type in case it needs patching.
                    sequence.element_type.visit_with(self, ast);
                }
                Node::Dictionary(index, dictionary) => {
                    self.add_scope_patch(*index);
                    // Visit the dictionary's key and value types in case they need patching.
                    dictionary.key_type.visit_with(self, ast);
                    dictionary.value_type.visit_with(self, ast);
                }
                _ => {}
            }
        }
    }
}

#[derive(Debug)]
pub(crate) struct ScopePatcher {}

impl ScopePatcher {
    pub(crate) fn patch_scopes(scope_patches: Vec<(usize, String)>, ast: &mut Ast) {
        for (index, mut scope) in scope_patches.into_iter() {
            if scope.is_empty() {
                scope = "::".to_owned();
            }

            let node = ast.resolve_index_mut(index);
            match node {
                Node::Module(_, module_def) => {
                    module_def.scope = Some(scope);
                }
                Node::Struct(_, struct_def) => {
                    struct_def.scope = Some(scope);
                }
                Node::Class(_, class_def) => {
                    class_def.scope = Some(scope);
                }
                Node::Exception(_, exception_def) => {
                    exception_def.scope = Some(scope);
                }
                Node::Interface(_, interface_def) => {
                    interface_def.scope = Some(scope);
                }
                Node::Enum(_, enum_def) => {
                    enum_def.scope = Some(scope.clone());
                    if let Some(underlying) = &mut enum_def.underlying {
                        underlying.scope = Some(scope);
                    }
                }
                Node::Operation(_, operation) => {
                    operation.scope = Some(scope.clone());
                    if let ReturnType::Single(type_ref, _) = &mut operation.return_type {
                        type_ref.scope = Some(scope);
                    }
                }
                Node::Member(_, member) => {
                    member.scope = Some(scope.clone());
                    member.data_type.scope = Some(scope);
                }
                Node::Enumerator(_, enumerator) => {
                    enumerator.scope = Some(scope);
                }
                Node::Sequence(_, sequence) => {
                    sequence.scope = Some(scope.clone());
                    sequence.element_type.scope = Some(scope);
                }
                Node::Dictionary(_, dictionary) => {
                    dictionary.scope = Some(scope.clone());
                    dictionary.key_type.scope = Some(scope.clone());
                    dictionary.value_type.scope = Some(scope);
                }
                _ => {
                    panic!("Grammar element does not need scope patching!\n{:?}", node);
                }
            }
        }
    }
}

#[derive(Debug)]
pub(crate) struct TypePatcher<'a> {
    /// Reference to the lookup table, which holds all the named elements parsed in the AST as
    /// (fully scoped identifier, AST index) entries in a map.
    lookup_table: &'a HashMap<String, usize>,
    /// Reference to the compiler's error handler so the patcher can output errors.
    error_handler: &'a mut ErrorHandler,
}

impl<'a> TypePatcher<'a> {
    pub(crate) fn new(
        lookup_table: &'a HashMap<String, usize>,
        error_handler: &'a mut ErrorHandler,
    ) -> Self {
        TypePatcher { lookup_table, error_handler }
    }

    pub(crate) fn patch_types(&mut self, ast: &mut Ast) {
        for node in ast.iter_mut() {
            match node {
                Node::Enum(_, enum_def) => {
                    // Check if the enum has an underlying type and patch it if needed.
                    if let Some(underlying) = &mut enum_def.underlying {
                        let scope = enum_def.scope.as_ref().unwrap();
                        self.patch_type(underlying, scope);
                    }
                }
                Node::Operation(_, operation) => {
                    // We only have to handle the case of single return types here.
                    // Return tuple elements will be handled by the `Member` case instead.
                    if let ReturnType::Single(type_ref, _) = &mut operation.return_type {
                        let scope = operation.scope.as_ref().unwrap();
                        self.patch_type(type_ref, scope);
                    }
                }
                Node::Member(_, member) => {
                    let scope = member.scope.as_ref().unwrap();
                    self.patch_type(&mut member.data_type, scope);
                }
                Node::Sequence(_, sequence) => {
                    let scope = sequence.scope.as_ref().unwrap();
                    self.patch_type(&mut sequence.element_type, scope);
                }
                Node::Dictionary(_, dictionary) => {
                    let scope = dictionary.scope.as_ref().unwrap();
                    self.patch_type(&mut dictionary.key_type, scope);
                    self.patch_type(&mut dictionary.value_type, scope);
                }
                _ => {}
            }
        }
    }

    fn patch_type(&mut self, typeref: &mut TypeRef, scope: &str) {
        // Skip if the type doesn't need patching. This is the case for builtin types.
        if typeref.definition.is_some() {
            return;
        }

        // Attempt to resolve the type, and report an error if it fails.
        match self.find_type(&typeref.type_name, scope) {
            Some(index) => {
                // Check that the index does point to a type, and not a normal element.
                // TODO

                typeref.definition = Some(index);
            }
            None => {
                self.error_handler.report_error((
                    format!("failed to resolve type `{}` in scope `{}`", &typeref.type_name, scope),
                    typeref,
                ).into());
            }
        }
    }

    fn find_type(&mut self, typename: &str, scope: &str) -> Option<usize> {
        // If the typename starts with '::' it's an absolute path, and we can directly look it up.
        if typename.starts_with("::") {
            return self.lookup_table.get(typename).copied();
        }

        // Otherwise we search for the typename through each enclosing scope, from the bottom up.
        let parents: Vec<&str> = scope.split("::").collect();
        for i in (0..parents.len()).rev() {
            let test_name = parents[..i].join("::") + "::" + typename;
            if let Some(result) = self.lookup_table.get(&test_name) {
                return Some(*result);
            }
        }
        None
    }
}
