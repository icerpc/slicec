// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::ast::Ast;
use crate::diagnostics::{DiagnosticReporter, Warning, WarningKind};
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
            Warning::new(WarningKind::ExtraReturnValueInDocComment, comment.span())
                .report(diagnostic_reporter, operation);
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
                Warning::new(
                    WarningKind::ExtraParameterInDocComment {
                        identifier: param.0.clone(),
                    },
                    comment.span(),
                )
                .report(diagnostic_reporter, operation);
            }
        });
    }
}

fn only_operations_can_throw(entity: &dyn Entity, diagnostic_reporter: &mut DiagnosticReporter) {
    let supported_on = ["operation"];
    if let Some(comment) = entity.comment() {
        if !supported_on.contains(&entity.kind()) && !comment.throws.is_empty() {
            let warning_kind = WarningKind::ExtraThrowInDocComment {
                kind: entity.kind().to_owned(),
                identifier: entity.identifier().to_owned(),
            };
            Warning::new(warning_kind, comment.span()).report(diagnostic_reporter, entity)
        };
    }
}

fn linked_identifiers_exist(entity: &dyn Entity, ast: &Ast, diagnostic_reporter: &mut DiagnosticReporter) {
    if let Some(comment) = entity.comment() {
        for (tag_type, value) in find_inline_tags(&comment.overview) {
            match tag_type {
                "@link" => {
                    if ast
                        .find_element_with_scope::<dyn Entity>(value, entity.module_scope())
                        .is_err()
                    {
                        Warning::new(
                            WarningKind::InvalidDocCommentLinkIdentifier {
                                identifier: value.to_owned(),
                            },
                            comment.span(),
                        )
                        .report(diagnostic_reporter, entity);
                    }
                }
                other if other.starts_with('@') => {
                    Warning::new(
                        WarningKind::InvalidDocCommentTag { tag: other.to_owned() },
                        comment.span(),
                    )
                    .report(diagnostic_reporter, entity);
                }
                _ => {}
            }
        }
    }
}
