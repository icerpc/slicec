// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::errors::*;

#[derive(Debug, Clone)]
pub enum WarningKind {
    DocCommentIndicatesThrow { kind: String, op_identifier: String },
    DocCommentIndicatesReturn,
    DocCommentIndicatesParam { param_name: String },
}

impl WarningKind {
    pub fn get_error_code(&self) -> u32 {
        match self {
            WarningKind::DocCommentIndicatesThrow { .. } => 5,
            WarningKind::DocCommentIndicatesReturn { .. } => 6,
            WarningKind::DocCommentIndicatesParam { .. } => 7,
        }
    }

    pub fn get_description(&self) -> String {
        match self {
            WarningKind::DocCommentIndicatesThrow { kind, op_identifier } => format!(
                "doc comment indicates that {kind} `{op_identifier}` throws, however, only operations can throw",
                kind = kind,
                op_identifier = op_identifier,
            ),
            WarningKind::DocCommentIndicatesParam { param_name } => format!(
                "doc comment has a param tag for '{param_name}', but there is no parameter by that name",
                param_name = param_name,
            ),
            WarningKind::DocCommentIndicatesReturn => {
                "void operation must not contain doc comment return tag".to_string()
            }
        }
    }
}
