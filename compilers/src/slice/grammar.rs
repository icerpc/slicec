
use crate::util::Location;

//------------------------------------------------------------------------------
// Element
//------------------------------------------------------------------------------
/// Base trait that all grammar elements implement.
pub trait Element {
    /// Retrieves the location where the grammar element is defined.
    ///
    /// For grammar elements with bodies (like modules), the location only spans the initial definition of the element,
    /// and not the entire body. For instance, the location for `module Foo { ... }` would only span `module Foo`.
    fn location(&self) -> &Location;
}

/// This macro implements the `Element` trait for a grammar element, to reduce boilerplate implementations.
macro_rules! implement_element_for{
    ($a:ty, $b:ident) => {
        impl Element for $a {
            fn location(&self) -> &Location {
                &self.$b
            }
        }
    }
}

implement_element_for!(Module, location);
implement_element_for!(Struct, location);
implement_element_for!(Interface, location);
implement_element_for!(DataMember, location);
implement_element_for!(Identifier, location);
implement_element_for!(TypeUse, location);

// TODO write comments for everything else below this line.

//------------------------------------------------------------------------------
// Module
//------------------------------------------------------------------------------
#[derive(Clone, Debug)]
pub struct Module {
    pub identifier: Identifier,
    pub contents: Vec<usize>,
    pub location: Location,
}

impl Module {
    pub fn new(identifier: Identifier, contents: Vec<usize>, location: Location) -> Self {
        Module { identifier, contents, location }
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
    pub location: Location,
}

impl Struct {
    pub fn new(identifier: Identifier, contents: Vec<usize>, location: Location) -> Self {
        Struct { identifier, contents, location }
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
    pub location: Location,
}

impl Interface {
    pub fn new(identifier: Identifier, location: Location) -> Self {
        Interface { identifier, location }
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
    pub location: Location,
}

impl DataMember {
    pub fn new(data_type: TypeUse, identifier: Identifier, location: Location) -> Self {
        DataMember { data_type, identifier, location }
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
// Type
//------------------------------------------------------------------------------
pub trait Type {}

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
            "String" => Self::String,
            _ => panic!("`{}` is not a valid builtin type!", s),
        }
    }
}
