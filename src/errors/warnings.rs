// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::errors::*;
use crate::{implement_error_functions, implement_from_for_error_sub_kind};

#[derive(Debug)]
pub enum WarningKind {
    /// The user-supplied doc comment indicated that the operation should contain a parameter that it does not have
    ///
    /// # Fields
    ///
    /// * `parameter_name` - The name of the parameter from the user-supplied doc comment
    DocCommentIndicatesParam(String),

    /// The user-supplied doc comment indicated that the operation should return a value, but the operation does not
    DocCommentIndicatesReturn,

    /// The user-supplied doc comment indicated that the entity should throw, but the entity does not support throwing
    ///
    /// # Fields
    ///
    /// * `kind` - The kind of that entity that was indicated to throw
    /// * `identifier` - The identifier of that entity that was indicated to throw
    DocCommentIndicatesThrow(String, String),
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
