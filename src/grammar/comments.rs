// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::slice_file::Span;

// TODO improve this to track the span of individual doc comment fields, so we can check for
// comment validity: EX: making sure 'params' match the operation's actual parameters, etc.
#[derive(Clone, Debug)] // TODO this shouldn't be cloned. We need to change CsharpComment.
pub struct DocComment {
    pub overview: String,
    pub see_also: Vec<String>,
    pub params: Vec<(String, String)>,
    pub returns: Option<String>,
    pub throws: Vec<(String, String)>,
    pub span: Span,
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
        self.throws = self
            .throws
            .iter()
            .map(|(s, t)| (s.to_owned(), t.trim().to_owned()))
            .collect();
    }
}

pub fn find_inline_tags(comment: &str) -> Vec<(&str, &str)> {
    let mut tags = Vec::new();

    let mut section = comment;

    while let Some(pos) = section.find('{') {
        // Search for the closing bracket. If we don't find one just exist the loop.
        match section[pos..].find('}') {
            Some(end) => {
                let tag = &section[pos + 1..pos + end];
                let tag_parts = tag
                    .split(char::is_whitespace)
                    .filter(|s| !s.trim().is_empty())
                    .collect::<Vec<&str>>();

                // Only match tags with two parts. We'll verify the tag type and value later.
                if tag_parts.len() == 2 {
                    tags.push((tag_parts[0], tag_parts[1]));
                }

                // The next section is the part of the comment after the closing bracket.
                section = &section[pos + end + 1..];
            }
            None => break,
        }
    }
    tags
}
