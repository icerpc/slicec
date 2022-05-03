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
impl DocComment {
    pub fn sanitize(&mut self) {
        self.overview = self.overview.trim().to_owned();
        self.see_also = self.see_also.iter().map(|s| s.trim().to_owned()).collect();
        self.params = self
            .params
            .iter()
            .map(|(s, t)| (s.to_owned(), t.trim().to_owned()))
            .collect();

        self.returns = self.returns.as_ref().map(|s| s.trim().to_owned());
        self.deprecate_reason = self.deprecate_reason.as_ref().map(|s| s.trim().to_owned());
        self.throws = self
            .throws
            .iter()
            .map(|(s, t)| (s.to_owned(), t.trim().to_owned()))
            .collect();
    }
}
