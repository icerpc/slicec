
use crate::util::{ParserError, Location};

use std::collections::HashMap;
use std::str::FromStr;

//------------------------------------------------------------------------------
// Base Traits
//------------------------------------------------------------------------------
pub trait Node {
    fn location(&self) -> &Location;
}

pub trait Type {
    // TODO
}

//------------------------------------------------------------------------------
// Module
//------------------------------------------------------------------------------
#[derive(Debug)]
pub struct Module {
    identifier: Identifier,
    contents: Vec<usize>,
    location: Location,
}

impl Module {
    pub fn new(identifier: Identifier, contents: Vec<usize>, location: Location) -> Self {
        // TODO do validation here

        Module { identifier, contents, location }
    }

    pub fn identifier(&self) -> &str {
        &self.identifier.value()
    }

    pub fn contents(&self) -> &Vec<usize> {
        &self.contents
    }
}

impl Node for Module {
    fn location(&self) -> &Location {
        &self.location
    }
}

//------------------------------------------------------------------------------
// Struct
//------------------------------------------------------------------------------
#[derive(Debug)]
pub struct Struct {
    identifier: Identifier,
    contents: Vec<DataMember>,
    location: Location,
}

impl Struct {
    pub fn new(identifier: Identifier, contents: Vec<DataMember>, location: Location) -> Self {
        // TODO do validation here

        Struct { identifier, contents, location }
    }

    pub fn identifier(&self) -> &str {
        &self.identifier.value()
    }

    pub fn contents(&self) -> &Vec<DataMember> {
        &self.contents
    }
}

impl Type for Struct {
    //TODO
}

impl Node for Struct {
    fn location(&self) -> &Location {
        &self.location
    }
}

//------------------------------------------------------------------------------
// Interface
//------------------------------------------------------------------------------
#[derive(Debug)]
pub struct Interface {
    identifier: Identifier,
    location: Location,
}

impl Interface {
    pub fn new(identifier: Identifier, location: Location) -> Self {
        // TODO do validation here

        Interface { identifier, location }
    }

    pub fn identifier(&self) -> &str {
        &self.identifier.value()
    }
}

impl Type for Interface {
    // TODO
}

impl Node for Interface {
    fn location(&self) -> &Location {
        &self.location
    }
}

//------------------------------------------------------------------------------
// DataMember
//------------------------------------------------------------------------------
#[derive(Debug)]
pub struct DataMember {
    data_type: TypeUse,
    identifier: Identifier,
    location: Location,
}

impl DataMember {
    pub fn new(data_type: TypeUse, identifier: Identifier, location: Location) -> Self {
        // TODO do validation here

        DataMember { data_type, identifier, location }
    }

    pub fn data_type(&self) -> &TypeUse {
        &self.data_type
    }

    pub fn identifier(&self) -> &str {
        &self.identifier.value()
    }
}

impl Node for DataMember {
    fn location(&self) -> &Location {
        &self.location
    }
}

//------------------------------------------------------------------------------
// Identifier
//------------------------------------------------------------------------------
#[derive(Debug)]
pub struct Identifier {
    value: String,
    location: Location,
}

impl Identifier {
    pub fn new(value: String, location: Location) -> Self {
        // TODO do validation here

        Identifier { value, location }
    }

    pub fn value(&self) -> &str {
        &self.value
    }
}

impl Node for Identifier {
    fn location(&self) -> &Location {
        &self.location
    }
}

//------------------------------------------------------------------------------
// TypeUse
//------------------------------------------------------------------------------
#[derive(Debug)]
pub struct TypeUse {
    type_name: String,
    is_tagged: bool,
    definition: Option<usize>,
    location: Location,
}

impl TypeUse {
    pub fn new(type_name: String, is_tagged: bool, location: Location) -> Self {
        // TODO do validation here

        TypeUse { type_name, is_tagged, definition: None, location }
    }

    pub fn type_name(&self) -> &str {
        &self.type_name
    }

    pub fn is_tagged(&self) -> &bool {
        &self.is_tagged
    }

    pub fn definition(&self) -> &usize {
        // panic if we try to access the definition before it's been patched.
        &self.definition.expect(
            format!("Failed to unwrap definition for type: {}\n{:?}", &self.type_name, &self)
        )
    }

    pub fn patch_definition(&mut self, type_table: &HashMap<String, usize>) -> Result<(), ParserError> {
        // Ensure that the definition hasn't already been patched.
        assert!(self.definition.is_none());

        // Try to resolve the type and store it's index.
        if let Some(resolved) = type_table.get(&self.type_name) {
            self.definition = Some(resolved.clone());
            Ok(())
        } else {
            Err(ParserError::new(
                format!("No definition was found for `{}` in this scope", self.type_name.clone()),
                self.location.clone(),
            ))
        }
    }
}

impl Node for TypeUse {
    fn location(&self) -> &Location {
        &self.location
    }
}

//------------------------------------------------------------------------------
// BuiltIn
//------------------------------------------------------------------------------
#[derive(Debug)]
pub enum BuiltIn {
    Int,
    String,
}

impl BuiltIn {
    // TODO
}

impl Type for BuiltIn {
    // TODO
}

impl FromStr for BuiltIn {
    fn from_str(s: &str) -> Result<BuiltIn, String> {
        match s {
            "int"    => Ok(BuiltIn::Int),
            "string" => Ok(BuiltIn::String),
            _        => Err(format!("{} is not a builtin type", s))
        }
    }
}
