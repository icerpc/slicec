// Copyright (c) ZeroC, Inc.

//! TODO write a doc comment for the module.

pub mod node;

use self::node::Node;
use crate::grammar::{Element, NamedSymbol, Primitive};
use crate::utils::ptr_util::{OwnedPtr, WeakPtr};
use std::collections::HashMap;

/// The AST (Abstract Syntax Tree) is the heart of the compiler, containing all the slice elements defined and used by
/// slice files passed into the compiler.
///
/// The AST is primarily for centralizing ownership of Slice elements, but also features lookup functions for finding
/// nodes (see [`find_node_by_id`](Ast::find_node_by_id)) and their
/// elements (see [`find_symbol_by_id`](Ast::find_symbol_by_id)).
///
/// In practice, there is a single instance of the AST per compilation, which is [created](Ast::create) during
/// initialization and lives as long as the program does, making the AST effectively `'static`.
///
/// All AST's contain the [primitive](Primitive) types by default. New Slice elements are inserted into the AST as
/// they're parsed (but this order shouldn't be relied upon). Since there's only one instance per compilation, even
/// elements in different Slice files are owned by the same AST.
#[derive(Debug)]
pub struct Ast {
    /// Stores all the slice elements in this AST as a flattened vector of [nodes](Node).
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
    /// # use slicec::ast::Ast;
    /// let ast = Ast::create();
    /// assert_eq!(ast.as_slice().len(), 16); // Only the 16 primitives are defined.
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
        ];

        let lookup_table = HashMap::new();

        Ast { elements, lookup_table }
    }

    /// Returns a reference to the Ast [node](Node) that corresponds to the provided [primitive](Primitive) type.
    ///
    /// This is a low level method used for retrieving nodes from the AST directly.
    ///
    /// # Examples
    ///
    /// ```
    /// # use slicec::ast::Ast;
    /// # use slicec::grammar::*;
    /// let ast = Ast::create();
    ///
    /// // Lookup a primitive type.
    /// let int32: &dyn Element = ast.find_primitive_node(Primitive::Int32).into();
    /// assert_eq!(int32.kind(), "int32");
    /// ```
    pub fn find_primitive_node(&self, primitive: Primitive) -> &Node {
        self.elements.get(primitive as usize).expect("Missing primitive node!")
    }

    /// Returns a reference to the AST [node](Node) with the provided identifier, if one exists.
    ///
    /// If the identifier starts with '::' it is treated as globally scoped, otherwise it is treated as relatively
    /// scoped.
    ///
    /// For relative identifiers, this method first checks if the identifier is defined in the provided scope. If so, a
    /// reference is returned to it. Otherwise each enclosing scope is checked, starting from the provided scope, and
    /// working outwards through each of its parent scopes until reaching global scope.
    ///
    /// This returns the first matching AST node it can find. If another node in a more outward scope also has the
    /// specified identifier, it is shadowed, and will not be returned. Exercise care when looking up modules (which
    /// can be re-opened) or parameters and return members (which share an AST scope), since these may not be unique.
    ///
    /// Primitive types (`int32`, `string`, etc.) and anonymous types (results, sequences, and dictionaries)
    /// cannot be looked up with this method.
    ///
    /// This is a low level method used for retrieving nodes from the AST directly.
    /// Only use this if you need access to the node, or the pointer, holding a slice element.
    ///
    /// If you want a reference to the Slice construct itself, use [find_symbol_by_id](Ast::find_symbol_by_id) instead.
    ///
    /// # Returns
    ///
    /// If a node can be found with the provided identifier, this returns a reference to its [node](Node) in the AST
    /// wrapped in `Ok`. Otherwise, this returns `Err` with a string describing why the lookup failed.
    pub fn find_node_by_id<'a>(&'a self, identifier: &str, scope: &str) -> Result<&'a Node, LookupError> {
        // If the identifier isn't globally scoped, we check for it in the provided scope,
        // followed by each of its parent scopes, until finally landing at global scope.
        if !identifier.starts_with("::") {
            // Split the provided scope into an iterator of scope segments.
            let mut scopes = scope.split("::").collect::<Vec<_>>();

            // Check for the identifier with the full scope first.
            // If it doesn't exist, keep checking for it in parent scopes until all enclosing scopes have been checked.
            while !scopes.is_empty() {
                let candidate = scopes.join("::") + "::" + identifier;
                if let Ok(node) = self.lookup_node_by_id(&candidate) {
                    return Ok(node);
                }
                // Pop the last scope segment off to get to the next highest scope.
                scopes.pop();
            }

            // If the identifier wasn't defined in any of the scopes, fallback to checking for it at global scope.
        }

        // Remove any leading '::' from the identifier, since the lookup table doesn't store them.
        // TODO switch to 'trim_prefix' (https://github.com/rust-lang/rust/issues/142312) when it's stabilized.
        let stripped_identifier = identifier.strip_prefix("::").unwrap_or(identifier);
        self.lookup_node_by_id(stripped_identifier)
    }

    /// Returns a reference to a Slice symbol (user-defined element) with the provided identifier and specified type,
    /// if one exists. The identifier must be fully qualified, but should not begin with a leading '::'.
    ///
    /// Care should be taken when looking up modules (which can be re-opened) or parameters and return members
    /// (which share an AST scope), since these may not be unique.
    ///
    /// # Returns
    ///
    /// If a symbol with the specified identifier and type can be found in this `Ast`, this returns a reference to it,
    /// wrapped in `Ok`. Otherwise, this returns `Err` with a string describing why the lookup failed.
    pub fn find_symbol_by_id<'a, T: NamedSymbol + ?Sized>(&'a self, identifier: &str) -> Result<&'a T, LookupError>
    where
        &'a T: TryFrom<&'a Node, Error = LookupError>,
    {
        self.lookup_node_by_id(identifier)?.try_into()
    }

    /// Returns an immutable slice of all the [nodes](Node) contained in this AST.
    ///
    /// # Examples
    ///
    /// ```
    /// # use slicec::ast::Ast;
    /// let ast = Ast::create();
    ///
    /// // Iterate through the contents of the AST.
    /// let contents = ast.as_slice();
    /// contents.iter().for_each(|x| { /* do something */ });
    /// ```
    pub fn as_slice(&self) -> &[Node] {
        self.elements.as_slice()
    }

    /// Returns a mutable slice of all the [nodes](Node) contained in this AST.
    ///
    /// # Examples
    ///
    /// ```
    /// # use slicec::ast::Ast;
    /// let mut ast = Ast::create();
    ///
    /// // Iterate through the contents of the AST.
    /// let contents = ast.as_mut_slice();
    /// contents.iter_mut().for_each(|x| { /* do something */ });
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
        // Add an entry to this AST's lookup table for the element.
        let scoped_identifier = element.borrow().parser_scoped_identifier();
        self.lookup_table.insert(scoped_identifier, self.elements.len());

        // Add the element to this AST.
        self.add_element(element)
    }

    fn lookup_node_by_id<'a>(&'a self, identifier: &str) -> Result<&'a Node, LookupError> {
        match self.lookup_table.get(identifier) {
            Some(index) => Ok(&self.elements[*index]),
            None => Err(LookupError::DoesNotExist {
                identifier: identifier.to_owned(),
            }),
        }
    }
}

impl Default for Ast {
    fn default() -> Self {
        Self::create()
    }
}

/// The error type for lookup operations on the AST.
#[derive(Debug)]
pub enum LookupError {
    /// No AST node exists that corresponds to the provided identifier.
    DoesNotExist {
        /// The (possibly scoped) identifier that was looked up.
        identifier: String,
    },

    /// An AST node with the provided identifier exists, but the element stored in it wasn't of the specified type.
    TypeMismatch {
        /// The type that the caller was expecting to find.
        expected: String,
        /// The type that was actually stored in the AST node.
        actual: String,
        /// Whether the expected type was concrete or a trait.
        /// This is used to change the wording of the error message.
        is_concrete: bool,
    },
}

impl From<LookupError> for crate::diagnostics::Error {
    fn from(error: LookupError) -> Self {
        match error {
            LookupError::DoesNotExist { identifier } => Self::DoesNotExist { identifier },
            LookupError::TypeMismatch {
                expected,
                actual,
                is_concrete,
            } => Self::TypeMismatch {
                expected,
                actual,
                is_concrete,
            },
        }
    }
}
