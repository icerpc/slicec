// Copyright (c) ZeroC, Inc.

use super::super::*;

#[derive(Debug)]
pub struct Sequence {
    pub element_type: TypeRef,
}

impl Type for Sequence {
    fn type_string(&self) -> String {
        format!("Sequence<{}>", self.element_type.type_string())
    }
}

implement_Element_for!(Sequence, "sequence");
