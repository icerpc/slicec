// Copyright (c) ZeroC, Inc.

use crate::ast::node::Node;
use crate::ast::Ast;
use crate::diagnostics::{Diagnostic, Diagnostics, Error, Note};
use crate::grammar::*;
use std::collections::{BTreeSet, HashSet};

pub(super) fn detect_cycles(ast: &Ast, diagnostics: &mut Diagnostics) {
    let mut cycle_detector = CycleDetector {
        type_being_checked: None,
        dependency_stack: Vec::new(),
        reported_cycles: HashSet::new(),
        diagnostics,
    };

    for node in ast.as_slice() {
        let candidate: &dyn CycleCandidate = match node {
            // Only structs and enums need to be checked for cycles.
            // Classes can safely contain cycles since they use reference semantics,
            // exceptions can't cause cycles since they cannot be used as types,
            // and type-alias cycles are caught during the type-patching phase.
            Node::Struct(struct_def) => struct_def.borrow(),
            Node::Enum(enum_def) => enum_def.borrow(),
            _ => continue,
        };

        debug_assert!(cycle_detector.dependency_stack.is_empty());
        cycle_detector.type_being_checked = Some((candidate.module_scoped_identifier(), candidate));
        candidate.check_for_cycles(&mut cycle_detector)
    }
}

/// This trait is implemented on a type if and only if it is possible for that type to cause a cycle.
/// It contains a single method, used to check the type for cycles with the help of a [`CycleDetector`].
trait CycleCandidate<'a>: Type + NamedSymbol {
    fn check_for_cycles(&'a self, cycle_detector: &mut CycleDetector<'a>);
}

impl<'a> CycleCandidate<'a> for Struct {
    /// Checks this struct's fields for cycles.
    fn check_for_cycles(&'a self, cycle_detector: &mut CycleDetector<'a>) {
        cycle_detector.check_fields_for_cycles(self);
    }
}

impl<'a> CycleCandidate<'a> for Enum {
    /// Iterates through the enumerators of this enum, and checks any fields for cycles.
    fn check_for_cycles(&'a self, cycle_detector: &mut CycleDetector<'a>) {
        for enumerator in self.enumerators() {
            cycle_detector.check_fields_for_cycles(enumerator);
        }
    }
}

struct CycleDetector<'a> {
    /// Stores a tuple of `(type_id, reference)` for the type currently being checked for cycles.
    type_being_checked: Option<(String, &'a dyn CycleCandidate<'a>)>,

    /// A stack containing all the fields we've seen in the dependency tree we're currently traversing through.
    /// Each stack element is made up of the type-id of the field's type, and a reference to the field itself.
    dependency_stack: Vec<(String, &'a Field)>,

    /// Stores all the cycles we've reported so far, so we can avoid reporting duplicates.
    reported_cycles: HashSet<BTreeSet<String>>,

    /// Reference to a diagnostics struct for reporting errors.
    diagnostics: &'a mut Diagnostics,
}

impl<'a> CycleDetector<'a> {
    fn check_fields_for_cycles(&mut self, container: &'a dyn Container<Field>) {
        for field in container.contents() {
            self.check_field_type_for_cycles(field.data_type(), field);
        }
    }

    fn check_field_type_for_cycles(&mut self, type_ref: &'a TypeRef, origin: &'a Field) {
        match type_ref.concrete_type() {
            // For struct or enum types, we push them onto the stack, and attempt to recursively check them.
            Types::Struct(struct_ref) => self.push_to_stack_and_check(struct_ref, origin),
            Types::Enum(enum_ref) => self.push_to_stack_and_check(enum_ref, origin),

            Types::ResultType(result_type) => {
                self.check_field_type_for_cycles(&result_type.ok_type, origin);
                self.check_field_type_for_cycles(&result_type.err_type, origin);
            }

            Types::Sequence(sequence) => self.check_field_type_for_cycles(&sequence.element_type, origin),
            Types::Dictionary(dictionary) => {
                self.check_field_type_for_cycles(&dictionary.key_type, origin);
                self.check_field_type_for_cycles(&dictionary.value_type, origin);
            }

            // Classes always break cycles since they use reference semantics.
            Types::Class(_) => {}

            // Primitive and custom types are terminal since they can't reference any other types.
            Types::Primitive(_) | Types::CustomType(_) => {}
        }
    }

    fn push_to_stack_and_check(&mut self, candidate: &'a dyn CycleCandidate<'a>, origin: &'a Field) {
        let candidate_type_string = candidate.module_scoped_identifier();

        // If the candidate's type is the type we're checking, then its definition is cyclic and we report an error.
        if self.type_being_checked.as_ref().unwrap().0 == candidate_type_string {
            // We still push the offending field onto the stack so we can use it in the error message.
            self.dependency_stack.push((candidate_type_string, origin));
            self.report_cycle_error();
            self.dependency_stack.pop();
            return;
        }

        // If the candidate is in the dependency stack, but isn't the type we're checking, skip it.
        // There is a cycle present (and so we have to return to avoid infinite recursion), but the
        // candidate isn't the cause of the cycle, just a link or offshoot of it.
        for (seen_type_id, _) in &self.dependency_stack {
            if seen_type_id == &candidate_type_string {
                return;
            }
        }

        // If we haven't detected any cycles yet, it's safe to continue recursing.
        // Push the current field and its type onto the stack, then check the candidate's fields.
        self.dependency_stack.push((candidate_type_string, origin));
        candidate.check_for_cycles(self);
        self.dependency_stack.pop();
    }

    fn report_cycle_error(&mut self) {
        // If we've already reported this cycle, do not report it again. For cycles consisting of N different types,
        // the cycle checker will detect N cycles, one for each type. But, logically these are all the same cycle.
        //
        // To prevent duplicate diagnostics, we take a set of the cycle elements, and check whether we've reported it.
        // Graph Theory tells us that directed cycles are NOT uniquely identified by their vertex sets, but good enough.
        let cycle_set: BTreeSet<String> = self.dependency_stack.iter().map(|(id, _)| id.clone()).collect();
        if !self.reported_cycles.insert(cycle_set) {
            return;
        }

        let type_being_checked = self.type_being_checked.as_ref().unwrap();
        let type_id = type_being_checked.0.clone();

        // Create a string showing the cycle that was detected (a string of the form "A -> B -> C -> A").
        let mut cycle = type_id.clone();
        for (link_type_id, _) in &self.dependency_stack {
            cycle = cycle + " -> " + link_type_id;
        }

        // Create notes for explaining the cycle's links in greater detail.
        let cycle_notes = self.dependency_stack.iter().map(|(_, field)| Self::get_note_for(field));

        // Report the error.
        Diagnostic::new(Error::InfiniteSizeCycle { type_id, cycle })
            .set_span(type_being_checked.1.span())
            .extend_notes(cycle_notes)
            .push_into(self.diagnostics);
    }

    fn get_note_for(field: &Field) -> Note {
        // Determine which kind of entity holds this field.
        let parent_type: &dyn Entity = match field.parent().concrete_entity() {
            Entities::Struct(struct_def) => struct_def,
            Entities::Enumerator(enumerator) => enumerator.parent(), // enumerators aren't types, we want the enum.
            _ => unreachable!("Attempted to get cycle note for a container that wasn't a struct or enumerator!"),
        };

        // Create and return a note explaining how this field fits into the cycle.
        let message = format!(
            "{container_kind} '{container}' contains a field named '{field}' that is of type '{field_type}'",
            container_kind = parent_type.kind(),
            container = parent_type.identifier(),
            field = field.identifier(),
            field_type = field.data_type().type_string(),
        );
        let span = Some(field.span().clone());
        Note { message, span }
    }
}
