// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::errors::*;
use crate::{implement_error_functions, implement_from_for_error_sub_kind};

#[derive(Debug)]
pub enum WarningKind {
    /// The user-supplied doc comment indicated that the operation should contain a parameter that it does not have
    ///
    /// # Fields
    ///
    /// * `identifier` - The name of the parameter from the user-supplied doc comment
    ExtraParameterInDocComment(String),

    /// The user-supplied doc comment indicated that the operation should return a value, but the operation does not
    ExtraReturnValueInDocComment,

    /// The user-supplied doc comment indicated that the entity should throw, but the entity does not support throwing
    ///
    /// # Fields
    ///
    /// * `kind` - The kind of that entity that was indicated to throw
    /// * `identifier` - The identifier of that entity that was indicated to throw
    ExtraThrowInDocComment(String, String),
}

implement_from_for_error_sub_kind!(WarningKind, ErrorKind::Warning);
implement_error_functions!(
    WarningKind,
    (
        WarningKind::ExtraParameterInDocComment,
        1000,
        format!("doc comment has a param tag for '{param_name}', but there is no parameter by that name"),
        param_name
    ),
    (
        WarningKind::ExtraReturnValueInDocComment,
        1001,
        "void operation must not contain doc comment return tag"
    ),
    (
        WarningKind::ExtraThrowInDocComment,
        1002,
        format!("doc comment indicates that {kind} `{identifier}` throws, however, only operations can throw"),
        kind,
        identifier
    )
);
