// Copyright (c) ZeroC, Inc.

use crate::ast::node::Node;
use crate::ast::Ast;
use crate::diagnostics::{Diagnostic, Diagnostics, Error};
use crate::grammar::{Container, Field, Member, Types};

pub(super) fn detect_cycles(ast: &Ast, diagnostics: &mut Diagnostics) {
    let mut cycle_detector = CycleDetector {
        dependency_stack: Vec::new(),
        diagnostics,
    };

    for node in ast.as_slice() {
        cycle_detector.dependency_stack.clear(); // Make sure the detector is cleared between checks.
        match node {
            // Only structs and enumerators (with associated fields) need to be checked for cycles.
            // It is safe for classes to contain cycles since they use reference semantics,
            // and exceptions can't cause cycles since they cannot be used as types.
            Node::Struct(struct_def) => cycle_detector.check_for_cycles(struct_def.borrow()),
            Node::Enumerator(enumerator) => cycle_detector.check_for_cycles(enumerator.borrow()),
            _ => false,
        };
    }
}

struct CycleDetector<'a> {
    /// A stack containing all of the types we've seen in the dependency tree we're currently traversing through.
    dependency_stack: Vec<String>,

    /// Reference to a diagnostics struct for reporting `InfiniteSizeCycle` errors.
    diagnostics: &'a mut Diagnostics,
}

impl CycleDetector<'_> {
    fn check_for_cycles(&mut self, container: &impl Container<Field>) -> bool {
        let type_id = container.module_scoped_identifier();

        if self.dependency_stack.first() == Some(&type_id) {
            // If this container's identifier is equal to the first element in the stack, then its definition is cyclic.
            // We report an error, then return `true` to signal to parent functions they should stop checking.
            let cycle = self.dependency_stack.join(" -> ") + " -> " + &type_id;
            Diagnostic::new(Error::InfiniteSizeCycle { type_id, cycle })
                .set_span(container.span())
                .push_into(self.diagnostics);
            return true;
        } else if self.dependency_stack.contains(&type_id) {
            // If this container's identifier is in the stack, but isn't the first element, this means it uses a cyclic
            // type, but isn't cyclic itself. We don't check its fields (that would cause an infinite cycle), but don't
            // want to report an error since this container isn't the 'real' problem. We just do nothing.
        } else {
            // If this container's identifier isn't in the stack, we check its fields for cycles.
            self.dependency_stack.push(type_id);
            for field in container.contents() {
                let cycle_was_found = match field.data_type().concrete_type() {
                    Types::Struct(struct_def) => self.check_for_cycles(struct_def),
                    // TODO we need to recursively check enums here now!
                    _ => false,
                };

                // If a cycle was found, stop searching and return immediately.
                if cycle_was_found {
                    return true;
                }
            }
            self.dependency_stack.pop();
        }
        false
    }
}
