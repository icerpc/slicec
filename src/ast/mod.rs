// Copyright (c) ZeroC, Inc. All rights reserved.

//! TODO write a doc comment for the module.

pub mod node;
mod patchers;

use self::node::Node;
use crate::grammar::{Element, NamedSymbol, Primitive};
use crate::parse_result::{ParsedData, ParserResult};
use crate::utils::ptr_util::{OwnedPtr, WeakPtr};
use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};

/// Since Slice definitions can be split across multiple files, and defined in any order, it is impossible for some
/// things to be determined during parsing (as it's a sequential process).
///
/// So, after parsing is complete, we modify the AST in place, 'patching' in the information that can only now be
/// computed, in the following order:
/// 1. References to other Slice types are verified and resolved.
/// 2. Compute and store the Slice encodings that each element can be used with.
///
/// This function fails fast, so if any phase of patching fails, we skip any remaining phases.
pub(crate) unsafe fn patch_ast(mut parsed_data: ParsedData) -> ParserResult {
    patchers::parent_patcher::patch_ast(&mut parsed_data.ast); // TODO remove this when we switch to LALRpop
    parsed_data = patchers::type_ref_patcher::patch_ast(parsed_data)?;
    parsed_data = patchers::encoding_patcher::patch_ast(parsed_data)?;

    parsed_data.into()
}

/// The AST (Abstract Syntax Tree) is the heart of the compiler, containing all the slice elements defined and used by
/// slice files passed into the compiler.
///
/// The AST is primarily for centralizing ownership of Slice elements, but also features lookup functions for finding
/// nodes (see [`find_node`](Ast::find_node) and [`find_node_with_scope`](Ast::find_node_with_scope)) and their
/// elements (see [`find_element`](Ast::find_element) and [`find_element_with_scope`](Ast::find_element_with_scope)).
///
/// In practice, there is a single instance of the AST per compilation, which is [created](Ast::create) during
/// initialization and lives as long as the program does, making the AST effectively `'static`.
///
/// All AST's contain the [primitive](Primitive) types by default. New Slice elements are inserted into the AST as
/// they're parsed (but this order shouldn't be relied upon). Since there's only one instance per compilation, even
/// elements in different Slice files are owned by the same instance of the AST.
#[derive(Debug)]
pub struct Ast {
    /// Stores all the slice elements in this AST as a flattened vector of [`Node`]s.
    ///
    /// Elements are stored in the order they're parsed, but this shouldn't be relied upon.
    /// Only the order of the primitive types is guaranteed by the AST (see [`create`](Ast::create)).
    elements: Vec<Node>,

    /// A hash-based lookup table with entries for every Slice element stored in this AST that implements the
    /// [`NamedSymbol`] trait (meaning it has an identifier).
    ///
    /// Each element's fully scoped identifier (without a leading '::') is used for its key, and the value stored is
    /// the element's index in this AST (specifically in the [`elements`](Ast::elements) vector).
    lookup_table: HashMap<String, usize>,
}

impl Ast {
    /// Creates an Ast that contains only the [primitive](Primitive) types.
    ///
    /// # Examples
    /// ```
    /// # use slice::ast::Ast;
    /// let ast = Ast::create();
    /// assert_eq!(ast.as_slice().len(), 17); // Only the 17 primitives are defined.
    /// ```
    pub fn create() -> Ast {
        // Primitive types are built in to the compiler. Since they aren't defined in Slice, we 'define' them here,
        // when the AST is created, to ensure they're always available.

        let elements = vec![
            Node::Primitive(OwnedPtr::new(Primitive::Bool)),
            Node::Primitive(OwnedPtr::new(Primitive::Int8)),
            Node::Primitive(OwnedPtr::new(Primitive::UInt8)),
            Node::Primitive(OwnedPtr::new(Primitive::Int16)),
            Node::Primitive(OwnedPtr::new(Primitive::UInt16)),
            Node::Primitive(OwnedPtr::new(Primitive::Int32)),
            Node::Primitive(OwnedPtr::new(Primitive::UInt32)),
            Node::Primitive(OwnedPtr::new(Primitive::VarInt32)),
            Node::Primitive(OwnedPtr::new(Primitive::VarUInt32)),
            Node::Primitive(OwnedPtr::new(Primitive::Int64)),
            Node::Primitive(OwnedPtr::new(Primitive::UInt64)),
            Node::Primitive(OwnedPtr::new(Primitive::VarInt62)),
            Node::Primitive(OwnedPtr::new(Primitive::VarUInt62)),
            Node::Primitive(OwnedPtr::new(Primitive::Float32)),
            Node::Primitive(OwnedPtr::new(Primitive::Float64)),
            Node::Primitive(OwnedPtr::new(Primitive::String)),
            Node::Primitive(OwnedPtr::new(Primitive::AnyClass)),
        ];

        let lookup_table = HashMap::from([
            ("bool".to_owned(), 0),
            ("int8".to_owned(), 1),
            ("uint8".to_owned(), 2),
            ("int16".to_owned(), 3),
            ("uint16".to_owned(), 4),
            ("int32".to_owned(), 5),
            ("uint32".to_owned(), 6),
            ("varint32".to_owned(), 7),
            ("varuint32".to_owned(), 8),
            ("int64".to_owned(), 9),
            ("uint64".to_owned(), 10),
            ("varint62".to_owned(), 11),
            ("varuint62".to_owned(), 12),
            ("float32".to_owned(), 13),
            ("float64".to_owned(), 14),
            ("string".to_owned(), 15),
            ("AnyClass".to_owned(), 16),
        ]);

        Ast { elements, lookup_table }
    }

    /// Returns a reference to the AST [node](Node) with the provided identifier, if one exists.
    /// The identifier must be globally scoped, since this method performs no scope resolution.
    ///
    /// Anonymous types (those without identifiers) cannot be looked up. This only includes sequences and dictionaries.
    /// Primitive types can be looked up by their Slice keywords. While it is possible to look up modules, it is
    /// unreliable and shouldn't be done, since module identifiers aren't necessarily unique.
    ///
    /// This is a low level method used for retrieving nodes from the AST directly.
    /// Only use this if you need access to the node, or the pointer, holding a slice element.
    /// If you just need the Slice element itself, use [`find_element`](Ast::find_element) instead.
    ///
    /// # Returns
    ///
    /// If a [node](Node) can be found with the provided identifier, this returns a reference to its [node](Node) in
    /// the AST, wrapped in `Ok`. Otherwise, this returns `Err` with a string describing why the lookup failed.
    ///
    /// # Examples
    ///
    /// ```
    /// # use slice::ast::Ast;
    /// # use slice::grammar::*;
    /// let ast = Ast::create();
    ///
    /// // Lookup a primitive type.
    /// let int32_node = ast.find_node("int32");
    /// assert!(int32_node.is_ok());
    ///
    /// // TODO add more examples once parsing is easier.
    ///
    /// // If an element doesn't exist with the specified identifier, `Err` is returned.
    /// let fake_node = ast.find_node("foo::bar");
    /// assert!(fake_node.is_err());
    /// ```
    pub fn find_node<'a>(&'a self, identifier: &str) -> Result<&'a Node, String> {
        self.lookup_table
            .get(identifier)
            .map(|i| &self.elements[*i])
            .ok_or_else(|| format!("no element with identifier `{identifier}` exists"))
    }

    /// Returns a reference to the AST [node](Node) with the provided identifier, if one exists.
    ///
    /// If the identifier begins with '::' it is treated as globally scoped, and this function just forwards to
    /// [`find_node`](Ast::find_node). Otherwise the identifier is treated as being relatively scoped.
    ///
    /// For relative identifiers, this method first checks if the identifier is defined in the provided scope. If so, a
    /// reference is returned to it. Otherwise each enclosing scope is checked, starting from the provided scope, and
    /// working outwards through each of its parent scopes until reaching global scope.
    ///
    /// This returns the first matching AST node it can find. If another node in a more outward scope also has the
    /// specified identifier, it is shadowed, and will not be returned.
    ///
    /// Anonymous types (those without identifiers) cannot be looked up. This only includes sequences and dictionaries.
    /// Primitive types can be looked up by their Slice keywords. While it is possible to look up modules, it is
    /// unreliable and shouldn't be done, since module identifiers aren't necessarily unique.
    ///
    /// This is a low level method used for retrieving nodes from the AST directly.
    /// Only use this if you need access to the node, or the pointer, holding a slice element.
    /// If you just need the Slice element itself, use [find_element_with_scope](Ast::find_element_with_scope) instead.
    ///
    /// # Returns
    ///
    /// If a node can be found with the provided identifier, this returns a reference to its [node](Node) in the AST
    /// wrapped in `Ok`. Otherwise, this returns `Err` with a string describing why the lookup failed.
    ///
    /// # Examples
    ///
    /// ```
    /// # use slice::ast::Ast;
    /// # use slice::grammar::*;
    /// let ast = Ast::create();
    ///
    /// // TODO add more examples once parsing is easier.
    ///
    /// // If an element doesn't exist with the specified identifier, `Err` is returned.
    /// let fake_node = ast.find_node_with_scope("hello", "foo::bar");
    /// assert!(fake_node.is_err());
    /// ```
    pub fn find_node_with_scope<'a>(&'a self, identifier: &str, scope: &str) -> Result<&'a Node, String> {
        // If the identifier is globally scoped (starts with '::'), find the node without scoping.
        if let Some(unprefixed_identifier) = identifier.strip_prefix("::") {
            return self.find_node(unprefixed_identifier);
        }

        // Split the provided scope into an iterator of scope segments.
        let mut scopes = scope.split("::").collect::<Vec<_>>();

        // Check for the identifier with the full scope first.
        // If it doesn't exist, keep checking for it in parent scopes until all enclosing scopes have been checked.
        while !scopes.is_empty() {
            let candidate = scopes.join("::") + "::" + identifier;
            if let Some(i) = self.lookup_table.get(&candidate) {
                return Ok(&self.elements[*i]);
            }
            // Pop the last scope segment off to get to the next highest scope.
            scopes.pop();
        }

        // If the identifier wasn't defined in any of the scopes, check for it at global scope.
        self.find_node(identifier)
            .map_err(|_| format!("no element with identifier `{identifier}` exists in the scope `{scope}`"))
    }

    /// Returns a reference to a Slice element with the provided identifier and specified type, if one exists.
    /// The identifier must be globally scoped, since this method performs no scope resolution.
    ///
    /// Anonymous types (those without identifiers) cannot be looked up. This only includes sequences and dictionaries.
    /// Primitive types can be looked up by their Slice keywords. While it is possible to look up modules, it is
    /// unreliable and shouldn't be done, since module identifiers aren't necessarily unique.
    ///
    /// # Returns
    ///
    /// If a Slice element of the specified type can be found with the provided identifier, this returns a reference to
    /// it, wrapped in `Ok`. Otherwise, this returns `Err` with a string describing why the lookup failed.
    ///
    /// # Examples
    ///
    /// ```
    /// # use slice::ast::Ast;
    /// # use slice::grammar::*;
    /// let ast = Ast::create();
    ///
    /// // Look up a primitive type.
    /// let int32_def = ast.find_element::<Primitive>("int32");
    /// assert!(int32_def.is_ok());
    /// assert_eq!(int32_def.unwrap().kind(), "int32");
    ///
    /// // Look up something implementing `Type`.
    /// let string_def = ast.find_element::<dyn Type>("string");
    /// assert!(string_def.is_ok());
    ///
    /// // TODO add more examples once parsing is easier.
    ///
    /// // If an element doesn't exist with the specified identifier, `Err` is returned.
    /// let fake_element = ast.find_element::<dyn Entity>("foo::bar");
    /// assert!(fake_element.is_err());
    ///
    /// // If an element exists but has the wrong type, `Err` is also returned.
    /// let wrong_type = ast.find_element::<Exception>("bool");
    /// assert!(fake_element.is_err());
    /// ```
    pub fn find_element<'a, T: Element + ?Sized>(&'a self, identifier: &str) -> Result<&'a T, String>
    where
        &'a T: TryFrom<&'a Node, Error = String>,
    {
        self.find_node(identifier).and_then(|x| x.try_into())
    }

    /// Returns a reference to a Slice element with the provided identifier and specified type, if one exists.
    ///
    /// If the identifier begins with '::' it is treated as globally scoped, and this function just forwards to
    /// [`find_element`](Ast::find_element). Otherwise the identifier is treated as being relatively scoped.
    ///
    /// For relative identifiers, this method first checks if the identifier is defined in the provided scope. If so, a
    /// reference is returned to it. Otherwise each enclosing scope is checked, starting from the provided scope, and
    /// working outwards through each of its parent scopes until reaching global scope.
    ///
    /// This returns the first matching Slice element it can find. If another element in a more outward scope also has
    /// the specified identifier, it is shadowed, and will not be returned.
    ///
    /// Anonymous types (those without identifiers) cannot be looked up. This only includes sequences and dictionaries.
    /// Primitive types can be looked up by their Slice keywords. While it is possible to look up modules, it is
    /// unreliable and shouldn't be done, since module identifiers aren't necessarily unique.
    ///
    /// # Returns
    ///
    /// If a Slice element of the specified type can be found with the provided identifier, this returns a reference to
    /// it, wrapped in `Ok`. Otherwise, this returns `Err` with a string describing why the lookup failed.
    ///
    /// # Examples
    ///
    /// ```
    /// # use slice::ast::Ast;
    /// # use slice::grammar::*;
    /// let ast = Ast::create();
    ///
    /// // TODO add more examples once parsing is easier.
    ///
    /// // If an element doesn't exist with the specified identifier, `Err` is returned.
    /// let fake_element = ast.find_element_with_scope::<dyn Entity>("hello", "foo::bar");
    /// assert!(fake_element.is_err());
    /// ```
    pub fn find_element_with_scope<'a, T: Element + ?Sized>(
        &'a self,
        identifier: &str,
        scope: &str,
    ) -> Result<&'a T, String>
    where
        &'a T: TryFrom<&'a Node, Error = String>,
    {
        self.find_node_with_scope(identifier, scope).and_then(|x| x.try_into())
    }

    /// Returns an immutable slice of all the [nodes](Node) contained in this AST.
    ///
    /// # Examples
    ///
    /// ```
    /// # use slice::ast::Ast;
    /// let ast = Ast::create();
    ///
    /// // Iterate through the contents of the AST.
    /// let contents = ast.as_slice();
    /// contents.iter().for_each(|x| { /* do something */ });
    /// ```
    pub fn as_slice(&self) -> &[Node] {
        self.elements.as_slice()
    }

    /// Returns a mutable slice of all the [node](Node) contained in this AST.
    ///
    /// # Examples
    ///
    /// ```
    /// # use slice::ast::Ast;
    /// let mut ast = Ast::create();
    ///
    /// // Iterate through the contents of the AST.
    /// let contents = ast.as_mut_slice();
    /// contents.iter_mut().for_each(|x| { /* do something */ } );
    /// ```
    pub fn as_mut_slice(&mut self) -> &mut [Node] {
        self.elements.as_mut_slice()
    }

    /// Moves a Slice element into this AST, and returns a [WeakPtr] to it.
    pub(crate) fn add_element<T: Element>(&mut self, element: OwnedPtr<T>) -> WeakPtr<T>
    where
        OwnedPtr<T>: Into<Node>,
    {
        let weak_ptr = element.downgrade();
        // Convert the element into a [Node] and add it to this AST.
        self.elements.push(element.into());
        weak_ptr
    }

    /// Moves a Slice element into this AST, and returns a [WeakPtr] to it, after adding an entry for the element into
    /// this AST's [lookup table](Ast::lookup_table), allowing it to be retrieved by identifier.
    pub(crate) fn add_named_element<T: NamedSymbol>(&mut self, element: OwnedPtr<T>) -> WeakPtr<T>
    where
        OwnedPtr<T>: Into<Node>,
    {
        // Add an entry for the element to this AST's lookup table.
        let scoped_identifier = element.borrow().parser_scoped_identifier();
        self.lookup_table.insert(scoped_identifier, self.elements.len());

        // Add the element to this AST.
        self.add_element(element)
    }
}
