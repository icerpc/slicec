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
