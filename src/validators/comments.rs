// Copyright (c) ZeroC, Inc.

use crate::diagnostics::{Diagnostic, DiagnosticReporter, Lint};
use crate::grammar::*;

pub fn validate_common_doc_comments(commentable: &dyn Commentable, reporter: &mut DiagnosticReporter) {
    // Only run this validation if a doc comment is present.
    let Some(comment) = commentable.comment() else { return };

    only_operations_have_parameters(comment, commentable, reporter);
    only_operations_can_return(comment, commentable, reporter);
    only_operations_can_throw(comment, commentable, reporter);
}

fn only_operations_have_parameters(comment: &DocComment, entity: &dyn Commentable, reporter: &mut DiagnosticReporter) {
    if !matches!(entity.concrete_entity(), Entities::Operation(_)) {
        for param_tag in &comment.params {
            report_only_operation_error(param_tag, entity, reporter);
        }
    }
}

fn only_operations_can_return(comment: &DocComment, entity: &dyn Commentable, reporter: &mut DiagnosticReporter) {
    if !matches!(entity.concrete_entity(), Entities::Operation(_)) {
        for returns_tag in &comment.returns {
            report_only_operation_error(returns_tag, entity, reporter);
        }
    }
}

fn only_operations_can_throw(comment: &DocComment, entity: &dyn Commentable, reporter: &mut DiagnosticReporter) {
    if !matches!(entity.concrete_entity(), Entities::Operation(_)) {
        for throws_tag in &comment.throws {
            report_only_operation_error(throws_tag, entity, reporter);
        }
    }
}

/// Helper function that emits an error if an operation only comment tag was used on something other than a comment.
fn report_only_operation_error(tag: &impl Symbol, entity: &dyn Commentable, reporter: &mut DiagnosticReporter) {
    let entity_kind = entity.kind();
    let note = format!(
        "'{identifier}' is {a} {entity_kind}",
        identifier = entity.identifier(),
        a = crate::utils::string_util::indefinite_article(entity_kind),
    );

    // All tag kinds are of the form "<kind> tag", so it's safe to unwrap. We only want the first word for the message.
    let tag_kind = tag.kind().split_once(' ').unwrap().0;
    let action_phrase = match tag_kind {
        "param" => "have parameters",
        "returns" => "return",
        "throws" => "throw",
        _ => unreachable!("'report_only_operation_error' was called with unsupported tag '{tag_kind}'"),
    };

    Diagnostic::new(Lint::IncorrectDocComment {
        message: format!("comment has a '{tag_kind}' tag, but only operations can {action_phrase}"),
    })
    .set_span(tag.span())
    .set_scope(entity.parser_scoped_identifier())
    .add_note(note, Some(entity.span()))
    .report(reporter);
}
