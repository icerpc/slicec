// Copyright (c) ZeroC, Inc.

//! TODO write a doc comment for the module.

pub mod comment_link_patcher;
pub mod encoding_patcher;
pub mod type_ref_patcher;

use crate::ast::node::Node;
use crate::compilation_state::CompilationState;
use crate::diagnostics::{Diagnostic, Error};
use crate::grammar::attributes::*;
use crate::grammar::Symbol;

/// Since Slice definitions can be split across multiple files, and defined in any order, it is impossible for some
/// things to be determined during parsing (as it's a sequential process).
///
/// So, after parsing is complete, we modify the AST in place, 'patching' in the information that can only now be
/// computed, in the following order:
/// 1. References to other Slice types are verified and resolved.
/// 2. Compute and store the Slice encodings that each element can be used with.
///
/// This function fails fast, so if any phase of patching fails, we skip any remaining phases.
pub unsafe fn patch_ast(compilation_state: &mut CompilationState) {
    let attribute_patcher = crate::patch_attributes!("", Allow, Compress, Deprecated, Oneway, SlicedFormat);
    compilation_state.apply_unsafe(attribute_patcher);
    compilation_state.apply_unsafe(type_ref_patcher::patch_ast);
    compilation_state.apply_unsafe(encoding_patcher::patch_ast);
    compilation_state.apply_unsafe(comment_link_patcher::patch_ast);
}

#[macro_export]
macro_rules! patch_attributes {
    ($prefix:literal, $($attribute_type:ty),* $(,)?) => {{
        unsafe fn _patch_attributes_impl(compilation_state: &mut CompilationState) {
            let reporter = &mut compilation_state.diagnostic_reporter;

            // Iterate through every node in the AST.
            for node in compilation_state.ast.as_mut_slice() {

                // If that node is an attribute...
                if let Node::Attribute(attribute_ptr) = node {

                    // And it is unparsed...
                    let attribute = attribute_ptr.borrow_mut();
                    if let Some(unparsed) = attribute.downcast::<Unparsed>() {

                        // Check it's directive to see if it's one that we know about.
                        match unparsed.directive.as_str() {

                            // This block checks the unparsed attribute's directive against the directives of every
                            // type of attribute supplied to this macro.
                            $(
                            directive if directive == <$attribute_type>::directive() => {

                                // If one of those matched, call that attribute's `parse_from` function,
                                // and replace the unparsed attribute with the result.
                                let parsed = <$attribute_type>::parse_from(unparsed, attribute.span(), reporter);
                                attribute.kind = Box::new(parsed);
                            }
                            )*

                            directive => {
                                // If the directive starts with the provided prefix, but didn't match a known attribute.
                                let directive_prefix = directive.split_once("::").map_or("", |(p, _)| p);
                                if $prefix == directive_prefix {
                                    Diagnostic::new(Error::UnexpectedAttribute {
                                        attribute: directive.to_owned(),
                                    })
                                    .set_span(attribute.span())
                                    .report(reporter);
                                }
                            }
                        }
                    }
                }
            }
        }
        _patch_attributes_impl
    }}
}
