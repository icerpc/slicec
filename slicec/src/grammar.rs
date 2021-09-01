// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::ast::{Ast, Node};
use crate::ref_from_node;
use crate::slice_file::Location;

/// The lowest base trait in the compiler, which all symbols and types implement.
pub trait Element {
    fn kind(&self) -> &'static str;
}

macro_rules! implement_element_for {
    ($a:ty, $b:literal) => {
        impl Element for $a {
            fn kind(&self) -> &'static str {
                $b
            }
        }
    };
}

implement_element_for!(Module, "module");
implement_element_for!(Struct, "struct");
implement_element_for!(Class, "class");
implement_element_for!(Exception, "exception");
implement_element_for!(Interface, "interface");
implement_element_for!(Enum, "enum");
implement_element_for!(ReturnType, "return type");
implement_element_for!(Operation, "operation");
// Member has its own custom implementation of Element which depends on its member type.
implement_element_for!(Enumerator, "enumerator");
implement_element_for!(Identifier, "identifier");
implement_element_for!(TypeRef, "type ref");
implement_element_for!(Sequence, "sequence");
implement_element_for!(Dictionary, "dictionary");
// Primitive has its own custom implementation of Element which returns the primitive's type name.
implement_element_for!(Attribute, "attribute");
implement_element_for!(DocComment, "comment");

/// Symbols represent elements of the actual source code written in the slice file.
pub trait Symbol: Element {
    fn location(&self) -> &Location;
}

macro_rules! implement_symbol_for {
    ($a:ty) => {
        impl Symbol for $a {
            fn location(&self) -> &Location {
                &self.location
            }
        }
    };
}

implement_symbol_for!(Module);
implement_symbol_for!(Struct);
implement_symbol_for!(Class);
implement_symbol_for!(Exception);
implement_symbol_for!(Interface);
implement_symbol_for!(Enum);
// ReturnType has its own custom implementation of Symbol, since it's an enum instead of a struct.
implement_symbol_for!(Operation);
implement_symbol_for!(Member);
implement_symbol_for!(Enumerator);
implement_symbol_for!(Identifier);
implement_symbol_for!(TypeRef);
implement_symbol_for!(Attribute);
implement_symbol_for!(DocComment);

/// Scoped symbols are symbols that are sensative to their enclosing scopes.
/// These also support having attributes placed on them, and provide methods for handling them.
pub trait ScopedSymbol: Symbol {
    fn attributes(&self) -> &Vec<Attribute>;
    fn find_attribute(&self, directive: &str) -> Option<&Vec<String>>;
    fn has_attribute(&self, directive: &str) -> bool;
    // TODOAUSTIN re-implement to discard parser scopes to make code generation easier.
    // EX: datamembers shouldn't have their struct as a scope in C#. That's purely for parsing.
    fn scope(&self) -> &str;
}

macro_rules! implement_scoped_symbol_for {
    ($a:ty) => {
        impl ScopedSymbol for $a {
            fn attributes(&self) -> &Vec<Attribute> {
                &self.attributes
            }

            /// Checks if the symbol has the specified attribute, and if so, returns it's
            /// arguments as a string vector. If it doesn't, it returns 'None'.
            fn find_attribute(&self, directive: &str) -> Option<&Vec<String>> {
                for m in &self.attributes {
                    if m.qualified_directive == directive {
                        return Some(&m.arguments);
                    }
                }
                return None;
            }

            /// Returns true if the symbol has the specified attribute on it,
            /// and false otherwise.
            fn has_attribute(&self, directive: &str) -> bool {
                self.find_attribute(directive).is_some()
            }

            fn scope(&self) -> &str {
                self.scope.as_ref().unwrap()
            }
        }
    };
}

implement_scoped_symbol_for!(Module);
implement_scoped_symbol_for!(Struct);
implement_scoped_symbol_for!(Class);
implement_scoped_symbol_for!(Exception);
implement_scoped_symbol_for!(Interface);
implement_scoped_symbol_for!(Enum);
implement_scoped_symbol_for!(Operation);
implement_scoped_symbol_for!(Member);
implement_scoped_symbol_for!(Enumerator);
implement_scoped_symbol_for!(TypeRef);

/// NamedSymbols are scoped symbols that have an identifier attached to them.
pub trait NamedSymbol: ScopedSymbol {
    fn identifier(&self) -> &str;
    fn comment(&self) -> Option<&DocComment>;
}

macro_rules! implement_named_symbol_for {
    ($a:ty) => {
        impl NamedSymbol for $a {
            fn identifier(&self) -> &str {
                &self.identifier.value
            }

            fn comment(&self) -> Option<&DocComment> {
                self.comment.as_ref()
            }
        }
    };
}

implement_named_symbol_for!(Module);
implement_named_symbol_for!(Struct);
implement_named_symbol_for!(Class);
implement_named_symbol_for!(Exception);
implement_named_symbol_for!(Interface);
implement_named_symbol_for!(Enum);
implement_named_symbol_for!(Operation);
implement_named_symbol_for!(Member);
implement_named_symbol_for!(Enumerator);

/// Base trait that all elements representing types implement.
pub trait Type {
    fn is_fixed_size(&self, ast: &Ast) -> bool;
    fn min_wire_size(&self, ast: &Ast) -> u32;
}

#[derive(Clone, Debug)]
pub struct Module {
    pub identifier: Identifier,
    pub contents: Vec<usize>,
    pub scope: Option<String>,
    pub location: Location,
    pub attributes: Vec<Attribute>,
    pub comment: Option<DocComment>,
}

impl Module {
    pub fn new(
        identifier: Identifier,
        contents: Vec<usize>,
        attributes: Vec<Attribute>,
        comment: Option<DocComment>,
        location: Location,
    ) -> Self {
        Module { identifier, contents, scope: None, attributes, comment, location }
    }

    pub fn contents<'a>(&self, ast: &'a Ast) -> Vec<&'a dyn NamedSymbol> {
        self.contents
            .iter()
            .map(|id| ast.resolve_index(*id).as_named_symbol().unwrap())
            .collect()
    }
}

#[derive(Clone, Debug)]
pub struct Struct {
    pub identifier: Identifier,
    pub members: Vec<usize>,
    pub scope: Option<String>,
    pub attributes: Vec<Attribute>,
    pub comment: Option<DocComment>,
    pub location: Location,
}

impl Struct {
    pub fn new(
        identifier: Identifier,
        members: Vec<usize>,
        attributes: Vec<Attribute>,
        comment: Option<DocComment>,
        location: Location,
    ) -> Self {
        Struct { identifier, members, scope: None, attributes, comment, location }
    }

    pub fn members<'a>(&self, ast: &'a Ast) -> Vec<&'a Member> {
        self.members
            .iter()
            .map(|id| ref_from_node!(Node::Member, ast, *id))
            .collect()
    }
}

impl Type for Struct {
    fn is_fixed_size(&self, ast: &Ast) -> bool {
        for member in self.members(ast) {
            let data_type = ast
                .resolve_index(member.data_type.definition.unwrap())
                .as_type();
            if !data_type.unwrap().is_fixed_size(ast) {
                return false;
            }
        }
        true
    }

    fn min_wire_size(&self, ast: &Ast) -> u32 {
        let mut size = 0;
        for member in self.members(ast) {
            size += member.data_type
                .definition(ast)
                .as_type()
                .unwrap()
                .min_wire_size(ast);
        }
        size
    }
}

#[derive(Clone, Debug)]
pub struct Class {
    pub identifier: Identifier,
    pub members: Vec<usize>,
    pub scope: Option<String>,
    pub attributes: Vec<Attribute>,
    pub comment: Option<DocComment>,
    pub location: Location,
}

impl Class {
    pub fn new(
        identifier: Identifier,
        members: Vec<usize>,
        attributes: Vec<Attribute>,
        comment: Option<DocComment>,
        location: Location,
    ) -> Self {
        Class { identifier, members, scope: None, attributes, comment, location }
    }

    pub fn members<'a>(&self, ast: &'a Ast) -> Vec<&'a Member> {
        self.members
            .iter()
            .map(|id| ref_from_node!(Node::Member, ast, *id))
            .collect()
    }
}

impl Type for Class {
    fn is_fixed_size(&self, ast: &Ast) -> bool {
        for member in self.members(ast) {
            let data_type = ast
                .resolve_index(member.data_type.definition.unwrap())
                .as_type();
            if !data_type.unwrap().is_fixed_size(ast) {
                return false;
            }
        }
        true
    }

    fn min_wire_size(&self, ast: &Ast) -> u32 {
        let mut size = 0;
        for member in self.members(ast) {
            size += member.data_type
                .definition(ast)
                .as_type()
                .unwrap()
                .min_wire_size(ast);
        }
        size
    }
}

#[derive(Clone, Debug)]
pub struct Exception {
    pub identifier: Identifier,
    pub members: Vec<usize>,
    pub scope: Option<String>,
    pub attributes: Vec<Attribute>,
    pub comment: Option<DocComment>,
    pub location: Location,
}

impl Exception {
    pub fn new(
        identifier: Identifier,
        members: Vec<usize>,
        attributes: Vec<Attribute>,
        comment: Option<DocComment>,
        location: Location,
    ) -> Self {
        Exception { identifier, members, scope: None, attributes, comment, location }
    }

    pub fn members<'a>(&self, ast: &'a Ast) -> Vec<&'a Member> {
        self.members
            .iter()
            .map(|id| ref_from_node!(Node::Member, ast, *id))
            .collect()
    }
}

#[derive(Clone, Debug)]
pub struct Interface {
    pub identifier: Identifier,
    pub operations: Vec<usize>,
    pub scope: Option<String>,
    pub attributes: Vec<Attribute>,
    pub comment: Option<DocComment>,
    pub location: Location,
}

impl Interface {
    pub fn new(
        identifier: Identifier,
        operations: Vec<usize>,
        attributes: Vec<Attribute>,
        comment: Option<DocComment>,
        location: Location,
    ) -> Self {
        Interface { identifier, operations, scope: None, attributes, comment, location }
    }

    pub fn operations<'a>(&self, ast: &'a Ast) -> Vec<&'a Operation> {
        self.operations
            .iter()
            .map(|id| ref_from_node!(Node::Operation, ast, *id))
            .collect()
    }
}

impl Type for Interface {
    fn is_fixed_size(&self, _: &Ast) -> bool {
        false
    }

    fn min_wire_size(&self, _: &Ast) -> u32 {
        3
    }
}

#[derive(Clone, Debug)]
pub struct Enum {
    pub identifier: Identifier,
    pub enumerators: Vec<usize>,
    pub is_unchecked: bool,
    pub underlying: Option<TypeRef>,
    pub scope: Option<String>,
    pub attributes: Vec<Attribute>,
    pub comment: Option<DocComment>,
    pub location: Location,
}

impl Enum {
    pub fn new(
        identifier: Identifier,
        enumerators: Vec<usize>,
        is_unchecked: bool,
        underlying: Option<TypeRef>,
        attributes: Vec<Attribute>,
        comment: Option<DocComment>,
        location: Location,
    ) -> Self {
        Enum {
            identifier,
            enumerators,
            is_unchecked,
            underlying,
            scope: None,
            attributes,
            comment,
            location,
        }
    }

    /// Returns the min enum value if the enum is non-empty.
    pub fn min_value(&self, ast: &Ast) -> Option<i64> {
        self.enumerators(ast)
            .iter()
            .map(|enumerator| enumerator.value)
            .min()
    }

    /// Returns the max enum value if the enum is non-empty.
    pub fn max_value(&self, ast: &Ast) -> Option<i64> {
        self.enumerators(ast)
            .iter()
            .map(|enumerator| enumerator.value)
            .max()
    }

    pub fn enumerators<'a>(&self, ast: &'a Ast) -> Vec<&'a Enumerator> {
        self.enumerators
            .iter()
            .map(|id| ref_from_node!(Node::Enumerator, ast, *id))
            .collect()
    }
}

impl Type for Enum {
    fn is_fixed_size(&self, ast: &Ast) -> bool {
        if let Some(typeref) = &self.underlying {
            typeref.definition(ast)
                   .as_type()
                   .unwrap()
                   .is_fixed_size(ast)
        } else {
            true
        }
    }

    fn min_wire_size(&self, ast: &Ast) -> u32 {
        if let Some(typeref) = &self.underlying {
            typeref.definition(ast)
                   .as_type()
                   .unwrap()
                   .min_wire_size(ast)
        } else {
            1
        }
    }
}

#[derive(Clone, Debug)]
pub enum ReturnType {
    Void(Location),
    Single(TypeRef, Location),
    Tuple(Vec<usize>, Location),
}

impl Symbol for ReturnType {
    fn location(&self) -> &Location {
        match self {
            Self::Void(location)      => location,
            Self::Single(_, location) => location,
            Self::Tuple(_, location)  => location,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Operation {
    pub return_type: ReturnType,
    pub parameters: Vec<usize>,
    pub identifier: Identifier,
    pub scope: Option<String>,
    pub attributes: Vec<Attribute>,
    pub comment: Option<DocComment>,
    pub location: Location,
}

impl Operation {
    pub fn new(
        return_type: ReturnType,
        identifier: Identifier,
        parameters: Vec<usize>,
        attributes: Vec<Attribute>,
        comment: Option<DocComment>,
        location: Location,
    ) -> Self {
        Operation {
            return_type,
            parameters,
            identifier,
            scope: None,
            attributes,
            comment,
            location,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Member {
    pub data_type: TypeRef,
    pub identifier: Identifier,
    pub tag: Option<u32>,
    pub member_type: MemberType,
    pub scope: Option<String>,
    pub attributes: Vec<Attribute>,
    pub comment: Option<DocComment>,
    pub location: Location,
}

impl Member {
    pub fn new(
        data_type: TypeRef,
        identifier: Identifier,
        tag: Option<u32>,
        member_type: MemberType,
        attributes: Vec<Attribute>,
        comment: Option<DocComment>,
        location: Location,
    ) -> Self {
        Member {
            data_type,
            identifier,
            tag,
            member_type,
            scope: None,
            attributes,
            comment,
            location,
        }
    }
}

impl Element for Member {
    fn kind(&self) -> &'static str {
        match self.member_type {
            MemberType::DataMember    => "data member",
            MemberType::Parameter     => "parameter",
            MemberType::ReturnElement => "return element",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum MemberType {
    DataMember,
    Parameter,
    ReturnElement,
}

#[derive(Clone, Debug)]
pub struct Enumerator {
    pub identifier: Identifier,
    pub value: i64,
    pub scope: Option<String>,
    pub attributes: Vec<Attribute>,
    pub comment: Option<DocComment>,
    pub location: Location,
}

impl Enumerator {
    pub fn new(
        identifier: Identifier,
        value: i64,
        attributes: Vec<Attribute>,
        comment: Option<DocComment>,
        location: Location,
    ) -> Self {
        Enumerator { identifier, value, scope: None, attributes, comment, location }
    }
}

#[derive(Clone, Debug)]
pub struct Identifier {
    pub value: String,
    pub location: Location,
}

impl Identifier {
    pub fn new(value: String, location: Location) -> Self {
        Identifier { value, location }
    }
}

#[derive(Clone, Debug)]
pub struct TypeRef {
    pub type_name: String,
    pub is_optional: bool,
    pub is_streamed: bool,
    pub definition: Option<usize>,
    pub scope: Option<String>,
    pub attributes: Vec<Attribute>,
    pub location: Location,
}

impl TypeRef {
    pub fn new(
        type_name: String,
        is_optional: bool,
        attributes: Vec<Attribute>,
        location: Location,
    ) -> Self {
        TypeRef {
            type_name,
            is_optional,
            is_streamed: false,
            definition: None,
            scope: None,
            attributes,
            location,
        }
    }

    pub fn definition<'a>(&self, ast: &'a Ast) -> &'a Node {
        ast.resolve_index(self.definition.unwrap())
    }

    pub fn min_wire_size(&self, ast: &Ast) -> u32 {
        let node = self.definition(ast);

        if self.is_optional {
            match node {
                Node::Interface(_, _) => 1,
                // Node::Class(_, _) => 1, TODO: class support
                _ => 0,
            }
        } else {
            node.as_type().unwrap().min_wire_size(ast)
        }
    }

    pub fn encode_using_bit_sequence(&self, ast: &Ast) -> bool {
        self.is_optional && self.min_wire_size(ast) == 0
    }
}

#[derive(Clone, Debug)]
pub struct Sequence {
    pub element_type: TypeRef,
    pub scope: Option<String>,
}

impl Sequence {
    pub fn new(element_type: TypeRef) -> Self {
        Sequence { element_type, scope: None }
    }

    pub fn is_element_fixed_sized_numeric(&self, ast: &Ast) -> bool {
        let mut element_node = self.element_type.definition(ast);

        // If the elements are an enum with an underlying type, check the underlying type instead.
        if let Node::Enum(_, enum_def) = element_node {
            if let Some(underlying) = &enum_def.underlying {
                element_node = underlying.definition(ast);
            }
        }

        if let Node::Primitive(_, primitive) = element_node {
            primitive.is_numeric_or_bool() && primitive.is_fixed_size(ast)
        } else {
            false
        }
    }
}

impl Type for Sequence {
    fn is_fixed_size(&self, _: &Ast) -> bool {
        false
    }

    fn min_wire_size(&self, _: &Ast) -> u32 {
        1
    }
}

#[derive(Clone, Debug)]
pub struct Dictionary {
    pub key_type: TypeRef,
    pub value_type: TypeRef,
    pub scope: Option<String>,
}

impl Dictionary {
    pub fn new(key_type: TypeRef, value_type: TypeRef) -> Self {
        Dictionary { key_type, value_type, scope: None }
    }
}

impl Type for Dictionary {
    fn is_fixed_size(&self, _: &Ast) -> bool {
        false
    }

    fn min_wire_size(&self, _: &Ast) -> u32 {
        1
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Primitive {
    Bool,
    Byte,
    Short,
    UShort,
    Int,
    UInt,
    VarInt,
    VarUInt,
    Long,
    ULong,
    VarLong,
    VarULong,
    Float,
    Double,
    String,
}

impl Primitive {
    pub fn is_numeric_or_bool(&self) -> bool {
        !matches!(&self, Self::String)
    }
}

impl Element for Primitive {
    fn kind(&self) -> &'static str {
        match self {
            Self::Bool => "bool",
            Self::Byte => "byte",
            Self::Short => "short",
            Self::UShort => "ushort",
            Self::Int => "int",
            Self::UInt => "uint",
            Self::VarInt => "varint",
            Self::VarUInt => "varuint",
            Self::Long => "long",
            Self::ULong => "ulong",
            Self::VarLong => "varlong",
            Self::VarULong => "varulong",
            Self::Float => "float",
            Self::Double => "double",
            Self::String => "string",
        }
    }
}

impl Type for Primitive {
    fn is_fixed_size(&self, _: &Ast) -> bool {
        !matches!(self,
            Self::VarInt | Self::VarUInt | Self::VarLong | Self::VarULong | Self::String
        )
    }

    fn min_wire_size(&self, _: &Ast) -> u32 {
        match self {
            Self::Bool => 1,
            Self::Byte => 1,
            Self::Short => 2,
            Self::UShort => 2,
            Self::Int => 4,
            Self::UInt => 4,
            Self::VarInt => 1,
            Self::VarUInt => 1,
            Self::Long => 8,
            Self::ULong => 8,
            Self::VarLong => 1,
            Self::VarULong => 1,
            Self::Float => 4,
            Self::Double => 8,
            Self::String => 1,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Attribute {
    /// If the attribute's directive had a language mapping prefix, it is stored here, otherwise
    /// this is `None`. Ex: the prefix for `cs::readonly` would be `cs`.
    pub prefix: Option<String>,
    /// The attribute's directive, without it's prefix if one was present.
    pub directive: String,
    /// Stores the fully qualified directive (the prefix and directive with a `::` separator).
    /// We compute this up-front, to make searching for fully qualified metadata more efficient.
    pub qualified_directive: String,
    /// Stores all the arguments passed into the directive, in the order they were passed.
    /// For directives that don't take any arguments, this should always be empty.
    pub arguments: Vec<String>,
    pub location: Location,
}

impl Attribute {
    pub fn new(
        prefix: Option<String>,
        directive: String,
        arguments: Vec<String>,
        location: Location,
    ) -> Self {
        // Combine the prefix and directive together to make searching qualified directives easier.
        let qualified_directive = prefix.clone().unwrap_or("".to_owned()) + &directive;
        Attribute { prefix, directive, qualified_directive, arguments, location }
    }
}

#[derive(Clone, Debug)]
pub struct DocComment {
    pub message: String,
    pub references: Vec<String>,
    pub params: Vec<(String, String)>,
    pub returns: Option<String>,
    pub throws: Vec<(String, String)>,
    pub location: Location,
}
