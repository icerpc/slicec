// Copyright (c) ZeroC, Inc.

//! This module contains the handwritten encoding code for the Slice-compiler definitions.
//! After rust code-gen has been implemented, this file will be deleted, and we will use generated definitions instead.

use slice_codec::slice2::Slice2;

use slice_codec::buffer::OutputTarget;
use slice_codec::encode_into::*;
use slice_codec::encoder::Encoder;
use slice_codec::Result;

/// TAG_END_MARKER must be encoded at the end of every non-compact type.
const TAG_END_MARKER: i32 = -1;

/// This macro implements `EncodeInto<Slice2>` for a Rust struct (which is mapped from a non-compact Slice struct).
/// It encodes all the struct's fields (in definition order), followed by `TAG_END_MARKER`.
///
/// It uses macro-function-syntax, and should be called like:
/// `implement_encode_into_for_struct!(struct_type_name, field1, field2, ...);`
macro_rules! implement_encode_into_for_struct {
    ($type_name:ty$(, $field_name:ident)*$(,)?) => {
        impl EncodeInto<Slice2> for &$type_name {
            fn encode_into(self, encoder: &mut Encoder<impl OutputTarget>) -> Result<()> {
                $(encoder.encode(&self.$field_name)?;)*
                encoder.encode_varint(TAG_END_MARKER)?;
                Ok(())
            }
        }
    }
}

// ================= //
// Hand-mapped types //
// ================= //

pub type EntityId = String;
pub type TypeId = String;
pub type Message = Vec<MessageComponent>;

#[derive(Clone, Debug)]
pub struct Attribute {
    pub directive: String,
    pub args: Vec<String>,
}
implement_encode_into_for_struct!(Attribute, directive, args);

#[derive(Clone, Debug)]
pub struct TypeRef {
    pub type_id: TypeId,
    pub is_optional: bool,
    pub type_attributes: Vec<Attribute>,
}
implement_encode_into_for_struct!(TypeRef, type_id, is_optional, type_attributes);

#[derive(Clone, Debug)]
pub struct EntityInfo {
    pub identifier: String,
    pub attributes: Vec<Attribute>,
    pub comment: Option<DocComment>,
}
impl EncodeInto<Slice2> for &EntityInfo {
    fn encode_into(self, encoder: &mut Encoder<impl OutputTarget>) -> Result<()> {
        encoder.encode(&self.identifier)?;
        encoder.encode(&self.attributes)?;
        // encoder.encode_tagged(0, &self.comment)?; TODO add doc-comments after adding tag encoding support.
        encoder.encode_varint(TAG_END_MARKER)?;
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct Module {
    pub identifier: String,
    pub attributes: Vec<Attribute>,
}
implement_encode_into_for_struct!(Module, identifier, attributes);

#[derive(Clone, Debug)]
pub struct Struct {
    pub entity_info: EntityInfo,
    pub is_compact: bool,
    pub fields: Vec<Field>,
}
implement_encode_into_for_struct!(Struct, entity_info, is_compact, fields);

#[derive(Clone, Debug)]
pub struct Field {
    pub entity_info: EntityInfo,
    pub tag: Option<i32>, // TODO: varint32 isn't a real type?
    pub data_type: TypeRef,
}
impl EncodeInto<Slice2> for &Field {
    fn encode_into(self, encoder: &mut Encoder<impl OutputTarget>) -> Result<()> {
        // Encode the bit-sequence. With only one optional, this is just a bool.
        encoder.encode(self.tag.is_some())?;

        // Encode the actual fields.
        encoder.encode(&self.entity_info)?;
        if let Some(tag_value) = self.tag {
            encoder.encode_varint(tag_value)?;
        }
        encoder.encode(&self.data_type)?;
        encoder.encode_varint(TAG_END_MARKER)?;
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct Interface {
    pub entity_info: EntityInfo,
    pub bases: Vec<EntityId>,
    pub operations: Vec<Operation>,
}
implement_encode_into_for_struct!(Interface, entity_info, bases, operations);

#[derive(Clone, Debug)]
pub struct Operation {
    pub entity_info: EntityInfo,
    pub is_idempotent: bool,
    pub parameters: Vec<Field>,
    pub has_streamed_parameter: bool,
    pub return_type: Vec<Field>,
    pub has_streamed_return: bool,
}
implement_encode_into_for_struct!(
    Operation,
    entity_info,
    is_idempotent,
    parameters,
    has_streamed_parameter,
    return_type,
    has_streamed_return,
);

#[derive(Clone, Debug)]
pub struct Enum {
    pub entity_info: EntityInfo,
    pub is_compact: bool,
    pub is_unchecked: bool,
    pub underlying: Option<TypeId>,
    pub enumerators: Vec<Enumerator>,
}
impl EncodeInto<Slice2> for &Enum {
    fn encode_into(self, encoder: &mut Encoder<impl OutputTarget>) -> Result<()> {
        // Encode the bit-sequence. With only one optional, this is just a bool.
        encoder.encode(self.underlying.is_some())?;

        // Encode the actual fields.
        encoder.encode(&self.entity_info)?;
        encoder.encode(self.is_compact)?;
        encoder.encode(self.is_unchecked)?;
        if let Some(underlying_value) = &self.underlying {
            encoder.encode(underlying_value)?;
        }
        encoder.encode(&self.enumerators)?;
        encoder.encode_varint(TAG_END_MARKER)?;
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct Enumerator {
    pub entity_info: EntityInfo,
    pub value: Discriminant,
    pub fields: Vec<Field>,
}
implement_encode_into_for_struct!(Enumerator, entity_info, value, fields);

#[derive(Clone, Debug)]
pub struct Discriminant {
    pub absolute_value: u64,
    pub is_positive: bool,
}
implement_encode_into_for_struct!(Discriminant, absolute_value, is_positive);

#[derive(Clone, Debug)]
pub struct CustomType {
    pub entity_info: EntityInfo,
}
implement_encode_into_for_struct!(CustomType, entity_info);

#[derive(Clone, Debug)]
pub struct TypeAlias {
    pub entity_info: EntityInfo,
    pub underlying_type: TypeRef, // Can never be optional.
}
implement_encode_into_for_struct!(TypeAlias, entity_info, underlying_type);

#[derive(Clone, Debug)]
pub struct SequenceType {
    pub element_type: TypeRef,
}
implement_encode_into_for_struct!(SequenceType, element_type);

#[derive(Clone, Debug)]
pub struct DictionaryType {
    pub key_type: TypeRef, // Can never be optional.
    pub value_type: TypeRef,
}
implement_encode_into_for_struct!(DictionaryType, key_type, value_type);

#[derive(Clone, Debug)]
pub struct ResultType {
    pub success_type: TypeRef,
    pub failure_type: TypeRef,
}
implement_encode_into_for_struct!(ResultType, success_type, failure_type);

#[derive(Clone, Debug)]
pub struct DocComment {
    pub overview: Message,
    pub see_tags: Vec<EntityId>,
}
implement_encode_into_for_struct!(DocComment, overview, see_tags);

#[repr(u8)]
#[derive(Clone, Debug)]
pub enum MessageComponent {
    Text(String) = 0,
    Link(EntityId) = 1,
}
impl EncodeInto<Slice2> for &MessageComponent {
    fn encode_into(self, encoder: &mut Encoder<impl OutputTarget>) -> Result<()> {
        // Write the discriminant value.
        // SAFETY: this cast is guaranteed to be safe because the enum is marked with `repr(u8)`, so it's safe to cast
        // it directly to a `u8`.
        unsafe {
            let discriminant = *<*const _>::from(self).cast::<u8>();
            encoder.encode_varint(discriminant)?;
        }

        // Encode the actual value.
        match self {
            MessageComponent::Text(v) => encoder.encode(v)?,
            MessageComponent::Link(v) => encoder.encode(v)?,
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct SliceFile {
    pub path: String,
    pub module_declaration: Module,
    pub attributes: Vec<Attribute>,
    pub contents: Vec<Symbol>,
}
implement_encode_into_for_struct!(SliceFile, path, module_declaration, attributes, contents);

#[derive(Clone, Debug)]
pub struct GeneratedFile {
    pub path: String,
    pub contents: String,
}
implement_encode_into_for_struct!(GeneratedFile, path, contents);

#[repr(u8)]
#[derive(Clone, Debug)]
pub enum Symbol {
    Interface(Interface) = 0,
    Enum(Enum) = 1,
    Struct(Struct) = 2,
    CustomType(CustomType) = 3,
    SequenceType(SequenceType) = 4,
    DictionaryType(DictionaryType) = 5,
    ResultType(ResultType) = 6, // TODO make result come before dictionary!
    TypeAlias(TypeAlias) = 7,
}
impl EncodeInto<Slice2> for &Symbol {
    fn encode_into(self, encoder: &mut Encoder<impl OutputTarget>) -> Result<()> {
        // Write the discriminant value.
        // SAFETY: this cast is guaranteed to be safe because the enum is marked with `repr(u8)`, which means we know
        // the first 'field' of this type's data layout must be a u8. This lets us read without offsetting the pointer.
        unsafe {
            let discriminant = *<*const _>::from(self).cast::<u8>();
            encoder.encode_varint(discriminant)?;
        }

        // Encode the actual value.
        match self {
            Symbol::Interface(v) => encoder.encode(v)?,
            Symbol::Enum(v) => encoder.encode(v)?,
            Symbol::Struct(v) => encoder.encode(v)?,
            Symbol::CustomType(v) => encoder.encode(v)?,
            Symbol::SequenceType(v) => encoder.encode(v)?,
            Symbol::DictionaryType(v) => encoder.encode(v)?,
            Symbol::ResultType(v) => encoder.encode(v)?,
            Symbol::TypeAlias(v) => encoder.encode(v)?,
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct Diagnostic {
    pub level: DiagnosticLevel,
    pub message: String,
    pub source: Option<String>,
}
impl EncodeInto<Slice2> for &Diagnostic {
    fn encode_into(self, encoder: &mut Encoder<impl OutputTarget>) -> Result<()> {
        // Encode the bit-sequence. With only one optional, this is just a bool.
        encoder.encode(self.source.is_some())?;

        // Encode the actual fields.
        encoder.encode(&self.level)?;
        encoder.encode(&self.message)?;
        if let Some(source_value) = &self.source {
            encoder.encode(source_value)?;
        }
        encoder.encode_varint(TAG_END_MARKER)?;
        Ok(())
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum DiagnosticLevel {
    Info,
    Warning,
    Error,
}
impl EncodeInto<Slice2> for &DiagnosticLevel {
    fn encode_into(self, encoder: &mut Encoder<impl OutputTarget>) -> Result<()> {
        encoder.encode(*self as u8)
    }
}
