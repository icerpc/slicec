// Copyright (c) ZeroC, Inc. All rights reserved.

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
// Member has it's own custom implementation of Element which depends on it's member type.
implement_element_for!(Enumerator, "enumerator");
implement_element_for!(Identifier, "identifier");
implement_element_for!(TypeRef, "type ref");
implement_element_for!(Sequence, "sequence");
implement_element_for!(Dictionary, "dictionary");
// Primitive has it's own custom implementation of Element which returns the primitive's type name.

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
// ReturnType has it's own custom implementation of Symbol, since it's an enum instead of a struct.
implement_symbol_for!(Operation);
implement_symbol_for!(Member);
implement_symbol_for!(Enumerator);
implement_symbol_for!(Identifier);
implement_symbol_for!(TypeRef);

/// NamedSymbols are symbols that have an identifier attached to them.
pub trait NamedSymbol : Symbol {
    fn identifier(&self) -> &str;
}

macro_rules! implement_named_symbol_for {
    ($a:ty) => {
        impl NamedSymbol for $a {
            fn identifier(&self) -> &str {
                &self.identifier.value
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
pub trait Type {}

#[derive(Clone, Debug)]
pub struct Module {
    pub identifier: Identifier,
    pub contents: Vec<usize>,
    pub scope: Option<String>,
    pub location: Location,
}

impl Module {
    pub fn new(identifier: Identifier, contents: Vec<usize>, location: Location) -> Self {
        Module { identifier, contents, scope: None, location }
    }
}

#[derive(Clone, Debug)]
pub struct Struct {
    pub identifier: Identifier,
    pub contents: Vec<usize>,
    pub scope: Option<String>,
    pub location: Location,
}

impl Struct {
    pub fn new(identifier: Identifier, contents: Vec<usize>, location: Location) -> Self {
        Struct { identifier, contents, scope: None, location }
    }
}

impl Type for Struct {}

#[derive(Clone, Debug)]
pub struct Interface {
    pub identifier: Identifier,
    pub operations: Vec<usize>,
    pub scope: Option<String>,
    pub location: Location,
}

impl Interface {
    pub fn new(identifier: Identifier, operations: Vec<usize>, location: Location) -> Self {
        Interface { identifier, operations, scope: None, location }
    }
}

impl Type for Interface {}

#[derive(Clone, Debug)]
pub struct Enum {
    pub identifier: Identifier,
    pub contents: Vec<usize>,
    pub is_checked: bool,
    pub underlying: Option<TypeRef>,
    pub scope: Option<String>,
    pub location: Location,
}

impl Enum {
    pub fn new(
        identifier: Identifier,
        contents: Vec<usize>,
        is_checked: bool,
        underlying: Option<TypeRef>,
        location: Location
    ) -> Self {
        Enum { identifier, contents, is_checked, underlying, scope: None, location }
    }
}

impl Type for Enum {}

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
    pub location: Location,
}

impl Operation {
    pub fn new(
        return_type: ReturnType,
        identifier: Identifier,
        parameters: Vec<usize>,
        location: Location
    ) -> Self {
        Operation { return_type, parameters, identifier, scope: None, location }
    }
}

#[derive(Clone, Debug)]
pub struct Member {
    pub data_type: TypeRef,
    pub identifier: Identifier,
    pub member_type: MemberType,
    pub scope: Option<String>,
    pub location: Location,
}

impl Member {
    pub fn new(
        data_type: TypeRef,
        identifier: Identifier,
        member_type: MemberType,
        location: Location,
    ) -> Self {
        Member { data_type, identifier, member_type, scope: None, location }
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
    pub location: Location,
}

impl Enumerator {
    pub fn new(identifier: Identifier, value: i64, location: Location) -> Self {
        Enumerator { identifier, value, scope: None, location }
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

impl Type for Sequence {}

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

impl Type for Dictionary {}

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

impl Type for Primitive {}
