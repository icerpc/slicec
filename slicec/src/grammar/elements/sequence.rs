// Copyright (c) ZeroC, Inc.

use super::super::*;

#[derive(Debug)]
pub struct Sequence {
    pub element_type: TypeRef,
}

impl Sequence {
    pub fn has_fixed_size_primitive_elements(&self) -> bool {
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
                primitive.fixed_wire_size().is_some()
            } else {
                false
            }
        }
    }
}

impl Type for Sequence {
    fn type_string(&self) -> String {
        format!("Sequence<{}>", self.element_type.type_string())
    }

    fn fixed_wire_size(&self) -> Option<u32> {
        None
    }
}

implement_Element_for!(Sequence, "sequence");
