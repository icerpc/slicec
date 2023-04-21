// Copyright (c) ZeroC, Inc.

use crate::grammar::{Encoding, Entity, Member, Message, MessageComponent};

/// The context that a type is being used in while generating code. This is used primarily by the
/// `type_to_string` methods in each of the language mapping's code generators.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum TypeContext {
    /// Used when generating the types of fields in structs, classes, and exceptions.
    Field,
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

pub fn get_bit_sequence_size<T: Member>(encoding: Encoding, members: &[&T]) -> usize {
    if encoding == Encoding::Slice1 {
        return 0;
    }

    members
        .iter()
        .filter(|member| !member.is_tagged() && member.data_type().is_optional)
        .count()
}

/// Takes a slice of Member references and returns two vectors. One containing the required members
/// and the other containing the tagged members. The tagged vector is sorted by its tags.
pub fn get_sorted_members<'a, T: Member + ?Sized>(members: &[&'a T]) -> (Vec<&'a T>, Vec<&'a T>) {
    let (mut tagged, required): (Vec<&T>, Vec<&T>) = members.iter().partition(|member| member.is_tagged());
    tagged.sort_by_key(|member| member.tag().unwrap());
    (required, tagged)
}

pub fn format_message(message: &Message, link_formatter: impl Fn(&dyn Entity) -> String) -> String {
    // Iterate through the components of the message and append them into a string.
    // If the component is text, append it as is. If the component is a link, format it first, then append it.
    message.iter().fold(String::new(), |s, component| match &component {
        MessageComponent::Text(text) => s + text,
        MessageComponent::Link(link_tag) => match link_tag.linked_entity() {
            // If the link is to a valid entity, run the link formatter. Otherwise just use the link's raw text.
            Ok(entity) => s + &link_formatter(entity),
            Err(identifier) => s + &identifier.value,
        },
    })
}
