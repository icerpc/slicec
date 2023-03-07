// Copyright (c) ZeroC, Inc.

use super::super::*;
use crate::slice_file::Span;
use crate::supported_encodings::SupportedEncodings;
use crate::utils::ptr_util::WeakPtr;

#[derive(Debug)]
pub struct TypeAlias {
    pub identifier: Identifier,
    pub underlying: TypeRef,
    pub parent: WeakPtr<Module>,
    pub scope: Scope,
    pub attributes: Vec<Attribute>,
    pub comment: Option<DocComment>,
    pub span: Span,
    pub(crate) supported_encodings: Option<SupportedEncodings>,
}

impl AsTypes for TypeAlias {
    fn concrete_type(&self) -> Types {
        self.underlying.concrete_type()
    }

    fn concrete_type_mut(&mut self) -> TypesMut {
        panic!("This has always been broken apparently");
    }
}

impl Type for TypeAlias {
    fn type_string(&self) -> String {
        self.identifier().to_owned()
    }

    fn fixed_wire_size(&self) -> Option<u32> {
        self.underlying.fixed_wire_size()
    }

    fn is_class_type(&self) -> bool {
        self.underlying.is_class_type()
    }

    fn tag_format(&self) -> Option<TagFormat> {
        self.underlying.tag_format()
    }

    fn supported_encodings(&self) -> SupportedEncodings {
        self.supported_encodings.clone().unwrap()
    }
}

implement_Element_for!(TypeAlias, "type alias");
implement_Entity_for!(TypeAlias);
implement_Contained_for!(TypeAlias, Module);
