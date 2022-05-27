// Copyright (c) ZeroC, Inc. All rights reserved.

pub mod attribute;
pub mod class;
pub mod custom_type;
pub mod data_member;
pub mod dictionary;
pub mod r#enum;
pub mod enumerator;
pub mod exception;
pub mod file_encoding;
pub mod identifier;
pub mod interface;
pub mod module;
pub mod operation;
pub mod parameter;
pub mod primitive;
pub mod sequence;
pub mod r#struct;
pub mod r#trait;
pub mod type_alias;
pub mod type_ref;

// Re-export the contents of the grammar submodules directly into the grammar module. This is
// for convenience, so users don't need to worry about the submodule structure while importing.
pub use self::attribute::*;
pub use self::class::*;
pub use self::custom_type::*;
pub use self::data_member::*;
pub use self::dictionary::*;
pub use self::enumerator::*;
pub use self::exception::*;
pub use self::file_encoding::*;
pub use self::identifier::*;
pub use self::interface::*;
pub use self::module::*;
pub use self::operation::*;
pub use self::parameter::*;
pub use self::primitive::*;
pub use self::r#enum::*;
pub use self::r#struct::*;
pub use self::r#trait::*;
pub use self::sequence::*;
pub use self::type_alias::*;
pub use self::type_ref::*;
