// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::ast::Ast;
use crate::diagnostics::{Diagnostic, DiagnosticReporter, WarningKind};
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
        if comment.returns.is_some() && operation.return_members().is_empty() {
            diagnostic_reporter.report(Diagnostic::new(
                WarningKind::ExtraReturnValueInDocComment,
                Some(comment.span()),
            ));
        }
    }
}

fn missing_parameter_comment(operation: &Operation, diagnostic_reporter: &mut DiagnosticReporter) {
    if let Some(comment) = operation.comment() {
        comment.params.iter().for_each(|param| {
            if !operation
                .parameters()
                .iter()
                .map(|p| p.identifier.value.clone())
                .any(|identifier| identifier == param.0)
            {
                diagnostic_reporter.report(Diagnostic::new(
                    WarningKind::ExtraParameterInDocComment(param.0.clone()),
                    Some(comment.span()),
                ));
            }
        });
    }
}

fn only_operations_can_throw(commentable: &dyn Entity, diagnostic_reporter: &mut DiagnosticReporter) {
    let supported_on = ["operation"];
    if let Some(comment) = commentable.comment() {
        if !supported_on.contains(&commentable.kind()) && !comment.throws.is_empty() {
            let warning =
                WarningKind::ExtraThrowInDocComment(commentable.kind().to_owned(), commentable.identifier().to_owned());
            diagnostic_reporter.report(Diagnostic::new(warning, Some(comment.span())));
        };
    }
}

fn linked_identifiers_exist(commentable: &dyn Entity, ast: &Ast, diagnostic_reporter: &mut DiagnosticReporter) {
    if let Some(comment) = commentable.comment() {
        for (tag_type, value) in find_inline_tags(&comment.overview) {
            match tag_type {
                "@link" => {
                    println!("{value}");
                    if ast
                        .find_element_with_scope::<dyn Entity>(value, commentable.module_scope())
                        .is_err()
                    {
                        diagnostic_reporter.report(Diagnostic::new(
                            WarningKind::InvalidDocCommentLinkIdentifier(value.to_owned()),
                            Some(comment.span()),
                        ));
                    }
                }
                other if other.starts_with('@') => {
                    diagnostic_reporter.report(Diagnostic::new(
                        WarningKind::InvalidDocCommentTag(other.to_owned()),
                        Some(comment.span()),
                    ));
                }
                _ => {}
            }
        }
    }
}
