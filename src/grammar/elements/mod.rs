// Copyright (c) ZeroC, Inc.

mod attribute;
mod class;
mod compilation_mode;
mod custom_type;
mod dictionary;
mod r#enum;
mod enumerator;
mod exception;
mod field;
mod identifier;
mod integer;
mod interface;
mod module;
mod operation;
mod parameter;
mod primitive;
mod result;
mod sequence;
mod r#struct;
mod type_alias;
mod type_ref;

// Re-export the grammar elements directly into this module so consumers don't need to think about submodule structure).
pub use self::attribute::*;
pub use self::class::*;
pub use self::compilation_mode::*;
pub use self::custom_type::*;
pub use self::dictionary::*;
pub use self::enumerator::*;
pub use self::exception::*;
pub use self::field::*;
pub use self::identifier::*;
pub use self::integer::*;
pub use self::interface::*;
pub use self::module::*;
pub use self::operation::*;
pub use self::parameter::*;
pub use self::primitive::*;
pub use self::r#enum::*;
pub use self::r#struct::*;
pub use self::result::*;
pub use self::sequence::*;
pub use self::type_alias::*;
pub use self::type_ref::*;
