// Copyright (c) ZeroC, Inc.

use crate::ast::node::Node;
use crate::ast::{Ast, LookupError};
use crate::compilation_state::CompilationState;
use crate::diagnostics::{Diagnostic, Diagnostics, Lint};
use crate::downgrade_as;
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
        $patcher.apply_patches(&mut element_ref.comment);
    }};
}

pub unsafe fn patch_ast(compilation_state: &mut CompilationState) {
    let mut patcher = CommentLinkPatcher {
        link_patches: VecDeque::new(),
        diagnostics: &mut compilation_state.diagnostics,
    };

    // Immutably iterate through the AST and compute patches for all the doc comments stored in it.
    for node in compilation_state.ast.as_slice() {
        match node {
            Node::Struct(ptr) => patcher.compute_patches_for(ptr.borrow(), &compilation_state.ast),
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
    diagnostics: &'a mut Diagnostics,
}

impl CommentLinkPatcher<'_> {
    fn compute_patches_for(&mut self, commentable: &impl Commentable, ast: &Ast) {
        if let Some(comment) = commentable.comment() {
            if let Some(overview) = &comment.overview {
                self.resolve_links_in(overview, commentable, ast);
            }
            for param_tag in &comment.params {
                self.resolve_links_in(&param_tag.message, commentable, ast);
            }
            for returns_tag in &comment.returns {
                self.resolve_links_in(&returns_tag.message, commentable, ast);
            }
            for see_tag in &comment.see {
                self.resolve_link(&see_tag.link, commentable, ast);
            }
        }
    }

    fn resolve_links_in(&mut self, message: &Message, commentable: &impl Commentable, ast: &Ast) {
        for component in &message.value {
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
            .map_err(|lookup_error| match lookup_error {
                LookupError::DoesNotExist { identifier } => format!("no element named '{identifier}' exists in scope"),
                _ => unreachable!("`find_node_with_scope` reported an error other than `DoesNotExist`"),
            })
            .and_then(convert_node_to_entity_ptr);

        // If the lookup succeeded, store the result, otherwise report a lint violation and store `None` as a dummy.
        self.link_patches.push_back(match result {
            Ok(ptr) => Some(ptr),
            Err(message) => {
                Diagnostic::new(Lint::BrokenDocLink { message })
                    .set_span(identifier.span())
                    .set_scope(commentable.parser_scoped_identifier())
                    .push_into(self.diagnostics);
                None
            }
        });
    }

    fn apply_patches(&mut self, comment: &mut Option<DocComment>) {
        if let Some(comment) = comment {
            if let Some(overview) = &mut comment.overview {
                self.patch_links_in(overview);
            }
            for param_tag in &mut comment.params {
                self.patch_links_in(&mut param_tag.message);
            }
            for returns_tag in &mut comment.returns {
                self.patch_links_in(&mut returns_tag.message);
            }
            for see_tag in &mut comment.see {
                patch_link!(self, see_tag);
            }
        }
    }

    fn patch_links_in(&mut self, message: &mut Message) {
        for component in &mut message.value {
            if let MessageComponent::Link(link_tag) = component {
                patch_link!(self, link_tag);
            }
        }
    }
}

fn convert_node_to_entity_ptr(node: &Node) -> Result<WeakPtr<dyn Entity>, String> {
    match node {
        Node::Struct(struct_ptr) => Ok(downgrade_as!(struct_ptr, dyn Entity)),
        Node::Field(field_ptr) => Ok(downgrade_as!(field_ptr, dyn Entity)),
        Node::Interface(interface_ptr) => Ok(downgrade_as!(interface_ptr, dyn Entity)),
        Node::Operation(operation_ptr) => Ok(downgrade_as!(operation_ptr, dyn Entity)),
        Node::Enum(enum_ptr) => Ok(downgrade_as!(enum_ptr, dyn Entity)),
        Node::Enumerator(enumerator_ptr) => Ok(downgrade_as!(enumerator_ptr, dyn Entity)),
        Node::CustomType(custom_type_ptr) => Ok(downgrade_as!(custom_type_ptr, dyn Entity)),
        Node::TypeAlias(type_alias_ptr) => Ok(downgrade_as!(type_alias_ptr, dyn Entity)),

        Node::Module(_) => Err("modules cannot be linked to".to_owned()),
        Node::Parameter(_) => Err("parameters cannot be linked to".to_owned()), // TODO improve for return members.
        Node::Primitive(_) => Err("primitive types cannot be linked to".to_owned()),

        _ => unreachable!("`convert_node_to_entity_ptr` was called on an anonymous type or attribute"),
    }
}
