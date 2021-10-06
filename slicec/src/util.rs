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

/// TODOAUSTIN write a good comment here
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum CaseStyle {
    Camel,
    Pascal,
    Snake,
}

// TODOAUSTIN write a good comment here. THIS EXPECTS 's' to be in camel case!!!
pub fn fix_case(s: &str, case: CaseStyle) -> String {
    if s.is_empty() {
        return String::new();
    }

    match case {
        CaseStyle::Camel => s.to_owned(), // strings are in camel-case by default.
        CaseStyle::Pascal => {
            let mut chars = s.chars();
            // We already handled empty strings, so unwrap is safe; there must be at least 1 char.
            let first_letter = chars.next().unwrap();

            // We capitalize the first letter and convert it to an owned string, then append the
            // rest of the original string to it. The 'chars' iterator skipped over the first char
            // when we called 'next', and so only contains the rest of the string.
            //
            // We need to 'collect' here, since 'to_uppercase' returns an iterator. 1 lowercase
            // grapheme can produce multiple graphemes when capitalized in UTF8.
            first_letter.to_uppercase().collect::<String>() + chars.as_str()
        }
        CaseStyle::Snake => {
            s.to_owned() // TODOAUSTIN I need to actually write this logic.
        }
    }
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

    // Sanity check that the tagged members are sorted by tag
    assert!(tagged_members
        .windows(2)
        .all(|member_pair| member_pair[0].tag.unwrap() < member_pair[1].tag.unwrap()));

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
/// Gets a Node of the give type as an Optional<$t> from node $n
/// e.g. is_some!(Node::Primitive, a_node)
#[macro_export]
macro_rules! is_some {
    ($t:path, $n:expr) => {{
        if let $t(_, p) = $n {
            Some(p)
        } else {
            None
        }
    }};
}
