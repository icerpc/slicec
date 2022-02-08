// Copyright (c) ZeroC, Inc. All rights reserved.
// TODO this entire file needs to be looked over again.

use crate::grammar::{Element, Member, Primitive, TypeRef, Types};

/// The context that a type is being used in while generating code. This is used primarily by the
/// `type_to_string` methods in each of the language mapping's code generators.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum TypeContext {
    /// Used when generating the types of data members in structs, classes, and exceptions.
    DataMember,
    /// Used when generating the types of operation parameters, and return types in places where they're being decoded.
    Decode,
    /// Used when generating the types of operation parameters, and return types in places where they're being encoded.
    Encode,
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

pub fn fix_case(s: &str, case: CaseStyle) -> String {
    if s.is_empty() {
        return String::new();
    }

    match case {
        CaseStyle::Camel => camel_case(s),
        CaseStyle::Pascal => pascal_case(s),
        CaseStyle::Snake => snake_case(s),
    }
}

fn camel_case(s: &str) -> String {
    let mut next_is_upper = false;
    s.chars()
        .enumerate()
        .filter_map(|(i, c)| {
            if i == 0 {
                Some(c.to_lowercase().collect::<Vec<_>>())
            } else if c == '_' {
                next_is_upper = true;
                None
            } else if next_is_upper {
                next_is_upper = false;
                Some(c.to_uppercase().collect::<Vec<_>>())
            } else {
                Some(vec![c])
            }
        })
        .flatten()
        .collect::<String>()
}

fn pascal_case(s: &str) -> String {
    let mut next_is_upper = false;
    s.chars()
        .enumerate()
        .filter_map(|(i, c)| {
            if i == 0 {
                Some(c.to_uppercase().collect::<Vec<_>>())
            } else if c == '_' {
                next_is_upper = true;
                None
            } else if next_is_upper {
                next_is_upper = false;
                Some(c.to_uppercase().collect::<Vec<_>>())
            } else {
                Some(vec![c])
            }
        })
        .flatten()
        .collect::<String>()
}

fn snake_case(s: &str) -> String {
    s.chars()
        .enumerate()
        .filter_map(|(i, c)| {
            if c.is_uppercase() {
                let mut chars = vec![];
                if i > 0 {
                    chars.push('_');
                }
                chars.extend(c.to_lowercase());
                Some(chars)
            } else {
                Some(vec![c])
            }
        })
        .flatten()
        .collect::<String>()
}

pub fn get_bit_sequence_size<T: Member>(members: &[&T]) -> usize {
    members
        .iter()
        .filter(|member| member.tag().is_none() && member.data_type().is_bit_sequence_encodable())
        .count()
}

/// Takes a slice of Member references and returns two vectors. One containing the required members
/// and the other containing the tagged members. The tagged vector is sorted by its tags.
pub fn get_sorted_members<'a, T: Member>(members: &[&'a T]) -> (Vec<&'a T>, Vec<&'a T>) {
    let required_members = members
        .iter()
        .filter(|member| member.tag().is_none())
        .cloned()
        .collect::<Vec<_>>();
    let mut tagged_members = members
        .iter()
        .filter(|member| member.tag().is_some())
        .cloned()
        .collect::<Vec<_>>();
    tagged_members.sort_by_key(|member| member.tag().unwrap());

    (required_members, tagged_members)
}

// TODO: move these to grammar/util once we implement slice-driven encoding.
pub fn are_members_11_compatible(members: &[&impl Member], allow_tags: bool) -> bool {
    members.iter().all(|member|
        let is_tagged = member.tag().is_some();
        is_type_11_compatible(member.data_type(), is_tagged) && (!is_tagged || allow_tags)
    )
}

pub fn is_type_11_compatible(type_ref: &TypeRef, is_tagged: bool) -> bool {
    // We don't count tagged types as optional for the 1.1 encoding.
    // Tagged types are always implied to be optional.
    let is_optional = type_ref.is_optional() && !is_tagged;

    match type_ref.concrete_type() {
        Types::Struct(struct_def) => {
            struct_def.is_compact
            && !is_optional
            && are_members_11_compatible(&struct_def.members(), false)
        }
        Types::Class(class_def) => are_members_11_compatible(&class_def.members(), true),
        Types::Interface(_) => true,
        Types::Enum(enum_def) => enum_def.underlying.is_none() && !is_optional,
        Types::Trait(_) => false,
        Types::Sequence(sequence_def) => {
            !is_optional
            && is_type_11_compatible(&sequence_def.element_type)
        }
        Types::Dictionary(dictionary_def) => {
            !is_optional
            && is_type_11_compatible(&dictionary_def.key_type)
            && is_type_11_compatible(&dictionary_def.value_type)
        }
        Types::Primitive(primitive_def) => {
            !is_optional && matches!(primitive_def,
                Primitive::Bool | Primitive::Byte | Primitive::Short | Primitive::Int |
                Primitive::Long | Primitive::Float | Primitive::Double | Primitive::String |
                Primitive::AnyClass
            )
        }
    }
}

pub fn clone_as_non_optional<T: Element + ?Sized>(type_ref: &TypeRef<T>) -> TypeRef<T> {
    let mut cloned = type_ref.clone();
    cloned.is_optional = false;
    cloned
}
