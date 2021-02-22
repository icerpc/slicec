
use crate::util::Location;

//------------------------------------------------------------------------------
// Element
//------------------------------------------------------------------------------
pub trait Element {
    fn kind(&self) -> ElementKind;
    fn location(&self) -> Location;
}

macro_rules! implement_element_for{
    ($a:ty, $b:ident, $c:ident) => {
        impl Element for $a {
            fn kind(&self) -> ElementKind {
                ElementKind::$b
            }

            fn location(&self) -> Location {
                self.$c.clone()
            }
        }
    }
}

implement_element_for!(Module, KindModule, location);
implement_element_for!(Struct, KindStruct, location);
implement_element_for!(Interface, KindInterface, location);
implement_element_for!(DataMember, KindDataMember, location);
implement_element_for!(Identifier, KindIdentifier, location);
implement_element_for!(TypeUse, KindTypeUse, location);

//------------------------------------------------------------------------------
// ElementKind
//------------------------------------------------------------------------------
#[derive(Clone, Eq, Hash, PartialEq, Debug)]
pub enum ElementKind {
    KindModule,
    KindStruct,
    KindInterface,
    KindDataMember,
    KindIdentifier,
    KindTypeUse,
}

//------------------------------------------------------------------------------
// Module
//------------------------------------------------------------------------------
#[derive(Clone, Debug)]
pub struct Module {
    pub identifier: Identifier,
    pub contents: Vec<usize>,
    pub location: Location,
    pub def_index: usize,
}

impl Module {
    pub fn new(identifier: Identifier, contents: Vec<usize>, location: Location) -> Self {
        Module { identifier, contents, location, def_index: usize::MAX }
    }

    pub fn get_identifier(&self) -> &str {
        &self.identifier.value
    }
}

//------------------------------------------------------------------------------
// Struct
//------------------------------------------------------------------------------
#[derive(Clone, Debug)]
pub struct Struct {
    pub identifier: Identifier,
    pub contents: Vec<DataMember>,
    pub location: Location,
    pub def_index: usize,
}

impl Struct {
    pub fn new(identifier: Identifier, contents: Vec<DataMember>, location: Location) -> Self {
        Struct { identifier, contents, location, def_index: usize::MAX }
    }

    pub fn get_identifier(&self) -> &str {
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
    pub def_index: usize,
}

impl Interface {
    pub fn new(identifier: Identifier, location: Location) -> Self {
        Interface { identifier, location, def_index: usize::MAX }
    }

    pub fn get_identifier(&self) -> &str {
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

    pub fn get_identifier(&self) -> &str {
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
// BuiltIn
//------------------------------------------------------------------------------
#[derive(Clone, Eq, Hash, PartialEq, Debug)]
pub enum BuiltIn {
    Int,
    String,
}

impl Type for BuiltIn {}
