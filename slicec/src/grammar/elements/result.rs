// Copyright (c) ZeroC, Inc.

use super::super::*;

#[derive(Debug)]
pub struct ResultType {
    pub success_type: TypeRef,
    pub failure_type: TypeRef,
}

impl Type for ResultType {
    fn type_string(&self) -> String {
        format!(
            "Result<{}, {}>",
            self.success_type.type_string(),
            self.failure_type.type_string(),
        )
    }
}

implement_Element_for!(ResultType, "result");
