// Copyright (c) ZeroC, Inc.

use super::super::*;
use crate::supported_encodings::SupportedEncodings;

#[derive(Debug)]
pub struct ResultType {
    pub ok_type: TypeRef,
    pub err_type: TypeRef,
}

impl Type for ResultType {
    fn type_string(&self) -> String {
        format!(
            "Result<{}, {}>",
            self.ok_type.type_string(),
            self.err_type.type_string(),
        )
    }

    fn fixed_wire_size(&self) -> Option<u32> {
        None
    }

    fn is_class_type(&self) -> bool {
        false
    }

    fn tag_format(&self) -> Option<TagFormat> {
        unreachable!("tag format was called on a Slice2 only type!")
    }

    fn supported_encodings(&self) -> SupportedEncodings {
        let mut encodings = self.ok_type.supported_encodings();
        encodings.intersect_with(&self.err_type.supported_encodings());
        encodings.disable(Encoding::Slice1);
        encodings
    }
}

implement_Element_for!(ResultType, "result");
