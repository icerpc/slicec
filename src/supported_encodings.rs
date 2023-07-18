// Copyright (c) ZeroC, Inc.

use crate::grammar::Encoding;

/// A struct for storing and computing what Slice encodings a Slice construct supports.
#[derive(Clone, Debug)]
pub struct SupportedEncodings(Vec<Encoding>);

impl SupportedEncodings {
    /// Creates a new [SupportedEncodings] with support for the specified encodings.
    ///
    /// # Arguments
    ///
    /// `encodings` - A list of all the encodings to support, in any order.
    pub fn new(mut encodings: Vec<Encoding>) -> Self {
        // Remove duplicate encodings from the vector.
        encodings.sort();
        encodings.dedup();

        SupportedEncodings(encodings)
    }

    /// Returns whether the specified encoding is supported.
    pub fn supports(&self, encoding: Encoding) -> bool {
        self.0.contains(&encoding)
    }

    /// Returns true if there are no supported encodings, and false otherwise.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Removes support for the specified encoding if it's currently supported.
    pub(crate) fn disable(&mut self, encoding: Encoding) {
        self.0.retain(|&e| e != encoding);
    }

    /// Computes the encodings supported by this and the provided [SupportedEncodings], in place.
    pub(crate) fn intersect_with(&mut self, other: &SupportedEncodings) {
        self.0.retain(|encoding| other.0.contains(encoding));
    }

    /// Creates a dummy version of this struct that supports all encodings.
    /// This is used internally by the compiler to avoid emitting redundant error messages.
    ///
    /// For example, if a class is declared in an 'encoding = Slice2' file, we emit an error for it,
    /// then set its supported encodings to this dummy value. Otherwise, it wouldn't have any
    /// supported encodings, causing any types that use it to also have no supported encodings.
    /// This would lead to a cascade of spurious error messages about unsupportable types.
    pub(crate) fn dummy() -> Self {
        SupportedEncodings(vec![Encoding::Slice1, Encoding::Slice2])
    }
}

/// Allows slice syntax to be used with [SupportedEncodings].
/// Example:
/// ```
/// # use slicec::supported_encodings::SupportedEncodings;
/// # use slicec::grammar::Encoding;
/// let encodings = vec![Encoding::Slice1];
/// let supported_encodings = SupportedEncodings::new(encodings);
///
/// match supported_encodings[..] {
///     [] => println!("No supported encodings"),
///     [e] => println!("Only supports {}", e),
///     _ => println!("Supports multiple encodings"),
/// }
/// ```
impl<I: std::slice::SliceIndex<[Encoding]>> std::ops::Index<I> for SupportedEncodings {
    type Output = I::Output;

    fn index(&self, index: I) -> &Self::Output {
        &self.0[index]
    }
}
