
use std::str::FromStr;
use crate::visitor::Visitable;



pub trait Definition : Visitable {}



pub struct Module
{
    pub identifier: String,
    pub content: Vec<Box<dyn Definition>>,
}

impl Definition for Module {}



pub struct Struct
{
    pub identifier: String,
    pub content: Vec<DataMember>,
}

impl Definition for Struct {}



pub struct Interface
{
    pub identifier: String,
}

impl Definition for Interface {}



pub struct DataMember
{
    pub typename: Type,
    pub identifier: String,
}



pub enum Type
{
    BuiltIn(BuiltInType),
    Custom(String),
}

impl ToString for Type
{
    fn to_string(&self) -> String
    {
        match &self {
            Type::BuiltIn(builtin) => builtin.to_string(),
            Type::Custom(custom) => custom.to_owned(),
        }
    }
}

impl FromStr for Type {
    type Err = ();

    fn from_str(s: &str) -> Result<Type, Self::Err> {
        let result = match BuiltInType::from_str(s) {
            Ok(x) => Type::BuiltIn(x),
            Err(_) => Type::Custom(s.to_owned()),
        };
        Ok(result)
    }
}



pub enum BuiltInType
{
    Int,
    String,
}

impl FromStr for BuiltInType {
    type Err = ();

    fn from_str(s: &str) -> Result<BuiltInType, Self::Err> {
        match s {
            "int"    => Ok(BuiltInType::Int),
            "string" => Ok(BuiltInType::String),
            _        => Err(()),
        }
    }
}

impl ToString for BuiltInType {
    fn to_string(&self) -> String
    {
        match &self {
            BuiltInType::Int => "int".to_owned(),
            BuiltInType::String => "string".to_owned(),
        }
    }
}
