// Copyright (c) ZeroC, Inc.

//! TODO write a doc comment for the module.

pub mod comment_link_patcher;
pub mod encoding_patcher;
pub mod type_ref_patcher;

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
