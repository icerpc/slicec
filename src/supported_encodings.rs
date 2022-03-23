// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::grammar::SliceEncoding;

/// A struct for storing and computing what Slice encodings a Slice construct supports.
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
    /// then set its supported encodings to this dummy value. Otherwise, it wouldn't have any
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
