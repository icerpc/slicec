// Copyright (c) ZeroC, Inc.

use super::super::*;
use crate::slice_file::Span;
use crate::supported_modes::SupportedModes;
use crate::utils::ptr_util::WeakPtr;

#[derive(Debug)]
pub struct CustomType {
    pub identifier: Identifier,
    pub scope: Scope,
    pub attributes: Vec<WeakPtr<Attribute>>,
    pub comment: Option<DocComment>,
    pub span: Span,
    pub(crate) supported_modes: Option<SupportedModes>,
}

impl Type for CustomType {
    fn type_string(&self) -> String {
        self.identifier().to_owned()
    }

    fn fixed_wire_size(&self) -> Option<u32> {
        None
    }

    fn is_class_type(&self) -> bool {
        false
    }

    fn tag_format(&self) -> Option<TagFormat> {
        Some(TagFormat::FSize)
    }

    fn supported_modes(&self) -> SupportedModes {
        self.supported_modes.clone().unwrap()
    }
}

implement_Element_for!(CustomType, "custom type");
implement_Attributable_for!(CustomType);
implement_Entity_for!(CustomType);
implement_Commentable_for!(CustomType);
