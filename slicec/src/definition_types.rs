// Copyright (c) ZeroC, Inc.

//! This module contains the handwritten encoding code for the Slice-compiler definitions.
//! After rust code-gen has been implemented, this file will be deleted, and we will use generated definitions instead.

use slice_codec::slice2::Slice2;

use slice_codec::buffer::{InputSource, OutputTarget};
use slice_codec::decode_from::DecodeFrom;
use slice_codec::decoder::Decoder;
use slice_codec::encode_into::EncodeInto;
use slice_codec::encoder::Encoder;
use slice_codec::{InvalidDataErrorKind, Result};

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
        // Encode the bit-sequence. With only one optional, this is just a bool.
        encoder.encode(self.comment.is_some())?;

        // Encode the actual fields.
        encoder.encode(&self.identifier)?;
        encoder.encode(&self.attributes)?;
        if let Some(comment_value) = &self.comment {
            encoder.encode(comment_value)?;
        }
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
pub struct BasicEnum {
    pub entity_info: EntityInfo,
    pub is_unchecked: bool,
    pub underlying: TypeId,
    pub enumerators: Vec<Enumerator>,
}
implement_encode_into_for_struct!(BasicEnum, entity_info, is_unchecked, underlying, enumerators);

#[derive(Clone, Debug)]
pub struct Enumerator {
    pub entity_info: EntityInfo,
    pub absolute_value: u64,
    pub has_negative_value: bool,
}
implement_encode_into_for_struct!(Enumerator, entity_info, absolute_value, has_negative_value);

#[derive(Clone, Debug)]
pub struct VariantEnum {
    pub entity_info: EntityInfo,
    pub is_compact: bool,
    pub is_unchecked: bool,
    pub variants: Vec<Variant>,
}
implement_encode_into_for_struct!(VariantEnum, entity_info, is_compact, is_unchecked, variants);

#[derive(Clone, Debug)]
pub struct Variant {
    pub entity_info: EntityInfo,
    pub discriminant: i32,
    pub fields: Vec<Field>,
}
implement_encode_into_for_struct!(Variant, entity_info, discriminant, fields);

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

        encoder.encode_varint(TAG_END_MARKER)?;
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

#[repr(u8)]
#[derive(Clone, Debug)]
pub enum Symbol {
    Interface(Interface) = 0,
    BasicEnum(BasicEnum) = 1,
    VariantEnum(VariantEnum) = 2,
    Struct(Struct) = 3,
    CustomType(CustomType) = 4,
    SequenceType(SequenceType) = 5,
    DictionaryType(DictionaryType) = 6,
    ResultType(ResultType) = 7, // TODO make result come before dictionary!
    TypeAlias(TypeAlias) = 8,
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
            Symbol::BasicEnum(v) => encoder.encode(v)?,
            Symbol::VariantEnum(v) => encoder.encode(v)?,
            Symbol::Struct(v) => encoder.encode(v)?,
            Symbol::CustomType(v) => encoder.encode(v)?,
            Symbol::SequenceType(v) => encoder.encode(v)?,
            Symbol::DictionaryType(v) => encoder.encode(v)?,
            Symbol::ResultType(v) => encoder.encode(v)?,
            Symbol::TypeAlias(v) => encoder.encode(v)?,
        }

        encoder.encode_varint(TAG_END_MARKER)?;
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct GeneratedFile {
    pub path: String,
    pub contents: String,
}
impl DecodeFrom<Slice2> for GeneratedFile {
    fn decode_from(decoder: &mut Decoder<impl InputSource, Slice2>) -> Result<Self> {
        let path = decoder.decode()?;
        let contents = decoder.decode()?;

        decoder.skip_tagged_fields()?;

        Ok(GeneratedFile { path, contents })
    }
}

#[derive(Clone, Debug)]
pub struct Diagnostic {
    pub level: DiagnosticLevel,
    pub message: String,
    pub source: Option<String>,
}
impl DecodeFrom<Slice2> for Diagnostic {
    fn decode_from(decoder: &mut Decoder<impl InputSource, Slice2>) -> Result<Self> {
        // Decode the bit-sequence. With only one optional, this is just a bool.
        let has_source = decoder.decode::<bool>()?;

        // Decode the actual fields.
        let level = decoder.decode()?;
        let message = decoder.decode()?;
        let source = has_source.then(|| decoder.decode()).transpose()?;

        decoder.skip_tagged_fields()?;

        Ok(Diagnostic { level, message, source })
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum DiagnosticLevel {
    Info = 0,
    Warning = 1,
    Error = 2,
}
impl DecodeFrom<Slice2> for DiagnosticLevel {
    fn decode_from(decoder: &mut Decoder<impl InputSource, Slice2>) -> Result<Self> {
        let value = decoder.decode::<u8>()?;
        match value {
            0 => Ok(DiagnosticLevel::Info),
            1 => Ok(DiagnosticLevel::Warning),
            2 => Ok(DiagnosticLevel::Error),
            _ => {
                let error = InvalidDataErrorKind::IllegalValue {
                    desc: "DiagnosticLevel",
                    value: Some(value.into()),
                };
                Err(error.into())
            }
        }
    }
}
