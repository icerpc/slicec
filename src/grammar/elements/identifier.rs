// Copyright (c) ZeroC, Inc. All rights reserved.

use super::super::*;
use crate::slice_file::Span;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Identifier {
    pub value: String,
    pub raw_value: String,
    pub span: Span,
}

impl Identifier {
    pub fn new(value: String, span: Span) -> Identifier {
        Identifier {
            value: value.trim_start_matches('\\').to_owned(), // Remove possible leading '\'.
            raw_value: value,
            span,
        }
    }
}

implement_Element_for!(Identifier, "identifier");
implement_Locatable_for!(Identifier);
implement_Symbol_for!(Identifier);
