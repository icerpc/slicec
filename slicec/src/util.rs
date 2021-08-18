// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::ast::Ast;
use crate::grammar::Member;

/// The context that a type is being used in while generating code. This is used primarily by the
/// `type_to_string` methods in each of the language mapping's code generators.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum TypeContext {
    /// Used when generating the types of data members in structs and classes.
    DataMember,
    /// Used when generating the types of operation members (parameters and return types) in places
    /// where they're being read off the wire and unmarshalled.
    Incoming,
    /// Used when generating the types of operation members (parameters and return types) in places
    /// where they're being going to be marshalled and written onto the wire.
    Outgoing,
    /// Used when generating types that are parts of other types, such as the key & value types of
    /// dictionaries, or the element type of a sequence.
    Nested,
}

pub fn get_bit_sequence_size(members: &[&Member], ast: &Ast) -> i32 {
    let mut size: i32 = 0;
    for member in members {
        if member.data_type.encode_using_bit_sequence(ast) && member.tag.is_none() {
            size += 1;
        }
    }
    size
}

/// Takes a slice of Member references and returns two vectors. One containing the required members
/// and the other containing the tagged members. The tagged vector is sorted by it's tags.
pub fn get_sorted_members<'a>(members: &[&'a Member]) -> (Vec<&'a Member>, Vec<&'a Member>) {
    let required_members = members
        .iter()
        .filter(|m| m.tag.is_none())
        .cloned()
        .collect::<Vec<_>>();
    let mut tagged_members = members
        .iter()
        .filter(|m| m.tag.is_some())
        .cloned()
        .collect::<Vec<_>>();
    tagged_members.sort_by_key(|m| m.tag.unwrap());

    (required_members, tagged_members)
}

/// Compare the TypeRef's underlying type
#[macro_export]
macro_rules! is_underlying_type {
    ($type_ref:expr, $ast:expr, $of_type:path) => {{
        let node = $ast.resolve_index($type_ref.definition.unwrap());
        if let $of_type(_, _) = node {
            true
        } else {
            false
        }
    }};
}
