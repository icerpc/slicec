// Copyright (c) ZeroC, Inc.

use clap::Parser;
use slicec::compilation_state::CompilationState;
use slicec::slice_options::SliceOptions;



use slice_codec::slice2::Slice2;

use slice_codec::buffer::OutputTarget;
use slice_codec::encode_into::*;
use slice_codec::encoder::Encoder;
use slice_codec::Result;

const TAG_END_MARKER: i32 = -1;

pub type EntityId = String;

macro_rules! implement_encode_into_for_struct {
    ($type_name:ty$(, $field_name:ident)*) => {
        impl EncodeInto<Slice2> for &$type_name {
            fn encode_into(self, encoder: &mut Encoder<impl OutputTarget>) -> Result<()> {
                $(encoder.encode(&self.$field_name);)*
                encoder.encode_varint(TAG_END_MARKER);
                Ok(())
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct EntityInfo {
    identifier: String,
    attributes: Option<Vec<Attribute>>, // TAG(0)
    comment: Option<DocComment>, // TAG(1)
}
implement_encode_into_for_struct!(EntityInfo, identifier, attributes, comment);

#[derive(Clone, Debug)]
pub struct Module {
    identifier: String,
    attributes: Option<Vec<Attribute>>, // TAG(0)
}
implement_encode_into_for_struct!(Module, identifier);

#[derive(Clone, Debug)]
pub struct Struct {
    entity_info: EntityInfo,
    fields: Vec<Field>,
    is_compact: bool,
    is_slice1_only: bool,
}
implement_encode_into_for_struct!(Struct, entity_info, fields, is_compact, is_slice1_only);

#[derive(Clone, Debug)]
pub struct Class {
    entity_info: EntityInfo,
    fields: Vec<Field>,
    compact_id: Option<i32>,
    base: Option<EntityId>,
}
implement_encode_into_for_struct!(Class, entity_info, fields, compact_id, base);

#[derive(Clone, Debug)]
pub struct Exception {
    entity_info: EntityInfo,
    fields: Vec<Field>,
    base: Option<EntityId>,
}
implement_encode_into_for_struct!(Exception, entity_info, fields, base);

#[derive(Clone, Debug)]
pub struct Field {
    entity_info: EntityInfo,
    data_type: TypeRef,
    tag: Option<i32>, // TODO: varint32 isn't a real type?
}
// TODO how to encode field properly>

#[derive(Clone, Debug)]
pub struct Interface {
    entity_info: EntityInfo,
    operations: Vec<Operation>,
    bases: Vec<EntityId>,
}
implement_encode_into_for_struct!(Interface, entity_info, operations, bases);

#[derive(Clone, Debug)]
pub struct Operation {
    entity_info: EntityInfo,
    parameters: Vec<Field>,
    has_streamed_parameter: bool,
    return_type: Vec<Field>,
    has_streamed_return: bool,
    exception_specification: Vec<EntityId>,
    is_idempotent: bool,
}
implement_encode_into_for_struct!(Operation, entity_info, parameters, has_streamed_parameter, return_type, has_streamed_return, exception_specification, is_idempotent);

#[derive(Clone, Debug)]
pub struct Enum {
    entity_info: EntityInfo,
    enumerators: Vec<Enumerator>,
    underlying: Option<Primitive>,
    is_compact: bool,
    is_unchecked: bool,
}
implement_encode_into_for_struct!(Enum, entity_info, enumerators, underlying, is_compact, is_unchecked);

#[derive(Clone, Debug)]
pub struct Enumerator {
    entity_info: EntityInfo,
    value: Discriminant,
    fields: Option<Vec<Field>>, // TODO we probably shouldn't distinguish
}
implement_encode_into_for_struct!(Enumerator, entity_info, value, fields);

#[derive(Clone, Debug)]
pub struct Discriminant {
    absolute_value: u64,
    is_positive: bool,
}
implement_encode_into_for_struct!(Discriminant, absolute_value, is_positive);

#[derive(Clone, Debug)]
pub struct CustomType {
    entity_info: EntityInfo,
}
implement_encode_into_for_struct!(CustomType, entity_info);

#[derive(Clone, Debug)]
pub struct TypeAlias {
    entity_info: EntityInfo,
    underlying: TypeRef,
}
implement_encode_into_for_struct!(TypeAlias, entity_info, underlying);

#[derive(Clone, Debug)]
#[repr(usize)]
pub enum AnonymousType {
    SequenceType(SequenceType),
    DictionaryType(DictionaryType),
    ResultType(ResultType),
}
impl EncodeInto<Slice2> for &AnonymousType {
    fn encode_into(self, encoder: &mut Encoder<impl OutputTarget>) -> Result<()> {
        encoder.encode(*<*const _>::from(self).cast::<usize>());  // TODO, we don't treat this as a varuint62 when we should...
        match self {
            AnonymousType::SequenceType(v) => encoder.encode(v),
            AnonymousType::DictionaryType(v) => encoder.encode(v),
            AnonymousType::ResultType(v) => encoder.encode(v),
        };
        encoder.encode_varint(TAG_END_MARKER);
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct SequenceType {
    element_type: TypeRef,
}
implement_encode_into_for_struct!(SequenceType, element_type);

#[derive(Clone, Debug)]
pub struct DictionaryType {
    key_type: TypeRef,
    value_type: TypeRef,
}
implement_encode_into_for_struct!(DictionaryType, key_type, value_type);

#[derive(Clone, Debug)]
pub struct ResultType {
    success_type: TypeRef,
    failure_type: TypeRef,
}
implement_encode_into_for_struct!(ResultType, success_type, failure_type);

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum Primitive {
    Bool, Int8, UInt8, Int16, UInt16, Int32, UInt32, VarInt32, VarUInt32, Int64, UInt64, VarInt62, VarUInt62, Float32, Float64, String, AnyClass,
}
impl EncodeInto<Slice2> for &Primitive {
    fn encode_into(self, encoder: &mut Encoder<impl OutputTarget>) -> Result<()> {
        encoder.encode(*self as u8);
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct TypeRef {
    value: TypeRefDefinition,
    is_optional: bool,
    attributes: Option<Vec<Attribute>>, // TAG(0)
}
implement_encode_into_for_struct!(TypeRef, value, is_optional, attributes);

#[derive(Clone, Debug)]
#[repr(usize)]
pub enum TypeRefDefinition {
    Definition(EntityId),
    Primitive(Primitive),
    Anonymous(usize), // TODO, we don't treat this as a varuint62 when we should...
}
impl EncodeInto<Slice2> for &TypeRefDefinition {
    fn encode_into(self, encoder: &mut Encoder<impl OutputTarget>) -> Result<()> {
        encoder.encode(*<*const _>::from(self).cast::<usize>());  // TODO, we don't treat this as a varuint62 when we should...
        match self {
            TypeRefDefinition::Definition(v) => encoder.encode(v),
            TypeRefDefinition::Primitive(v) => encoder.encode(v),
            TypeRefDefinition::Anonymous(v) => encoder.encode(v),
        };
        encoder.encode_varint(TAG_END_MARKER);
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct Attribute {
    directive: String,
    args: Vec<String>,
}
implement_encode_into_for_struct!(Attribute, directive, args);

fn main() {
    // Parse the command-line input.
    let slice_options = SliceOptions::parse();

    // Perform the compilation.
    let compilation_state = slicec::compile_from_options(&slice_options, |_| {}, |_| {});
    let CompilationState { ast, diagnostics, files } = compilation_state;

    // Process the diagnostics (filter out allowed lints, and update diagnostic levels as necessary).
    let updated_diagnostics = diagnostics.into_updated(&ast, &files, &slice_options);
    let totals = slicec::diagnostics::get_totals(&updated_diagnostics);

    // Print output to stdout.
    print!("Diagnostics: ");
    println!("{totals:?}");
    for diagnostic in updated_diagnostics {
        println!("{diagnostic:?}");
    }
    println!("{ast:?}");

    std::process::exit(i32::from(totals.1 != 0));
}
