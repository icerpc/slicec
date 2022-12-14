// Copyright (c) ZeroC, Inc. All rights reserved.

use super::super::*;
use crate::supported_encodings::SupportedEncodings;

#[derive(Debug)]
pub struct Sequence {
    pub element_type: TypeRef,
}

impl Sequence {
    pub fn has_fixed_size_numeric_elements(&self) -> bool {
        if self.element_type.is_optional {
            false
        } else {
            let mut definition = self.element_type.concrete_type();

            // If the elements are enums with an underlying type, check the underlying type instead.
            if let Types::Enum(enum_def) = definition {
                if let Some(underlying) = &enum_def.underlying {
                    definition = underlying.concrete_type();
                }
            }

            if let Types::Primitive(primitive) = definition {
                primitive.is_numeric_or_bool() && primitive.fixed_wire_size().is_some()
            } else {
                false
            }
        }
    }
}

impl Type for Sequence {
    fn type_string(&self) -> String {
        format!("sequence<{}>", self.element_type.type_string())
    }

    fn fixed_wire_size(&self) -> Option<u32> {
        None
    }

    fn is_class_type(&self) -> bool {
        false
    }

    fn tag_format(&self) -> Option<TagFormat> {
        match self.element_type.fixed_wire_size() {
            Some(1) => Some(TagFormat::OptimizedVSize),
            Some(_) => Some(TagFormat::VSize),
            None => Some(TagFormat::FSize),
        }
    }

    fn supported_encodings(&self) -> SupportedEncodings {
        self.element_type.supported_encodings()
    }
}

implement_Element_for!(Sequence, "sequence");
