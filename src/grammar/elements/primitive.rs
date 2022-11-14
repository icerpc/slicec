// Copyright (c) ZeroC, Inc. All rights reserved.

use super::super::*;
use crate::supported_encodings::SupportedEncodings;

#[derive(Debug)]
pub enum Primitive {
    Bool,
    Int8,
    UInt8,
    Int16,
    UInt16,
    Int32,
    UInt32,
    VarInt32,
    VarUInt32,
    Int64,
    UInt64,
    VarInt62,
    VarUInt62,
    Float32,
    Float64,
    String,
    AnyClass,
}

impl Primitive {
    pub fn is_numeric(&self) -> bool {
        matches!(
            self,
            Self::Int8
                | Self::UInt8
                | Self::Int16
                | Self::UInt16
                | Self::Int32
                | Self::UInt32
                | Self::VarInt32
                | Self::VarUInt32
                | Self::Int64
                | Self::UInt64
                | Self::VarInt62
                | Self::VarUInt62
                | Self::Float32
                | Self::Float64
        )
    }

    pub fn is_integral(&self) -> bool {
        matches!(
            self,
            Self::Int8
                | Self::UInt8
                | Self::Int16
                | Self::UInt16
                | Self::Int32
                | Self::UInt32
                | Self::VarInt32
                | Self::VarUInt32
                | Self::Int64
                | Self::UInt64
                | Self::VarInt62
                | Self::VarUInt62
        )
    }

    pub fn is_unsigned_numeric(&self) -> bool {
        matches!(
            self,
            Self::UInt8 | Self::UInt16 | Self::UInt32 | Self::VarUInt32 | Self::UInt64 | Self::VarUInt62
        )
    }

    pub fn is_numeric_or_bool(&self) -> bool {
        self.is_numeric() || matches!(self, Self::Bool)
    }

    pub fn numeric_bounds(&self) -> Option<(i128, i128)> {
        static VARINT62_MIN: i128 = -2_305_843_009_213_693_952; // -2^61
        static VARINT62_MAX: i128 = 2_305_843_009_213_693_951; // 2^61 - 1
        static VARUINT62_MAX: i128 = 4_611_686_018_427_387_903; // 2^62 - 1

        match self {
            Self::Int8 => Some((i8::MIN as i128, i8::MAX as i128)),
            Self::UInt8 => Some((0, u8::MAX as i128)),
            Self::Int16 => Some((i16::MIN as i128, i16::MAX as i128)),
            Self::UInt16 => Some((0, u16::MAX as i128)),
            Self::Int32 => Some((i32::MIN as i128, i32::MAX as i128)),
            Self::UInt32 => Some((0, u32::MAX as i128)),
            Self::VarInt32 => Some((i32::MIN as i128, i32::MAX as i128)),
            Self::VarUInt32 => Some((0, u32::MAX as i128)),
            Self::Int64 => Some((i64::MIN as i128, i64::MAX as i128)),
            Self::UInt64 => Some((0, u64::MAX as i128)),
            Self::VarInt62 => Some((VARINT62_MIN, VARINT62_MAX)),
            Self::VarUInt62 => Some((0, VARUINT62_MAX)),
            _ => None,
        }
    }
}

impl Type for Primitive {
    fn type_string(&self) -> String {
        self.kind().to_owned()
    }

    fn is_fixed_size(&self) -> bool {
        matches!(
            self,
            Self::Bool
                | Self::Int8
                | Self::UInt8
                | Self::Int16
                | Self::UInt16
                | Self::Int32
                | Self::UInt32
                | Self::Int64
                | Self::UInt64
                | Self::Float32
                | Self::Float64
        )
    }

    fn min_wire_size(&self) -> u32 {
        match self {
            Self::Bool => 1,
            Self::Int8 => 1,
            Self::UInt8 => 1,
            Self::Int16 => 2,
            Self::UInt16 => 2,
            Self::Int32 => 4,
            Self::UInt32 => 4,
            Self::VarInt32 => 1,
            Self::VarUInt32 => 1,
            Self::Int64 => 8,
            Self::UInt64 => 8,
            Self::VarInt62 => 1,
            Self::VarUInt62 => 1,
            Self::Float32 => 4,
            Self::Float64 => 8,
            Self::String => 1,   // At least 1 byte for the empty string.
            Self::AnyClass => 1, // At least 1 byte to encode an index (instead of an instance).
        }
    }

    fn uses_classes(&self) -> bool {
        matches!(self, Self::AnyClass)
    }

    fn is_class_type(&self) -> bool {
        matches!(self, Self::AnyClass)
    }

    fn tag_format(&self) -> Option<TagFormat> {
        match self {
            Self::Bool => Some(TagFormat::F1),
            Self::Int8 => None,
            Self::UInt8 => Some(TagFormat::F1),
            Self::Int16 => Some(TagFormat::F2),
            Self::UInt16 => None,
            Self::Int32 => Some(TagFormat::F4),
            Self::UInt32 => None,
            Self::VarInt32 => None,
            Self::VarUInt32 => None,
            Self::Int64 => Some(TagFormat::F8),
            Self::UInt64 => None,
            Self::VarInt62 => None,
            Self::VarUInt62 => None,
            Self::Float32 => Some(TagFormat::F4),
            Self::Float64 => Some(TagFormat::F8),
            Self::String => Some(TagFormat::OptimizedVSize),
            Self::AnyClass => Some(TagFormat::Class),
        }
    }

    fn supported_encodings(&self) -> SupportedEncodings {
        SupportedEncodings::new(match self {
            Self::Bool => vec![Encoding::Slice1, Encoding::Slice2],
            Self::Int8 => vec![Encoding::Slice2],
            Self::UInt8 => vec![Encoding::Slice1, Encoding::Slice2],
            Self::Int16 => vec![Encoding::Slice1, Encoding::Slice2],
            Self::UInt16 => vec![Encoding::Slice2],
            Self::Int32 => vec![Encoding::Slice1, Encoding::Slice2],
            Self::UInt32 => vec![Encoding::Slice2],
            Self::VarInt32 => vec![Encoding::Slice2],
            Self::VarUInt32 => vec![Encoding::Slice2],
            Self::Int64 => vec![Encoding::Slice1, Encoding::Slice2],
            Self::UInt64 => vec![Encoding::Slice2],
            Self::VarInt62 => vec![Encoding::Slice2],
            Self::VarUInt62 => vec![Encoding::Slice2],
            Self::Float32 => vec![Encoding::Slice1, Encoding::Slice2],
            Self::Float64 => vec![Encoding::Slice1, Encoding::Slice2],
            Self::String => vec![Encoding::Slice1, Encoding::Slice2],
            Self::AnyClass => vec![Encoding::Slice1],
        })
    }
}

impl Element for Primitive {
    fn kind(&self) -> &'static str {
        match self {
            Self::Bool => "bool",
            Self::Int8 => "int8",
            Self::UInt8 => "uint8",
            Self::Int16 => "int16",
            Self::UInt16 => "uint16",
            Self::Int32 => "int32",
            Self::UInt32 => "uint32",
            Self::VarInt32 => "varint32",
            Self::VarUInt32 => "varuint32",
            Self::Int64 => "int64",
            Self::UInt64 => "uint64",
            Self::VarInt62 => "varint62",
            Self::VarUInt62 => "varuint62",
            Self::Float32 => "float32",
            Self::Float64 => "float64",
            Self::String => "string",
            Self::AnyClass => "AnyClass",
        }
    }
}
