// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::ast::{Ast, Node};
use crate::compilation_result::{CompilationData, CompilationResult};
use crate::diagnostics::{DiagnosticReporter, Error, ErrorKind, Warning, WarningKind};
use crate::downgrade_as;
use crate::grammar::{DocComment, Entity, LinkDefinition, Message, MessageComponent, Symbol};
use crate::utils::ptr_util::WeakPtr;
use convert_case::{Case, Casing};

pub unsafe fn patch_ast(mut compilation_data: CompilationData) -> CompilationResult {
    let mut patcher = CommentLinkPatcher {
        link_patches: Vec::new(),
        diagnostic_reporter: &mut compilation_data.diagnostic_reporter,
    };

    // Iterate through the AST and compute patches for all the doc comments stored in it.
    for node in compilation_data.ast.as_slice() {
        if let Ok(entity) = <&dyn Entity>::try_from(node) {
            if let Some(comment) = entity.comment() {
                patcher.compute_patches_for(comment, entity, &compilation_data.ast);
            }
        }
    }

    // Convert the patcher into an iterator of patches, then apply them in order.
    let patches = &mut patcher.link_patches.into_iter();
    for node in compilation_data.ast.as_mut_slice() {
        match node {
            Node::Module(ptr) => apply_patches(&mut ptr.borrow_mut().comment, patches),
            Node::Struct(ptr) => apply_patches(&mut ptr.borrow_mut().comment, patches),
            Node::Class(ptr) => apply_patches(&mut ptr.borrow_mut().comment, patches),
            Node::Exception(ptr) => apply_patches(&mut ptr.borrow_mut().comment, patches),
            Node::DataMember(ptr) => apply_patches(&mut ptr.borrow_mut().comment, patches),
            Node::Interface(ptr) => apply_patches(&mut ptr.borrow_mut().comment, patches),
            Node::Operation(ptr) => apply_patches(&mut ptr.borrow_mut().comment, patches),
            Node::Parameter(ptr) => apply_patches(&mut ptr.borrow_mut().comment, patches),
            Node::Enum(ptr) => apply_patches(&mut ptr.borrow_mut().comment, patches),
            Node::Enumerator(ptr) => apply_patches(&mut ptr.borrow_mut().comment, patches),
            Node::CustomType(ptr) => apply_patches(&mut ptr.borrow_mut().comment, patches),
            Node::TypeAlias(ptr) => apply_patches(&mut ptr.borrow_mut().comment, patches),
            _ => {} // Skip any non-entity types.
        }
    }
    assert!(patches.next().is_none());

    compilation_data.into()
}

macro_rules! resolve_link {
    ($tag:expr, $ident:expr, $entity:expr, $ast:expr, $self:ident) => {
        // All links should be unpatched at this point.
        debug_assert!(matches!($tag.definition, LinkDefinition::Unpatched));

        // Look up the link in the AST, and make sure it's an `Entity`.
        let result = $ast
            .find_node_with_scope(&$ident.value, $entity.parser_scope())
            .and_then(|node| try_into_patch(node));

        $self.link_patches.push(match result {
            Ok(ptr) => Some(ptr),
            Err(error) => {
                let warning_kind = match error.kind() {
                    ErrorKind::DoesNotExist { identifier } => WarningKind::DoesNotExist {
                        identifier: identifier.to_owned(),
                    },
                    ErrorKind::TypeMismatch { actual, .. } => WarningKind::LinkToInvalidElement {
                        kind: actual.to_owned(),
                    },
                    _ => unreachable!(), // No other types of errors can be returned from `find_element_with_scope`
                };
                Warning::new(warning_kind)
                    .set_span($tag.span())
                    .set_scope($entity.parser_scoped_identifier())
                    .report($self.diagnostic_reporter);
                None
            }
        });
    };
}

struct CommentLinkPatcher<'a> {
    link_patches: Vec<Option<WeakPtr<dyn Entity>>>,
    diagnostic_reporter: &'a mut DiagnosticReporter,
}

#[allow(clippy::result_large_err)] // TODO Adding a new result type for AST lookup would solve this.
impl CommentLinkPatcher<'_> {
    fn compute_patches_for(&mut self, comment: &DocComment, entity: &dyn Entity, ast: &Ast) {
        if let Some(overview) = &comment.overview {
            self.resolve_links_in(&overview.message, entity, ast);
        }
        for param_tag in &comment.params {
            self.resolve_links_in(&param_tag.message, entity, ast);
        }
        for returns_tag in &comment.returns {
            self.resolve_links_in(&returns_tag.message, entity, ast);
        }
        for throws_tag in &comment.throws {
            if let Some(identifier) = &throws_tag.identifier {
                resolve_link!(throws_tag, identifier, entity, ast, self);
            }
            self.resolve_links_in(&throws_tag.message, entity, ast);
        }
        for see_tag in &comment.see {
            resolve_link!(see_tag, &see_tag.link, entity, ast, self);
        }
    }

    fn resolve_links_in(&mut self, message: &Message, entity: &dyn Entity, ast: &Ast) {
        for component in message {
            if let MessageComponent::Link(link_tag) = component {
                resolve_link!(link_tag, &link_tag.link, entity, ast, self);
            }
        }
    }
}

fn apply_patches(comment: &mut Option<DocComment>, patches: &mut impl Iterator<Item = Option<WeakPtr<dyn Entity>>>) {
    if let Some(comment) = comment {
        if let Some(overview) = &mut comment.overview {
            patch_links_in(&mut overview.message, patches);
        }
        for param_tag in &mut comment.params {
            patch_links_in(&mut param_tag.message, patches);
        }
        for returns_tag in &mut comment.params {
            patch_links_in(&mut returns_tag.message, patches);
        }
        for throws_tag in &mut comment.throws {
            if throws_tag.identifier.is_some() {
                if let Some(patch) = patches.next().unwrap() {
                    throws_tag.definition = LinkDefinition::Patched(patch);
                }
            }
            patch_links_in(&mut throws_tag.message, patches);
        }
        for see_tag in &mut comment.see {
            if let Some(patch) = patches.next().unwrap() {
                see_tag.definition = LinkDefinition::Patched(patch);
            }
        }
    }
}

fn patch_links_in(message: &mut Message, patches: &mut impl Iterator<Item = Option<WeakPtr<dyn Entity>>>) {
    for component in message {
        if let MessageComponent::Link(link_tag) = component {
            if let Some(patch) = patches.next().unwrap() {
                link_tag.definition = LinkDefinition::Patched(patch);
            }
        }
    }
}

#[allow(clippy::result_large_err)]
fn try_into_patch(node: &Node) -> Result<WeakPtr<dyn Entity>, Error> {
    match node {
        Node::Module(ptr) => Ok(downgrade_as!(ptr, dyn Entity)),
        Node::Struct(ptr) => Ok(downgrade_as!(ptr, dyn Entity)),
        Node::Class(ptr) => Ok(downgrade_as!(ptr, dyn Entity)),
        Node::Exception(ptr) => Ok(downgrade_as!(ptr, dyn Entity)),
        Node::DataMember(ptr) => Ok(downgrade_as!(ptr, dyn Entity)),
        Node::Interface(ptr) => Ok(downgrade_as!(ptr, dyn Entity)),
        Node::Operation(ptr) => Ok(downgrade_as!(ptr, dyn Entity)),
        Node::Parameter(ptr) => Ok(downgrade_as!(ptr, dyn Entity)),
        Node::Enum(ptr) => Ok(downgrade_as!(ptr, dyn Entity)),
        Node::Enumerator(ptr) => Ok(downgrade_as!(ptr, dyn Entity)),
        Node::CustomType(ptr) => Ok(downgrade_as!(ptr, dyn Entity)),
        Node::TypeAlias(ptr) => Ok(downgrade_as!(ptr, dyn Entity)),
        _ => Err(Error::new(ErrorKind::TypeMismatch {
            expected: "Entity".to_owned(),
            actual: node.to_string().to_case(Case::Lower),
        })),
    }
}
