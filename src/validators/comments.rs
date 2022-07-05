// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::error::ErrorReporter;
use crate::grammar::*;
use crate::validators::{ValidationChain, Validator};

pub fn comments_validators() -> ValidationChain {
    vec![
        Validator::Entities(only_operations_can_throw),
        Validator::Operations(non_empty_return_comment),
        Validator::Operations(missing_parameter_comment),
    ]
}

fn non_empty_return_comment(operation: &Operation, error_reporter: &mut ErrorReporter) {
    if let Some(comment) = operation.comment() {
        // Return doc comment exists but operation has no return members.
        // `DocComment.return_members` contains a list of descriptions of the return members.
        // example: @return A description of the return value.`
        if comment.returns.is_some() && operation.return_members().is_empty() {
            error_reporter.report_warning(
                "void operation must not contain doc comment return tag",
                Some(&comment.location),
            );
        }
    }
}

fn missing_parameter_comment(operation: &Operation, error_reporter: &mut ErrorReporter) {
    if let Some(comment) = operation.comment() {
        comment.params.iter().for_each(|param| {
            if !operation
                .parameters()
                .iter()
                .map(|p| p.identifier.value.clone())
                .any(|identifier| identifier == param.0)
            {
                error_reporter.report_warning(
                    format!(
                        "doc comment has a param tag for '{param_name}', but there is no parameter by that name",
                        param_name = param.0,
                    ),
                    Some(&comment.location),
                );
            }
        })
    }
}

fn only_operations_can_throw(commentable: &dyn Entity, error_reporter: &mut ErrorReporter) {
    let supported_on = ["operation"];
    if let Some(comment) = commentable.comment() {
        if !supported_on.contains(&commentable.kind()) && !comment.throws.is_empty() {
            error_reporter.report_warning(
                format!(
                    "doc comment indicates that {kind} `{op_identifier}` throws, however, only operations can throw",
                    kind = &commentable.kind(),
                    op_identifier = commentable.identifier(),
                ),
                Some(&comment.location),
            );
        };
    }
}
