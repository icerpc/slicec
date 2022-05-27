// Copyright (c) ZeroC, Inc. All rights reserved.

use super::super::*;
use crate::slice_file::Location;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Identifier {
    pub value: String,
    pub raw_value: String,
    pub location: Location,
}

impl Identifier {
    pub fn new(value: String, location: Location) -> Identifier {
        Identifier {
            value: value.trim_start_matches('\\').to_owned(), // Remove possible leading '\'.
            raw_value: value,
            location,
        }
    }
}

implement_Element_for!(Identifier, "identifier");
implement_Symbol_for!(Identifier);
