// Copyright (c) ZeroC, Inc.

use crate::diagnostics::{Diagnostic, DiagnosticReporter, Warning};
use crate::grammar::*;

pub fn validate_common_doc_comments(commentable: &dyn Commentable, diagnostic_reporter: &mut DiagnosticReporter) {
    only_operations_can_throw(commentable, diagnostic_reporter);
}

fn only_operations_can_throw(commentable: &dyn Commentable, diagnostic_reporter: &mut DiagnosticReporter) {
    let supported_on = ["operation"];
    if let Some(comment) = commentable.comment() {
        if !supported_on.contains(&commentable.kind()) && !comment.throws.is_empty() {
            for throws_tag in &comment.throws {
                Diagnostic::new(Warning::IncorrectDocComment {
                    message: format!(
                        "doc comment indicates that {} '{}' throws, however, only operations can throw",
                        commentable.kind(),
                        commentable.identifier(),
                    ),
                })
                .set_span(throws_tag.span())
                .set_scope(commentable.parser_scoped_identifier())
                .report(diagnostic_reporter);
            }
        }
    }
}
