// Copyright (c) ZeroC, Inc. All rights reserved.

// TODO this entire file needs to be looked over again.

use crate::grammar::{Element, Member, TypeRef};

/// The context that a type is being used in while generating code. This is used primarily by the
/// `type_to_string` methods in each of the language mapping's code generators.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum TypeContext {
    /// Used when generating the types of data members in structs, classes, and exceptions.
    DataMember,
    /// Used when generating the types of operation parameters, and return types in places where
    /// they're being decoded.
    Decode,
    /// Used when generating the types of operation parameters, and return types in places where
    /// they're being encoded.
    Encode,
    /// Used when generating types that are parts of other types, such as the key & value types of
    /// dictionaries, or the element type of a sequence.
    Nested,
}

pub fn get_bit_sequence_size<T: Member>(members: &[&T]) -> usize {
    members
        .iter()
        .filter(|member| !member.is_tagged() && member.data_type().is_bit_sequence_encodable())
        .count()
}

/// Takes a slice of Member references and returns two vectors. One containing the required members
/// and the other containing the tagged members. The tagged vector is sorted by its tags.
pub fn get_sorted_members<'a, T: Member>(members: &[&'a T]) -> (Vec<&'a T>, Vec<&'a T>) {
    let required_members = members
        .iter()
        .filter(|member| !member.is_tagged())
        .cloned()
        .collect::<Vec<_>>();
    let mut tagged_members = members
        .iter()
        .filter(|member| member.is_tagged())
        .cloned()
        .collect::<Vec<_>>();
    tagged_members.sort_by_key(|member| member.tag().unwrap());

    (required_members, tagged_members)
}

pub fn clone_as_non_optional<T: Element + ?Sized>(type_ref: &TypeRef<T>) -> TypeRef<T> {
    let mut cloned = type_ref.clone();
    cloned.is_optional = false;
    cloned
}
