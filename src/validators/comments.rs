// Copyright (c) ZeroC, Inc.

use crate::diagnostics::{DiagnosticReporter, Warning, WarningKind};
use crate::grammar::*;
use crate::validators::{ValidationChain, Validator};

pub fn comments_validators() -> ValidationChain {
    vec![
        Validator::Entities(only_operations_can_throw),
        Validator::Operations(non_empty_return_comment),
        Validator::Operations(missing_parameter_comment),
        Validator::Operations(thrown_type_must_be_exception),
    ]
}

fn non_empty_return_comment(operation: &Operation, diagnostic_reporter: &mut DiagnosticReporter) {
    if let Some(comment) = operation.comment() {
        // Return doc comment exists but operation has no return members.
        // `DocComment.return_members` contains a list of descriptions of the return members.
        // example: @return A description of the return value.
        if !comment.returns.is_empty() && operation.return_members().is_empty() {
            for returns_tag in &comment.returns {
                Warning::new(WarningKind::ExtraReturnValueInDocComment)
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
                Warning::new(WarningKind::ExtraParameterInDocComment {
                    identifier: param_tag.identifier.value.clone(),
                })
                .set_span(param_tag.span())
                .set_scope(operation.parser_scoped_identifier())
                .report(diagnostic_reporter);
            }
        }
    }
}

fn only_operations_can_throw(entity: &dyn Entity, diagnostic_reporter: &mut DiagnosticReporter) {
    let supported_on = ["operation"];
    if let Some(comment) = entity.comment() {
        if !supported_on.contains(&entity.kind()) && !comment.throws.is_empty() {
            for throws_tag in &comment.throws {
                Warning::new(WarningKind::ExtraThrowInDocComment {
                    kind: entity.kind().to_owned(),
                    identifier: entity.identifier().to_owned(),
                })
                .set_span(throws_tag.span())
                .set_scope(entity.parser_scoped_identifier())
                .report(diagnostic_reporter);
            }
        }
    }
}

fn thrown_type_must_be_exception(operation: &Operation, diagnostic_reporter: &mut DiagnosticReporter) {
    if let Some(comment) = operation.comment() {
        for throws_tag in &comment.throws {
            if let Some(entity) = throws_tag.thrown_type() {
                // TODO: Add a better type check.
                if entity.kind() != "exception" {
                    Warning::new(WarningKind::InvalidThrowInDocComment {
                        identifier: entity.identifier().to_owned(),
                    })
                    .add_note(
                        format!(
                            "{} '{}' was defined here: ",
                            entity.kind().to_owned(),
                            entity.identifier()
                        ),
                        Some(entity.span()),
                    )
                    .add_note("operations can only throw exceptions", None)
                    .set_span(throws_tag.span())
                    .set_scope(operation.parser_scoped_identifier())
                    .report(diagnostic_reporter);
                }
            }
        }
    }
}
