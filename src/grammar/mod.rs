// Copyright (c) ZeroC, Inc. All rights reserved.

mod comments;
mod elements;
mod traits;
mod util;
mod wrappers;

// Re-export the contents of the grammar submodules directly into the grammar module. This is
// for convenience, so users don't need to worry about the submodule structure while importing.
pub use self::comments::*;
pub use self::elements::*;
pub use self::traits::*;
pub use self::util::*;
pub use self::wrappers::*;

pub mod attributes {
    pub const COMPRESS: &str = "compress";
    pub const DEPRECATED: &str = "deprecated";
    pub const FORMAT: &str = "format";
    pub const IGNORE_WARNINGS: &str = "ignore_warnings";
    pub const ONEWAY: &str = "oneway";
}
