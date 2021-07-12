// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::ref_from_node;
use crate::ast::{Ast, Node};
use crate::util::Location;

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
    }
}

implement_element_for!(Module, "module");
implement_element_for!(Struct, "struct");
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
implement_element_for!(Metadata, "metadata");
implement_element_for!(DocComment, "comment");

/// Symbols represent elements of the actual source code written in the slice file.
pub trait Symbol : Element {
    fn location(&self) -> &Location;
}

macro_rules! implement_symbol_for {
    ($a:ty) => {
        impl Symbol for $a {
            fn location(&self) -> &Location {
                &self.location
            }
        }
    }
}

implement_symbol_for!(Module);
implement_symbol_for!(Struct);
implement_symbol_for!(Interface);
implement_symbol_for!(Enum);
// ReturnType has its own custom implementation of Symbol, since it's an enum instead of a struct.
implement_symbol_for!(Operation);
implement_symbol_for!(Member);
implement_symbol_for!(Enumerator);
implement_symbol_for!(Identifier);
implement_symbol_for!(TypeRef);
implement_symbol_for!(Metadata);
implement_symbol_for!(DocComment);

/// NamedSymbols are symbols that have an identifier attached to them.
pub trait NamedSymbol : Symbol {
    fn identifier(&self) -> &str;
    fn metadata(&self) -> &Vec<Metadata>;
    fn find_metadata(&self, directive: &str) -> Option<&Vec<String>>;
    fn has_metadata(&self, directive: &str) -> bool;
    fn comment(&self) -> Option<&DocComment>;
}

macro_rules! implement_named_symbol_for {
    ($a:ty) => {
        impl NamedSymbol for $a {
            fn identifier(&self) -> &str {
                &self.identifier.value
            }

            fn metadata(&self) -> &Vec<Metadata> {
                &self.metadata
            }

            /// Checks if the symbol has the specified metadata attribute, and if so, returns it's
            /// attributes as a vector. If it doesn't, it returns 'None'.
            fn find_metadata(&self, directive: &str) -> Option<&Vec<String>> {
                for m in &self.metadata {
                    if m.raw_directive == directive {
                        return Some(&m.arguments);
                    }
                }
                return None;
            }

            /// Returns true if the symbol has the specified metadata attribute on it,
            /// and false otherwise.
            fn has_metadata(&self, directive: &str) -> bool {
                self.find_metadata(directive).is_some()
            }

            fn comment(&self) -> Option<&DocComment> {
                self.comment.as_ref()
            }
        }
    }
}

implement_named_symbol_for!(Module);
implement_named_symbol_for!(Struct);
implement_named_symbol_for!(Interface);
implement_named_symbol_for!(Enum);
implement_named_symbol_for!(Operation);
implement_named_symbol_for!(Member);
implement_named_symbol_for!(Enumerator);

/// Base trait that all elements representing types implement.
pub trait Type {
    fn is_fixed_size(&self, ast: &Ast) -> bool;
}

#[derive(Clone, Debug)]
pub struct Module {
    pub identifier: Identifier,
    pub contents: Vec<usize>,
    pub scope: Option<String>,
    pub location: Location,
    pub metadata: Vec<Metadata>,
    pub comment: Option<DocComment>,
}

impl Module {
    pub fn new(
        identifier: Identifier,
        contents: Vec<usize>,
        metadata: Vec<Metadata>,
        comment: Option<DocComment>,
        location: Location,
    ) -> Self {
        Module { identifier, contents, scope: None, metadata, comment, location }
    }
}

#[derive(Clone, Debug)]
pub struct Struct {
    pub identifier: Identifier,
    pub contents: Vec<usize>,
    pub scope: Option<String>,
    pub metadata: Vec<Metadata>,
    pub comment: Option<DocComment>,
    pub location: Location,
}

impl Struct {
    pub fn new(
        identifier: Identifier,
        contents: Vec<usize>,
        metadata: Vec<Metadata>,
        comment: Option<DocComment>,
        location: Location,
    ) -> Self {
        Struct { identifier, contents, scope: None, metadata, comment, location }
    }
}

impl Type for Struct {
    fn is_fixed_size(&self, ast: &Ast) -> bool {
        for id in &self.contents {
            let member = ref_from_node!(Node::Member, ast, *id);
            let data_type = ast.resolve_index(member.data_type.definition.unwrap()).as_type();
            if !data_type.unwrap().is_fixed_size(ast) {
                return false;
            }
        }
        true
    }
}

#[derive(Clone, Debug)]
pub struct Interface {
    pub identifier: Identifier,
    pub operations: Vec<usize>,
    pub scope: Option<String>,
    pub metadata: Vec<Metadata>,
    pub comment: Option<DocComment>,
    pub location: Location,
}

impl Interface {
    pub fn new(
        identifier: Identifier,
        operations: Vec<usize>,
        metadata: Vec<Metadata>,
        comment: Option<DocComment>,
        location: Location,
    ) -> Self {
        Interface { identifier, operations, scope: None, metadata, comment, location }
    }
}

impl Type for Interface {
    fn is_fixed_size(&self, _: &Ast) -> bool {
        false
    }
}

#[derive(Clone, Debug)]
pub struct Enum {
    pub identifier: Identifier,
    pub contents: Vec<usize>,
    pub is_checked: bool,
    pub underlying: Option<TypeRef>,
    pub scope: Option<String>,
    pub metadata: Vec<Metadata>,
    pub comment: Option<DocComment>,
    pub location: Location,
}

impl Enum {
    pub fn new(
        identifier: Identifier,
        contents: Vec<usize>,
        is_checked: bool,
        underlying: Option<TypeRef>,
        metadata: Vec<Metadata>,
        comment: Option<DocComment>,
        location: Location,
    ) -> Self {
        Enum {
            identifier,
            contents,
            is_checked,
            underlying,
            scope: None,
            metadata,
            comment,
            location,
        }
    }
}

impl Type for Enum {
    fn is_fixed_size(&self, ast: &Ast) -> bool {
        if let Some(typeref) = &self.underlying {
            let underlying_id = typeref.definition.unwrap();
            let underlying_type = ast.resolve_index(underlying_id).as_type().unwrap();
            return underlying_type.is_fixed_size(ast);
        }
        true
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
    pub metadata: Vec<Metadata>,
    pub comment: Option<DocComment>,
    pub location: Location,
}

impl Operation {
    pub fn new(
        return_type: ReturnType,
        identifier: Identifier,
        parameters: Vec<usize>,
        metadata: Vec<Metadata>,
        comment: Option<DocComment>,
        location: Location,
    ) -> Self {
        Operation { return_type, parameters, identifier, scope: None, metadata, comment, location }
    }
}

#[derive(Clone, Debug)]
pub struct Member {
    pub data_type: TypeRef,
    pub identifier: Identifier,
    pub member_type: MemberType,
    pub scope: Option<String>,
    pub metadata: Vec<Metadata>,
    pub comment: Option<DocComment>,
    pub location: Location,
}

impl Member {
    pub fn new(
        data_type: TypeRef,
        identifier: Identifier,
        member_type: MemberType,
        metadata: Vec<Metadata>,
        comment: Option<DocComment>,
        location: Location,
    ) -> Self {
        Member { data_type, identifier, member_type, scope: None, metadata, comment, location }
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
    pub metadata: Vec<Metadata>,
    pub comment: Option<DocComment>,
    pub location: Location,
}

impl Enumerator {
    pub fn new(
        identifier: Identifier,
        value: i64,
        metadata: Vec<Metadata>,
        comment: Option<DocComment>,
        location: Location,
    ) -> Self {
        Enumerator { identifier, value, scope: None, metadata, comment, location }
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
    pub is_tagged: bool,
    pub definition: Option<usize>,
    pub location: Location,
}

impl TypeRef {
    pub fn new(type_name: String, is_tagged: bool, location: Location) -> Self {
        TypeRef { type_name, is_tagged, definition: None, location }
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
}

impl Type for Sequence {
    fn is_fixed_size(&self, _: &Ast) -> bool {
        false
    }
}

#[derive(Clone, Debug)]
pub struct Dictionary {
    pub key_type: TypeRef,
    pub value_type: TypeRef,
    pub scope: Option<String>,
}

impl Dictionary {
    pub fn new(key_type: TypeRef, value_type: TypeRef,) -> Self {
        Dictionary { key_type, value_type, scope: None }
    }
}

impl Type for Dictionary {
    fn is_fixed_size(&self, _: &Ast) -> bool {
        false
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
        match self {
            Self::VarInt | Self::VarUInt | Self::VarLong | Self::VarULong | Self::String => false,
            _ => true,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Metadata {
    pub prefix: Option<String>,
    pub directive: String,
    pub raw_directive: String,
    pub arguments: Vec<String>,
    pub location: Location,
}

impl Metadata {
    pub fn new(
        prefix: Option<String>,
        directive: String,
        arguments: Vec<String>,
        location: Location,
    ) -> Self {
        // Combine the prefix and directive together to make searching qualified directives easier.
        let raw_directive = prefix.clone().unwrap_or("".to_owned()) + &directive;
        Metadata { prefix, directive, raw_directive, arguments, location}
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
