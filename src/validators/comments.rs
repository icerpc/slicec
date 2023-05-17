// Copyright (c) ZeroC, Inc.

use crate::diagnostics::{Diagnostic, DiagnosticReporter, Warning};
use crate::grammar::*;
use crate::validators::{ValidationChain, Validator};

pub fn comments_validators() -> ValidationChain {
    vec![
        Validator::DocComments(only_operations_can_throw),
        Validator::Operations(missing_parameter_comment),
        Validator::Operations(operation_missing_throws),
        Validator::Operations(non_empty_return_comment),
    ]
}

fn non_empty_return_comment(operation: &Operation, diagnostic_reporter: &mut DiagnosticReporter) {
    if let Some(comment) = operation.comment() {
        // Return doc comment exists but operation has no return members.
        // `DocComment.return_members` contains a list of descriptions of the return members.
        // example: @return A description of the return value.
        if !comment.returns.is_empty() && operation.return_members().is_empty() {
            for returns_tag in &comment.returns {
                Diagnostic::new(Warning::IncorrectDocComment {
                    message: "void operation must not contain doc comment return tag".to_owned(),
                })
                .set_span(returns_tag.span())
                .set_scope(operation.parser_scoped_identifier())
                .report(diagnostic_reporter);
            }
        }
    }
}

fn missing_parameter_comment(operation: &Operation, diagnostic_reporter: &mut DiagnosticReporter) {
    if let Some(comment) = operation.comment() {
        for param_tag in &comment.params {
            if !operation
                .parameters()
                .iter()
                .any(|param_def| param_def.identifier() == param_tag.identifier.value)
            {
                Diagnostic::new(Warning::IncorrectDocComment {
                    message: format!(
                        "doc comment has a param tag for '{}', but there is no parameter by that name",
                        &param_tag.identifier.value,
                    ),
                })
                .set_span(param_tag.span())
                .set_scope(operation.parser_scoped_identifier())
                .report(diagnostic_reporter);
            }
        }
    }
}

fn operation_missing_throws(operation: &Operation, diagnostic_reporter: &mut DiagnosticReporter) {
    if let Some(comment) = operation.comment() {
        if !&comment.throws.is_empty() && matches!(operation.throws, Throws::None) {
            Diagnostic::new(Warning::IncorrectDocComment {
                message: format!("operation '{}' does not throw anything", operation.identifier()),
            })
            .set_span(operation.span())
            .set_scope(operation.parser_scoped_identifier())
            .report(diagnostic_reporter);
        }
    }
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
