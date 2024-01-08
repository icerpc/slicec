// Copyright (c) ZeroC, Inc.

#![no_std]

#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "slice2")]
pub mod slice2;

#[cfg(feature = "slice1")]
pub mod slice1;

pub mod buf_io;
pub mod decoder;
pub mod encoder;
pub mod try_decode;
pub mod try_encode;

mod decoding;
mod encoding;

use decoder::Decoder;
use encoder::Encoder;

use core::fmt::Debug;

pub trait Encoding: Clone + Copy + Debug + Default + PartialEq + Eq {
    fn try_decode_size(decoder: &mut Decoder<Self>) -> Result<usize>;
    fn try_encode_size(size: usize, encoder: &mut Encoder<Self>) -> Result<()>;
}

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    InvalidData {
        desc: &'static str,
    },

    IllegalValue {
        value: i128,
        desc: &'static str,
    },

    OutOfRange {
        value: i128,
        min: i128,
        max: i128,
        typename: &'static str,
    },

    HeapAllocationLimitReached {
        limit: usize,
        current: usize,
        requested: usize,
    },

    EndOfBuffer {
        attempted: usize,
        remaining: usize,
    },
}
