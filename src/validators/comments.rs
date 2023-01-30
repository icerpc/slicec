// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::ast::Ast;
use crate::diagnostics::{DiagnosticReporter, ErrorKind, Warning, WarningKind};
use crate::grammar::*;
use crate::validators::{ValidationChain, Validator};

pub fn comments_validators() -> ValidationChain {
    vec![
        Validator::Entities(only_operations_can_throw),
        Validator::Operations(non_empty_return_comment),
        Validator::Operations(missing_parameter_comment),
        Validator::DocComments(linked_identifiers_exist),
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
                .report(diagnostic_reporter)
            }
        };
    }
}

macro_rules! check_link {
    ($tag:expr, $entity:expr, $ast:expr, $diagnostic_reporter:expr) => {
        if let Err(error) = $ast.find_element_with_scope::<dyn Entity>(&$tag.link.value, $entity.module_scope()) {
            let message = match error.kind() {
                ErrorKind::DoesNotExist { identifier } => {
                    format!("no element with identifier '{identifier}' can be found from this scope.")
                }
                ErrorKind::TypeMismatch { actual, .. } => {
                    format!("elements of type '{actual}' cannot be referenced in doc comments.")
                }
                _ => unreachable!(), // No other types of errors can be thrown from `find_element_with_scope`
            };
            Warning::new(WarningKind::InvalidDocCommentLinkIdentifier { message })
                .set_span(&$tag.span)
                .set_scope($entity.parser_scoped_identifier())
                .report($diagnostic_reporter);
        }
    };
}

fn linked_identifiers_exist(entity: &dyn Entity, ast: &Ast, diagnostic_reporter: &mut DiagnosticReporter) {
    let mut check_links_in = |message: &Message| {
        for component in message {
            if let MessageComponent::Link(link_tag) = component {
                check_link!(link_tag, entity, ast, diagnostic_reporter);
            }
        }
    };

    if let Some(comment) = entity.comment() {
        if let Some(overview) = &comment.overview {
            check_links_in(&overview.message);
        }
        for param_tag in &comment.params {
            check_links_in(&param_tag.message);
        }
        for returns_tag in &comment.returns {
            check_links_in(&returns_tag.message);
        }
        for throws_tag in &comment.throws {
            check_links_in(&throws_tag.message);
        }
        for see_tag in &comment.see {
            check_link!(see_tag, entity, ast, diagnostic_reporter);
        }
    }
}
