// Copyright (c) ZeroC, Inc. All rights reserved.

pub mod code_gen_util;
pub mod ptr_util;
pub mod string_util;

pub mod attribute {
    pub const IGNORE_WARNINGS: &str = "ignore_warnings";
    pub const DEPRECATED: &str = "deprecated";
    pub const FORMAT: &str = "format";
    pub const COMPRESS: &str = "compress";
}
