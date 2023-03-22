// Copyright (c) ZeroC, Inc.

use crate::diagnostics::{Diagnostic, DiagnosticReporter, Warning};
use crate::grammar::*;
use crate::validators::{ValidationChain, Validator};

pub fn comments_validators() -> ValidationChain {
    vec![
        Validator::Entities(only_operations_can_throw),
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
                Diagnostic::new(Warning::ExtraReturnValueInDocComment)
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
                Diagnostic::new(Warning::ExtraParameterInDocComment {
                    identifier: param_tag.identifier.value.clone(),
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
            Diagnostic::new(Warning::OperationDoesNotThrow {
                identifier: operation.identifier().to_owned(),
            })
            .set_span(operation.span())
            .set_scope(operation.parser_scoped_identifier())
            .report(diagnostic_reporter);
        }
    }
}

fn only_operations_can_throw(entity: &dyn Entity, diagnostic_reporter: &mut DiagnosticReporter) {
    let supported_on = ["operation"];
    if let Some(comment) = entity.comment() {
        if !supported_on.contains(&entity.kind()) && !comment.throws.is_empty() {
            for throws_tag in &comment.throws {
                Diagnostic::new(Warning::ExtraThrowInDocComment {
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
