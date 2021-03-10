// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::util::Location;

//------------------------------------------------------------------------------
// Symbol
//------------------------------------------------------------------------------
pub trait Symbol {
    fn location(&self) -> &Location;
    fn kind(&self) -> &'static str;
}

macro_rules! implement_symbol_for {
    ($a:ty, $b:literal) => {
        impl Symbol for $a {
            fn location(&self) -> &Location {
                &self.location
            }

            fn kind(&self) -> &'static str {
                $b
            }
        }
    }
}

implement_symbol_for!(Module, "module");
implement_symbol_for!(Struct, "struct");
implement_symbol_for!(Interface, "interface");
implement_symbol_for!(DataMember, "data member");
implement_symbol_for!(Identifier, "identifier");
implement_symbol_for!(TypeUse, "type use");

//------------------------------------------------------------------------------
// NamedSymbol
//------------------------------------------------------------------------------
pub trait NamedSymbol : Symbol {
    fn identifier(&self) -> &str;
}

macro_rules! implement_named_symbol_for {
    ($a:ty) => {
        impl NamedSymbol for $a {
            fn identifier(&self) -> &str {
                &self.identifier()
            }
        }
    }
}

implement_named_symbol_for!(Module);
implement_named_symbol_for!(Struct);
implement_named_symbol_for!(Interface);
implement_named_symbol_for!(DataMember);

//------------------------------------------------------------------------------
// Type
//------------------------------------------------------------------------------
pub trait Type {}

//------------------------------------------------------------------------------
// Module
//------------------------------------------------------------------------------
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

    pub fn identifier(&self) -> &str {
        &self.identifier.value
    }
}

//------------------------------------------------------------------------------
// Struct
//------------------------------------------------------------------------------
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

    pub fn identifier(&self) -> &str {
        &self.identifier.value
    }
}

impl Type for Struct {}

//------------------------------------------------------------------------------
// Interface
//------------------------------------------------------------------------------
#[derive(Clone, Debug)]
pub struct Interface {
    pub identifier: Identifier,
    pub scope: Option<String>,
    pub location: Location,
}

impl Interface {
    pub fn new(identifier: Identifier, location: Location) -> Self {
        Interface { identifier, scope: None, location }
    }

    pub fn identifier(&self) -> &str {
        &self.identifier.value
    }
}

impl Type for Interface {}

//------------------------------------------------------------------------------
// DataMember
//------------------------------------------------------------------------------
#[derive(Clone, Debug)]
pub struct DataMember {
    pub data_type: TypeUse,
    pub identifier: Identifier,
    pub scope: Option<String>,
    pub location: Location,
}

impl DataMember {
    pub fn new(data_type: TypeUse, identifier: Identifier, location: Location) -> Self {
        DataMember { data_type, identifier, scope: None, location }
    }

    pub fn identifier(&self) -> &str {
        &self.identifier.value
    }
}

//------------------------------------------------------------------------------
// Identifier
//------------------------------------------------------------------------------
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

//------------------------------------------------------------------------------
// TypeUse
//------------------------------------------------------------------------------
#[derive(Clone, Debug)]
pub struct TypeUse {
    pub type_name: String,
    pub is_tagged: bool,
    pub definition: Option<usize>,
    pub location: Location,
}

impl TypeUse {
    pub fn new(type_name: String, is_tagged: bool, location: Location) -> Self {
        TypeUse { type_name, is_tagged, definition: None, location }
    }
}

//------------------------------------------------------------------------------
// Builtin
//------------------------------------------------------------------------------
#[derive(Clone, Eq, Hash, PartialEq, Debug)]
pub enum Builtin {
    Int,
    String,
}

impl Type for Builtin {}

impl From<&str> for Builtin {
    fn from(s: &str) -> Self {
        match s {
            "int" => Self::Int,
            "string" => Self::String,
            _ => panic!("`{}` is not a valid builtin type!", s),
        }
    }
}
