// Copyright (c) ZeroC, Inc. All rights reserved.

use super::ValidatorVisitor;
use crate::diagnostics::{Warning, WarningKind};
use crate::grammar::*;

impl ValidatorVisitor<'_> {
pub(super) fn non_empty_return_comment(&mut self, operation: &Operation) {
    if let Some(comment) = operation.comment() {
        // Return doc comment exists but operation has no return members.
        // `DocComment.return_members` contains a list of descriptions of the return members.
        // example: @return A description of the return value.
        if comment.returns.is_some() && operation.return_members().is_empty() {
            Warning::new(WarningKind::ExtraReturnValueInDocComment, comment.span())
                .report(self.diagnostic_reporter, operation);
        }
    }
}

pub(super) fn missing_parameter_comment(&mut self, operation: &Operation) {
    if let Some(comment) = operation.comment() {
        comment.params.iter().for_each(|param| {
            if !operation
                .parameters()
                .iter()
                .map(|p| p.identifier.value.clone())
                .any(|identifier| identifier == param.0)
            {
                Warning::new(WarningKind::ExtraParameterInDocComment(param.0.clone()), comment.span())
                    .report(self.diagnostic_reporter, operation);
            }
        });
    }
}

pub(super) fn only_operations_can_throw(&mut self, entity: &dyn Entity) {
    let supported_on = ["operation"];
    if let Some(comment) = entity.comment() {
        if !supported_on.contains(&entity.kind()) && !comment.throws.is_empty() {
            let warning_kind =
                WarningKind::ExtraThrowInDocComment(entity.kind().to_owned(), entity.identifier().to_owned());
            Warning::new(warning_kind, comment.span()).report(self.diagnostic_reporter, entity)
        };
    }
}

pub(super) fn linked_identifiers_exist(&mut self, entity: &dyn Entity) {
    if let Some(comment) = entity.comment() {
        for (tag_type, value) in find_inline_tags(&comment.overview) {
            match tag_type {
                "@link" => {
                    if self.ast
                        .find_element_with_scope::<dyn Entity>(value, entity.module_scope())
                        .is_err()
                    {
                        Warning::new(
                            WarningKind::InvalidDocCommentLinkIdentifier(value.to_owned()),
                            comment.span(),
                        )
                        .report(self.diagnostic_reporter, entity);
                    }
                }
                other if other.starts_with('@') => {
                    Warning::new(WarningKind::InvalidDocCommentTag(other.to_owned()), comment.span())
                        .report(self.diagnostic_reporter, entity);
                }
                _ => {}
            }
        }
    }
}
}
