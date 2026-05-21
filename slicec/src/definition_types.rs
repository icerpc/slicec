// Copyright (c) ZeroC, Inc.

//! This module contains the handwritten encoding code for the Slice-compiler definitions.
//! After rust code-gen has been implemented, this file will be deleted, and we will use generated definitions instead.

#![allow(dead_code)]

use slice_codec::buffer::{InputSource, OutputTarget};
use slice_codec::decode_from::DecodeFrom;
use slice_codec::decoder::Decoder;
use slice_codec::encode_into::EncodeInto;
use slice_codec::encoder::Encoder;
use slice_codec::Result;

/// TAG_END_MARKER must be encoded at the end of every non-compact type.
const TAG_END_MARKER: i32 = -1;

/// This macro implements `EncodeInto` for a Rust struct (which is mapped from a non-compact Slice struct).
/// It encodes all the struct's fields (in definition order), followed by `TAG_END_MARKER`.
///
/// It uses macro-function-syntax, and should be called like:
/// `implement_encode_into_for_struct!(struct_type_name, field1, field2, ...);`
macro_rules! implement_encode_into_for_struct {
    ($type_name:ty$(, $field_name:ident)*$(,)?) => {
        impl EncodeInto for &$type_name {
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
impl EncodeInto for &EntityInfo {
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
impl EncodeInto for &Field {
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
pub struct ResultType {
    pub success_type: TypeRef,
    pub failure_type: TypeRef,
}
implement_encode_into_for_struct!(ResultType, success_type, failure_type);

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
impl EncodeInto for &MessageComponent {
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

pub struct Options(pub Vec<(String, String)>);
impl EncodeInto for Options {
    fn encode_into(self, encoder: &mut Encoder<impl OutputTarget>) -> Result<()> {
        encoder.encode_size(self.0.len())?;
        for (e1, e2) in &self.0 {
            encoder.encode(e1)?;
            encoder.encode(e2)?;
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

#[repr(u8)]
#[derive(Clone, Debug)]
pub enum Symbol {
    Interface(Interface) = 0,
    BasicEnum(BasicEnum) = 1,
    VariantEnum(VariantEnum) = 2,
    Struct(Struct) = 3,
    CustomType(CustomType) = 4,
    ResultType(ResultType) = 5,
    SequenceType(SequenceType) = 6,
    DictionaryType(DictionaryType) = 7,
    TypeAlias(TypeAlias) = 8,
}
impl EncodeInto for &Symbol {
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
            Symbol::ResultType(v) => encoder.encode(v)?,
            Symbol::SequenceType(v) => encoder.encode(v)?,
            Symbol::DictionaryType(v) => encoder.encode(v)?,
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
impl DecodeFrom for GeneratedFile {
    fn decode_from(decoder: &mut Decoder<impl InputSource>) -> Result<Self> {
        let path = decoder.decode()?;
        let contents = decoder.decode()?;

        decoder.skip_tagged_fields()?;

        Ok(GeneratedFile { path, contents })
    }
}

#[derive(Clone, Debug)]
pub struct Diagnostic {
    pub kind: DiagnosticKind,
    pub source: Option<String>,
    pub notes: Vec<DiagnosticNote>,
}
impl DecodeFrom for Diagnostic {
    fn decode_from(decoder: &mut Decoder<impl InputSource>) -> Result<Self> {
        // Decode the bit-sequence. With only one optional, this is just a bool.
        let has_source = decoder.decode::<bool>()?;

        // Decode the actual fields.
        let kind = decoder.decode()?;
        let source = has_source.then(|| decoder.decode()).transpose()?;
        let notes = decoder.decode()?;

        decoder.skip_tagged_fields()?;

        Ok(Diagnostic { kind, source, notes })
    }
}

#[derive(Clone, Debug)]
pub struct DiagnosticNote {
    pub message: String,
    pub source: Option<String>,
}
impl DecodeFrom for DiagnosticNote {
    fn decode_from(decoder: &mut Decoder<impl InputSource>) -> Result<Self> {
        // Decode the bit-sequence. With only one optional, this is just a bool.
        let has_source = decoder.decode::<bool>()?;

        // Decode the actual fields.
        let message = decoder.decode()?;
        let source = has_source.then(|| decoder.decode()).transpose()?;

        decoder.skip_tagged_fields()?;

        Ok(DiagnosticNote { message, source })
    }
}

#[repr(usize)]
#[derive(Clone, Debug)]
pub enum DiagnosticKind {
    Info {
        message: String,
    } = 0,
    Warning {
        message: String,
    } = 1,
    Error {
        message: String,
    } = 2,
    InvalidAttribute {
        directive: String,
    } = 3,
    UnknownAttribute {
        directive: String,
    } = 4,
    MissingRequiredAttribute {
        expected_attribute: String,
    } = 5,
    AttributeIsNotRepeatable {
        directive: String,
    } = 6,
    InvalidAttributeArgument {
        directive: String,
        argument: String,
    } = 7,
    IncorrectAttributeArgumentCount {
        directive: String,
        min_expected: u8,
        max_expected: u8,
        actual_count: u8,
    } = 8,

    Unknown {
        discriminant: usize,
        fields_payload: Vec<u8>,
    },
}
impl DecodeFrom for DiagnosticKind {
    fn decode_from(decoder: &mut Decoder<impl InputSource>) -> Result<Self> {
        let value: usize = decoder.decode_varint()?;
        let variant = match value {
            0 => Self::Info {
                message: decoder.decode()?,
            },
            1 => Self::Warning {
                message: decoder.decode()?,
            },
            2 => Self::Error {
                message: decoder.decode()?,
            },
            3 => Self::InvalidAttribute {
                directive: decoder.decode()?,
            },
            4 => Self::UnknownAttribute {
                directive: decoder.decode()?,
            },
            5 => Self::MissingRequiredAttribute {
                expected_attribute: decoder.decode()?,
            },
            6 => Self::AttributeIsNotRepeatable {
                directive: decoder.decode()?,
            },
            7 => Self::InvalidAttributeArgument {
                directive: decoder.decode()?,
                argument: decoder.decode()?,
            },
            8 => Self::IncorrectAttributeArgumentCount {
                directive: decoder.decode()?,
                min_expected: decoder.decode()?,
                max_expected: decoder.decode()?,
                actual_count: decoder.decode()?,
            },

            n => {
                // Read the unknown variant's fields payload.
                // We don't know what they are, but we at least know the length of the payload.
                let payload_size = decoder.decode_size()?;
                let mut fields_payload = vec![0; payload_size];
                decoder.read_bytes_into_exact(&mut fields_payload)?;

                Self::Unknown {
                    discriminant: n,
                    fields_payload,
                }
            }
        };

        decoder.skip_tagged_fields()?;
        Ok(variant)
    }
}
