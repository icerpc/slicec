// Copyright (c) ZeroC, Inc.

pub mod attributes;

mod comments;
mod elements;
mod traits;
mod util;
mod wrappers;

// Re-export the contents of the grammar submodules directly into the grammar module. This is
// for convenience, so users don't need to worry about the submodule structure while importing.
pub use self::attributes::AttributeKind;
pub use self::comments::*;
pub use self::elements::*;
pub use self::traits::*;
pub use self::util::*;
pub use self::wrappers::*;
