// Copyright (c) ZeroC, Inc. All rights reserved.
// TODO this entire file needs to be looked over again.

use crate::grammar::{Element, Member, SliceEncoding, TypeRef};

/// A struct for storing and computing what Slice encodings a Slice entity supports.
#[derive(Clone, Debug)]
pub struct SupportedEncodings(Vec<SliceEncoding>);

impl SupportedEncodings {
    /// Creates a new [SupportedEncodings] with support for the specified encodings.
    ///
    /// # Arguments
    ///
    /// `encodings` - A list of all the encodings to support.
    /// It's allowed to have duplicate entries, and the ordering of entries doesn't matter.
    pub fn new(encodings: Vec<SliceEncoding>) -> Self {
        SupportedEncodings(encodings)
    }

    /// Returns whether the specified encoding is supported.
    pub fn supports(&self, encoding: &SliceEncoding) -> bool {
        self.0.contains(encoding)
    }

    /// Returns whether the Slice 1.1 encoding is supported.
    pub fn supports_11(&self) -> bool {
        self.supports(&SliceEncoding::Slice11)
    }

    /// Returns whether the Slice 2 encoding is supported.
    pub fn supports_2(&self) -> bool {
        self.supports(&SliceEncoding::Slice2)
    }

    /// Returns true if there are multiple supported encodings, and false otherwise.
    pub fn supports_multiple_encodings(&self) -> bool {
        self.0.len() > 1
    }

    /// Returns true if there are no supported encodings, and false otherwise.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Removes support for the Slice 1.1 encoding if it's currently supported.
    pub(crate) fn disable_11(&mut self) {
        self.0.retain(|&encoding| encoding != SliceEncoding::Slice11);
    }

    /// Removes support for the Slice 2 encoding if it's currently supported.
    pub(crate) fn disable_2(&mut self) {
        self.0.retain(|&encoding| encoding != SliceEncoding::Slice2);
    }

    /// Computes the encodings supported by this and the provided [SupportedEncodings], in place.
    pub(crate) fn intersect_with(&mut self, other: &SupportedEncodings) {
        self.0.retain(|encoding| other.0.contains(encoding));
    }

    /// Creates a dummy version of this struct that supports all encodings.
    /// This is used internally by the compiler to avoid emitting redundant error messages.
    ///
    /// For example, if a class is declared in an 'encoding = 2' file, we emit an error for it,
    /// then set it's supported encodings to this dummy value. Otherwise, it wouldn't have any
    /// supported encodings, causing any types that use it to also have no supported encodings.
    /// This would lead to a cascade of spurious error messages about unsupportable types.
    pub(crate) fn dummy() -> Self {
        SupportedEncodings(vec![
            SliceEncoding::Slice11,
            SliceEncoding::Slice2,
        ])
    }
}

/// Allows slice syntax to be used with [SupportedEncodings].
/// Example:
/// ```
/// # use slice::code_gen_util::SupportedEncodings;
/// # use slice::grammar::SliceEncoding;
/// let encodings = vec![SliceEncoding::Slice11];
/// let supported_encodings = SupportedEncodings::new(encodings);
///
/// match supported_encodings[..] {
///     [] => println!("No supported encodings"),
///     [e] => println!("Only supports {}", e),
///     _ => println!("Supports multiple encodings")
/// }
/// ```
impl<I: std::slice::SliceIndex<[SliceEncoding]>> std::ops::Index<I> for SupportedEncodings {
    type Output = I::Output;

    fn index(&self, index: I) -> &Self::Output {
        &self.0[index]
    }
}

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
        .map(|(i, c)| {
            if c.is_uppercase() {
                let mut chars = vec![];
                if i > 0 {
                    chars.push('_');
                }
                chars.extend(c.to_lowercase());
                chars
            } else {
                vec![c]
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

pub fn clone_as_non_optional<T: Element + ?Sized>(type_ref: &TypeRef<T>) -> TypeRef<T> {
    let mut cloned = type_ref.clone();
    cloned.is_optional = false;
    cloned
}
