// Copyright (c) ZeroC, Inc. All rights reserved.

use super::super::*;
use crate::slice_file::Span;

#[derive(Clone, Debug)]
pub struct Attribute {
    pub prefix: Option<String>,
    pub directive: String,
    pub prefixed_directive: String,
    pub arguments: Vec<String>,
    pub span: Span,
}

impl Attribute {
    pub(crate) fn new(prefix: Option<String>, directive: String, arguments: Vec<String>, span: Span) -> Self {
        let prefixed_directive = prefix.clone().map_or(
            directive.clone(),                   // Default value if prefix == None
            |prefix| prefix + "::" + &directive, // Function to call if prefix == Some
        );
        Attribute {
            prefix,
            directive,
            prefixed_directive,
            arguments,
            span,
        }
    }
}

implement_Element_for!(Attribute, "attribute");
implement_Symbol_for!(Attribute);
