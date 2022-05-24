// Copyright (c) ZeroC, Inc. All rights reserved.

use super::super::*;
use crate::slice_file::Location;

#[derive(Clone, Debug)]
pub struct Attribute {
    pub prefix: Option<String>,
    pub directive: String,
    pub prefixed_directive: String,
    pub arguments: Vec<String>,
    pub location: Location,
}

impl Attribute {
    pub(crate) fn new(
        prefix: Option<String>,
        directive: String,
        arguments: Vec<String>,
        location: Location,
    ) -> Self {
        let prefixed_directive = prefix.clone().map_or(
            directive.clone(),                   // Default value if prefix == None
            |prefix| prefix + "::" + &directive, // Function to call if prefix == Some
        );
        Attribute { prefix, directive, prefixed_directive, arguments, location }
    }
}

implement_Element_for!(Attribute, "attribute");
implement_Symbol_for!(Attribute);
