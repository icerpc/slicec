// Copyright (c) ZeroC, Inc.

use crate::diagnostics::{Diagnostic, DiagnosticReporter, Lint};
use crate::grammar::*;

pub fn validate_operation(operation: &Operation, reporter: &mut DiagnosticReporter) {
    if let Some(comment) = operation.comment() {
        validate_param_tags(comment, operation, reporter);
        validate_returns_tags(comment, operation, reporter);
        validate_throws_tags(comment, operation, reporter);
    }
}

fn validate_param_tags(comment: &DocComment, operation: &Operation, reporter: &mut DiagnosticReporter) {
    let parameters: Vec<_> = operation.parameters().iter().map(|p| p.identifier()).collect();

    for param_tag in &comment.params {
        let tag_identifier = param_tag.identifier.value.as_str();
        if !parameters.contains(&tag_identifier) {
            Diagnostic::new(Lint::IncorrectDocComment {
                message: format!(
                    "comment has a 'param' tag for '{tag_identifier}', but operation '{}' has no parameter with that name",
                    operation.identifier(),
                ),
            })
            .set_span(param_tag.span())
            .set_scope(operation.parser_scoped_identifier())
            .report(reporter);
        }
    }
}

fn validate_returns_tags(comment: &DocComment, operation: &Operation, reporter: &mut DiagnosticReporter) {
    let returns_tags = &comment.returns;
    match operation.return_members().as_slice() {
        // If the operation doesn't return anything, but its doc comment has 'returns' tags, emit an error.
        [] => validate_returns_tags_for_operation_with_no_return_type(returns_tags, operation, reporter),

        // If the operation returns a single type, ensure that its 'returns' tag doesn't have an identifier.
        [_] => validate_returns_tags_for_operation_with_single_return(returns_tags, operation, reporter),

        // If the operation returns a tuple, ensure its returns tags use identifiers matching the tuple's.
        tuple => validate_returns_tags_for_operation_with_return_tuple(returns_tags, operation, tuple, reporter),
    }
}

fn validate_returns_tags_for_operation_with_no_return_type(
    returns_tags: &[ReturnsTag],
    operation: &Operation,
    reporter: &mut DiagnosticReporter,
) {
    for returns_tag in returns_tags {
        Diagnostic::new(Lint::IncorrectDocComment {
            message: format!(
                "comment has a 'returns' tag, but operation '{}' does not return anything",
                operation.identifier(),
            ),
        })
        .set_span(returns_tag.span())
        .set_scope(operation.parser_scoped_identifier())
        .report(reporter);
    }
}

fn validate_returns_tags_for_operation_with_single_return(
    returns_tags: &[ReturnsTag],
    operation: &Operation,
    reporter: &mut DiagnosticReporter,
) {
    for returns_tag in returns_tags {
        if let Some(tag_identifier) = &returns_tag.identifier {
            Diagnostic::new(Lint::IncorrectDocComment {
                message: format!(
                    "comment has a 'returns' tag for '{}', but operation '{}' doesn't return anything with that name",
                    &tag_identifier.value,
                    operation.identifier(),
                ),
            })
            .set_span(returns_tag.span())
            .set_scope(operation.parser_scoped_identifier())
            .add_note(
                format!("operation '{}' returns a single unnamed type", operation.identifier()),
                Some(operation.span()),
            )
            .add_note("try removing the identifier from your comment: \"@returns: ...\"", None)
            .report(reporter);
        }
    }
}

fn validate_returns_tags_for_operation_with_return_tuple(
    returns_tags: &[ReturnsTag],
    operation: &Operation,
    return_tuple: &[&Parameter],
    reporter: &mut DiagnosticReporter,
) {
    let return_members: Vec<_> = return_tuple.iter().map(|p| p.identifier()).collect();

    for returns_tag in returns_tags {
        if let Some(tag_identifier) = &returns_tag.identifier {
            let tag_identifier = tag_identifier.value.as_str();
            if !return_members.contains(&tag_identifier) {
                Diagnostic::new(Lint::IncorrectDocComment {
                    message: format!(
                        "comment has a 'returns' tag for '{tag_identifier}', but operation '{}' doesn't return anything with that name",
                        operation.identifier(),
                    ),
                })
                .set_span(returns_tag.span())
                .set_scope(operation.parser_scoped_identifier())
                .report(reporter);
            }
        }
    }
}

fn validate_throws_tags(comment: &DocComment, operation: &Operation, reporter: &mut DiagnosticReporter) {
    let throws_tags = &comment.throws;

    match &operation.throws {
        // If the operation doesn't throw, but its doc comment has 'throws' tags, emit an error.
        Throws::None => validate_throws_tags_for_operation_with_no_throws_clause(throws_tags, operation, reporter),

        // If the operation throws a specific exception, ensure that its 'throws' tag agrees with it.
        Throws::Specific(exception_ref) => validate_throws_tags_for_operation_that_throws_a_specific_exception(
            throws_tags,
            operation,
            exception_ref,
            reporter,
        ),

        // We perform no validation if the operation throws 'AnyException'.
        Throws::AnyException => {}
    }
}

fn validate_throws_tags_for_operation_with_no_throws_clause(
    throws_tags: &[ThrowsTag],
    operation: &Operation,
    reporter: &mut DiagnosticReporter,
) {
    for throws_tag in throws_tags {
        Diagnostic::new(Lint::IncorrectDocComment {
            message: format!(
                "comment has a 'throws' tag, but operation '{}' does not throw anything",
                operation.identifier(),
            ),
        })
        .set_span(throws_tag.span())
        .set_scope(operation.parser_scoped_identifier())
        .report(reporter);
    }
}

fn validate_throws_tags_for_operation_that_throws_a_specific_exception(
    throws_tags: &[ThrowsTag],
    operation: &Operation,
    thrown_exception: &TypeRef<Exception>,
    reporter: &mut DiagnosticReporter,
) {
    for throws_tag in throws_tags {
        if let Some(Ok(documented_exception)) = throws_tag.thrown_type() {
            if (thrown_exception.definition() as *const _) != (documented_exception as *const _) {
                let note_message = format!(
                    "operation '{}' only throws '{}'",
                    operation.identifier(),
                    thrown_exception.identifier(),
                );

                Diagnostic::new(Lint::IncorrectDocComment {
                    message: format!(
                        "comment has a 'throws' tag for '{}', but operation '{}' doesn't throw this exception",
                        documented_exception.identifier(),
                        operation.identifier(),
                    ),
                })
                .set_span(throws_tag.span())
                .set_scope(operation.parser_scoped_identifier())
                .add_note(note_message, Some(thrown_exception.span()))
                .report(reporter);
            }
        }
    }
}
