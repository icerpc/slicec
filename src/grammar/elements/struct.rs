// Copyright (c) ZeroC, Inc. All rights reserved.

use super::super::*;
use crate::slice_file::Span;
use crate::supported_encodings::SupportedEncodings;
use crate::utils::ptr_util::WeakPtr;

#[derive(Debug)]
pub struct Struct {
    pub identifier: Identifier,
    pub members: Vec<WeakPtr<DataMember>>,
    pub is_compact: bool,
    pub parent: WeakPtr<Module>,
    pub scope: Scope,
    pub attributes: Vec<Attribute>,
    pub comment: Option<DocComment>,
    pub span: Span,
    pub(crate) supported_encodings: Option<SupportedEncodings>,
}

impl Struct {
    pub fn members(&self) -> Vec<&DataMember> {
        self.members.iter().map(|member_ptr| member_ptr.borrow()).collect()
    }
}

impl Type for Struct {
    fn type_string(&self) -> String {
        self.identifier().to_owned()
    }

    fn fixed_wire_size(&self) -> Option<u32> {
        // A struct is fixed size if and only if all its members are fixed size.
        self.members()
            .iter()
            .map(|member| member.data_type.fixed_wire_size())
            .collect::<Option<Vec<u32>>>()
            .map(|sizes| sizes.iter().sum())
            .map(|size: u32| size + if self.is_compact { 0 } else { 1 })
    }

    fn is_class_type(&self) -> bool {
        false
    }

    fn tag_format(&self) -> Option<TagFormat> {
        if self.fixed_wire_size().is_some() {
            Some(TagFormat::VSize)
        } else {
            Some(TagFormat::FSize)
        }
    }

    fn supported_encodings(&self) -> SupportedEncodings {
        self.supported_encodings.clone().unwrap()
    }
}

implement_Element_for!(Struct, "struct");
implement_Entity_for!(Struct);
implement_Container_for!(Struct, WeakPtr<DataMember>, members);
implement_Contained_for!(Struct, Module);
