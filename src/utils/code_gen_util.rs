// Copyright (c) ZeroC, Inc.

use crate::grammar::{Encoding, Member};

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
