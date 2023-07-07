// Copyright (c) ZeroC, Inc.

use crate::grammar::Mode;

/// A struct for storing and computing what Slice modes a Slice construct supports.
#[derive(Clone, Debug)]
pub struct SupportedModes(Vec<Mode>);

impl SupportedModes {
    /// Creates a new [Supportedmodes] with support for the specified modes.
    ///
    /// # Arguments
    ///
    /// `modes` - A list of all the modes to support, in any order.
    pub fn new(mut modes: Vec<Mode>) -> Self {
        // Remove duplicate modes from the vector.
        modes.sort();
        modes.dedup();

        SupportedModes(modes)
    }

    /// Returns whether the specified mode is supported.
    pub fn supports(&self, mode: &Mode) -> bool {
        self.0.contains(mode)
    }

    /// Returns true if there are no supported modes, and false otherwise.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Removes support for the specified mode if it's currently supported.
    pub(crate) fn disable(&mut self, mode: Mode) {
        self.0.retain(|&e| e != mode);
    }

    /// Computes the modes supported by this and the provided [SupportedModes], in place.
    pub(crate) fn intersect_with(&mut self, other: &SupportedModes) {
        self.0.retain(|mode| other.0.contains(mode));
    }

    /// Creates a dummy version of this struct that supports all modes.
    /// This is used internally by the compiler to avoid emitting redundant error messages.
    ///
    /// For example, if a class is declared in an 'mode = Slice2' file, we emit an error for it,
    /// then set its supported modes to this dummy value. Otherwise, it wouldn't have any
    /// supported modes, causing any types that use it to also have no supported modes.
    /// This would lead to a cascade of spurious error messages about unsupportable types.
    pub(crate) fn dummy() -> Self {
        SupportedModes(vec![Mode::Slice1, Mode::Slice2])
    }
}

/// Allows slice syntax to be used with [SupportedModes].
/// Example:
/// ```
/// # use slicec::supported_modes::SupportedModes;
/// # use slicec::grammar::Mode;
/// let modes = vec![Mode::Slice1];
/// let supported_modes = SupportedModes::new(modes);
///
/// match supported_modes[..] {
///     [] => println!("No supported modes"),
///     [e] => println!("Only supports {}", e),
///     _ => println!("Supports multiple modes"),
/// }
/// ```
impl<I: std::slice::SliceIndex<[Mode]>> std::ops::Index<I> for SupportedModes {
    type Output = I::Output;

    fn index(&self, index: I) -> &Self::Output {
        &self.0[index]
    }
}
