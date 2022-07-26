// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::errors::*;
use crate::{implement_from_for_error_sub_kind, implement_kind_for_enumerator};

pub enum WarningKind {
    DocCommentIndicatesParam(String), // (param_name)
    DocCommentIndicatesReturn,
    DocCommentIndicatesThrow(String, String), // (kind, op_identifier)
}

implement_from_for_error_sub_kind!(WarningKind, ErrorKind::Warning);
implement_kind_for_enumerator!(
    WarningKind,
    (
        WarningKind::DocCommentIndicatesParam,
        1000,
        format!(
            "doc comment has a param tag for '{}', but there is no parameter by that name",
            param_name
        )
        .as_str(),
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
        )
        .as_str(),
        kind,
        op_identifier
    )
);
