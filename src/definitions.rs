
trait Definition {}

pub struct Module
{
    identifier: String,
    content: Vec<Box<dyn Definition>>,
}

impl Definition for Module {}

pub struct Struct
{
    identifier: String,
    content: Vec<DataMember>,
}

impl Definition for Struct {}

pub struct Interface
{
    identifier: String,
}

impl Definition for Interface {}

pub struct DataMember
{
    typename: Type,
    identifier: String,
}

enum BuiltInType
{
    Int,
    String,
}

enum Type
{
    BuiltIn(BuiltInType),
    Custom(String),
}
