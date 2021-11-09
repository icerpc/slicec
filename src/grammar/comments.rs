// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::slice_file::Location;

// TODO improve this to track the location of individual doc comment fields, so we can check for
// comment validity: EX: making sure 'params' match the operation's actual parameters, etc.
#[derive(Clone, Debug)] // TODO this shouldn't be cloned. We need to change CsharpComment.
pub struct DocComment {
    pub overview: String,
    pub see_also: Vec<String>,
    pub params: Vec<(String, String)>,
    pub returns: Option<String>,
    pub throws: Vec<(String, String)>,
    pub deprecate_reason: Option<String>,
    pub location: Location,
}
