
use crate::util::{SliceError, Location};

use std::collections::HashMap;
use std::str::FromStr;

//------------------------------------------------------------------------------
// Module
//------------------------------------------------------------------------------
#[derive(Debug)]
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
#[derive(Debug)]
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

impl Type for Struct {
    //TODO
}

//------------------------------------------------------------------------------
// Interface
//------------------------------------------------------------------------------
#[derive(Debug)]
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

impl Type for Interface {
    // TODO
}

//------------------------------------------------------------------------------
// DataMember
//------------------------------------------------------------------------------
#[derive(Debug)]
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
#[derive(Debug)]
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
#[derive(Debug)]
pub struct TypeUse {
    pub type_name: String,
    pub is_tagged: bool,
    pub location: Location,
    definition: Option<usize>,
}

impl TypeUse {
    pub fn new(type_name: String, is_tagged: bool, location: Location) -> Self {
        TypeUse { type_name, is_tagged, definition: None, location }
    }

    pub fn definition(&self) -> usize {
        // Panic if we try to access the definition before it's been patched.
        match self.definition {
            Some(value) => value,
            None => { panic!("Failed to unwrap definition!\n\t{:?}", &self) },
        }
    }

    pub fn patch_definition(&mut self, type_table: &HashMap<String, usize>) -> Result<(), SliceError> {
        // Panic if the definition has already been patched.
        if self.definition.is_some() {
            panic!("Definition has already been patched!\n\t{:?}", &self);
        }

        // Try to resolve the type and store it's index.
        if let Some(resolved) = type_table.get(&self.type_name) {
            self.definition = Some(resolved.clone());
            Ok(())
        } else {
            Err(SliceError::new(
                format!("No definition was found for `{}` in this scope", &self.type_name),
                self.location,
            ))
        }
    }
}

//------------------------------------------------------------------------------
// Type
//------------------------------------------------------------------------------
pub trait Type {
    // TODO
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
    type Err = String;

    fn from_str(s: &str) -> Result<BuiltIn, Self::Err> {
        match s {
            "int"    => Ok(BuiltIn::Int),
            "string" => Ok(BuiltIn::String),
            _        => Err(format!("{} is not a builtin type", s))
        }
    }
}
