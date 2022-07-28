// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::errors::*;
use crate::{implement_error_functions, implement_from_for_error_sub_kind};

#[derive(Debug)]
pub enum WarningKind {
    DocCommentIndicatesParam(String), // (param_name)
    DocCommentIndicatesReturn,
    DocCommentIndicatesThrow(String, String), // (kind, op_identifier)
}

implement_from_for_error_sub_kind!(WarningKind, ErrorKind::Warning);
implement_error_functions!(
    WarningKind,
    (
        WarningKind::DocCommentIndicatesParam,
        1000,
        format!(
            "doc comment has a param tag for '{}', but there is no parameter by that name",
            param_name
        ),
        param_name
    ),
    (
        WarningKind::DocCommentIndicatesReturn,
        1001,
        "void operation must not contain doc comment return tag"
    ),
    (
        WarningKind::DocCommentIndicatesThrow,
        1002,
        format!(
            "doc comment indicates that {} `{}` throws, however, only operations can throw",
            kind, op_identifier,
        ),
        kind,
        op_identifier
    )
);
