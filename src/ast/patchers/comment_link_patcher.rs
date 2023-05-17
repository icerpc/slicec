// Copyright (c) ZeroC, Inc.

use crate::ast::{Ast, LookupError, Node};
use crate::compilation_state::CompilationState;
use crate::diagnostics::{Diagnostic, DiagnosticReporter, Warning};
use crate::grammar::*;
use crate::utils::ptr_util::WeakPtr;
use std::collections::VecDeque;

macro_rules! patch_link {
    ($self:ident, $tag:expr) => {
        // Get the next patch out of the queue and apply it to the tag.
        if let Some(patch) = $self.link_patches.pop_front().unwrap() {
            $tag.link = TypeRefDefinition::Patched(patch);
        }
    };
}

macro_rules! patch_element {
    ($element_ptr:expr, $patcher:expr) => {{
        let element_ref = $element_ptr.borrow_mut();
        $patcher.apply_patches(&element_ref.parser_scoped_identifier(), &mut element_ref.comment);
    }};
}

pub unsafe fn patch_ast(compilation_state: &mut CompilationState) {
    let mut patcher = CommentLinkPatcher {
        link_patches: VecDeque::new(),
        diagnostic_reporter: &mut compilation_state.diagnostic_reporter,
    };

    // Immutably iterate through the AST and compute patches for all the doc comments stored in it.
    for node in compilation_state.ast.as_slice() {
        match node {
            Node::Struct(ptr) => patcher.compute_patches_for(ptr.borrow(), &compilation_state.ast),
            Node::Class(ptr) => patcher.compute_patches_for(ptr.borrow(), &compilation_state.ast),
            Node::Exception(ptr) => patcher.compute_patches_for(ptr.borrow(), &compilation_state.ast),
            Node::Field(ptr) => patcher.compute_patches_for(ptr.borrow(), &compilation_state.ast),
            Node::Interface(ptr) => patcher.compute_patches_for(ptr.borrow(), &compilation_state.ast),
            Node::Operation(ptr) => patcher.compute_patches_for(ptr.borrow(), &compilation_state.ast),
            Node::Enum(ptr) => patcher.compute_patches_for(ptr.borrow(), &compilation_state.ast),
            Node::Enumerator(ptr) => patcher.compute_patches_for(ptr.borrow(), &compilation_state.ast),
            Node::CustomType(ptr) => patcher.compute_patches_for(ptr.borrow(), &compilation_state.ast),
            Node::TypeAlias(ptr) => patcher.compute_patches_for(ptr.borrow(), &compilation_state.ast),
            _ => {} // Skip any elements that don't implement `Commentable`.
        }
    }

    // Mutably iterate through the AST and apply all the patches in the same oder they were computed.
    for node in compilation_state.ast.as_mut_slice() {
        match node {
            Node::Struct(ptr) => patch_element!(ptr, patcher),
            Node::Class(ptr) => patch_element!(ptr, patcher),
            Node::Exception(ptr) => patch_element!(ptr, patcher),
            Node::Field(ptr) => patch_element!(ptr, patcher),
            Node::Interface(ptr) => patch_element!(ptr, patcher),
            Node::Operation(ptr) => patch_element!(ptr, patcher),
            Node::Enum(ptr) => patch_element!(ptr, patcher),
            Node::Enumerator(ptr) => patch_element!(ptr, patcher),
            Node::CustomType(ptr) => patch_element!(ptr, patcher),
            Node::TypeAlias(ptr) => patch_element!(ptr, patcher),
            _ => {} // Skip any elements that don't implement `Commentable`.
        }
    }
    debug_assert!(patcher.link_patches.is_empty());
}

struct CommentLinkPatcher<'a> {
    link_patches: VecDeque<Option<WeakPtr<dyn Entity>>>,
    diagnostic_reporter: &'a mut DiagnosticReporter,
}

impl CommentLinkPatcher<'_> {
    fn compute_patches_for(&mut self, commentable: &impl Commentable, ast: &Ast) {
        if let Some(comment) = commentable.comment() {
            if let Some(overview) = &comment.overview {
                self.resolve_links_in(&overview.message, commentable, ast);
            }
            for param_tag in &comment.params {
                self.resolve_links_in(&param_tag.message, commentable, ast);
            }
            for returns_tag in &comment.returns {
                self.resolve_links_in(&returns_tag.message, commentable, ast);
            }
            for throws_tag in &comment.throws {
                if let Some(thrown_type) = &throws_tag.thrown_type {
                    self.resolve_link(thrown_type, commentable, ast);
                }
                self.resolve_links_in(&throws_tag.message, commentable, ast);
            }
            for see_tag in &comment.see {
                self.resolve_link(&see_tag.link, commentable, ast);
            }
        }
    }

    fn resolve_links_in(&mut self, message: &Message, commentable: &impl Commentable, ast: &Ast) {
        for component in message {
            if let MessageComponent::Link(link_tag) = component {
                self.resolve_link(&link_tag.link, commentable, ast);
            }
        }
    }

    fn resolve_link<T>(&mut self, link: &TypeRefDefinition<T>, commentable: &impl Commentable, ast: &Ast)
    where
        T: Element + ?Sized,
    {
        // All links should be unpatched at this point.
        let TypeRefDefinition::Unpatched(identifier) = link else {
            panic!("encountered comment link that was already patched");
        };

        // Look up the linked-to entity in the AST.
        let result = ast
            .find_node_with_scope(&identifier.value, &commentable.parser_scoped_identifier())
            .and_then(<WeakPtr<dyn Entity>>::try_from);

        // If the lookup succeeded, store the result, otherwise report a warning and store `None` as a placeholder.
        self.link_patches.push_back(match result {
            Ok(ptr) => Some(ptr),
            Err(error) => {
                let message = match error {
                    LookupError::DoesNotExist { identifier } => {
                        format!("no element named '{identifier}' exists in scope")
                    }
                    LookupError::TypeMismatch { actual, .. } => {
                        debug_assert_eq!(actual, "primitive"); // Only primitives are un-linkable and named.
                        format!("{actual}s cannot be linked to")
                    }
                };
                Diagnostic::new(Warning::BrokenDocLink { message })
                    .set_span(identifier.span())
                    .set_scope(commentable.parser_scoped_identifier())
                    .report(self.diagnostic_reporter);
                None
            }
        });
    }

    fn apply_patches(&mut self, scope: &str, comment: &mut Option<DocComment>) {
        if let Some(comment) = comment {
            if let Some(overview) = &mut comment.overview {
                self.patch_links_in(&mut overview.message);
            }
            for param_tag in &mut comment.params {
                self.patch_links_in(&mut param_tag.message);
            }
            for returns_tag in &mut comment.returns {
                self.patch_links_in(&mut returns_tag.message);
            }
            for throws_tag in &mut comment.throws {
                if throws_tag.thrown_type.is_some() {
                    self.patch_thrown_type(scope, throws_tag);
                }
                self.patch_links_in(&mut throws_tag.message);
            }
            for see_tag in &mut comment.see {
                patch_link!(self, see_tag);
            }
        }
    }

    fn patch_links_in(&mut self, message: &mut Message) {
        for component in message {
            if let MessageComponent::Link(link_tag) = component {
                patch_link!(self, link_tag);
            }
        }
    }

    fn patch_thrown_type(&mut self, scope: &str, tag: &mut ThrowsTag) {
        // Get the next patch out of the queue and apply it to the tag.
        if let Some(patch) = self.link_patches.pop_front().unwrap() {
            // If the linked-to type isn't an exception report a warning and leave the link unpatched.
            match patch.downcast::<Exception>() {
                Ok(converted_patch) => {
                    tag.thrown_type = Some(TypeRefDefinition::Patched(converted_patch));
                }
                Err(original_patch) => {
                    let entity = original_patch.borrow();
                    Diagnostic::new(Warning::IncorrectDocComment {
                        message: format!("'{}' is not a throwable type", entity.identifier()),
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
                    .set_span(tag.span())
                    .set_scope(scope)
                    .report(self.diagnostic_reporter);
                }
            }
        }
    }
}
