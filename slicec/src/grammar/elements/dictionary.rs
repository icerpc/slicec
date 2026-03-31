// Copyright (c) ZeroC, Inc.

use super::super::*;

#[derive(Debug)]
pub struct Dictionary {
    pub key_type: TypeRef,
    pub value_type: TypeRef,
}

impl Type for Dictionary {
    fn type_string(&self) -> String {
        format!(
            "Dictionary<{}, {}>",
            self.key_type.type_string(),
            self.value_type.type_string(),
        )
    }

    fn fixed_wire_size(&self) -> Option<u32> {
        None
    }
}

implement_Element_for!(Dictionary, "dictionary");
