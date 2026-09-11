// Copyright (c) ZeroC, Inc.

use crate::diagnostics::{Diagnostic, Diagnostics, Lint};
use crate::grammar::*;
use crate::slice_file::Span;

pub fn validate_common_doc_comments(commentable: &dyn Commentable, diagnostics: &mut Diagnostics) {
    // Only run this validation if a doc comment is present.
    let Some(comment) = commentable.comment() else { return };

    only_operations_have_parameters(comment, commentable, diagnostics);
    only_operations_can_return(comment, commentable, diagnostics);
}

fn only_operations_have_parameters(comment: &DocComment, entity: &dyn Commentable, diagnostics: &mut Diagnostics) {
    let concrete_entity = entity.concrete_entity();
    if !matches!(concrete_entity, Entities::Operation(_) | Entities::Enumerator(_)) {
        for param_tag in &comment.params {
            report_only_operation_error(
                "comment has a 'param' tag, but only operations and enumerators have parameters".to_owned(),
                param_tag,
                param_tag.message.span(),
                entity,
                diagnostics,
            );
        }
    }
}

fn only_operations_can_return(comment: &DocComment, entity: &dyn Commentable, diagnostics: &mut Diagnostics) {
    if !matches!(entity.concrete_entity(), Entities::Operation(_)) {
        for returns_tag in &comment.returns {
            report_only_operation_error(
                "comment has a 'returns' tag, but only operations have return types".to_owned(),
                returns_tag,
                returns_tag.message.span(),
                entity,
                diagnostics,
            );
        }
    }
}

/// Helper function that reports an error if an operation-only comment-tag was used on something other than a comment.
fn report_only_operation_error(
    message: String,
    tag: &impl Symbol,
    message_span: &Span,
    entity: &dyn Commentable,
    diagnostics: &mut Diagnostics,
) {
    let entity_kind = entity.kind();
    let note = format!(
        "'{identifier}' is {a} {entity_kind}",
        identifier = entity.identifier(),
        a = crate::utils::string_util::indefinite_article(entity_kind),
    );

    Diagnostic::from_lint(Lint::IncorrectDocComment { message })
        .set_span(&(tag.span() + message_span))
        .set_scope(entity.parser_scoped_identifier())
        .add_note(note, Some(entity.span()))
        .push_into(diagnostics);
}
