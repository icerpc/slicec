// Copyright (c) ZeroC, Inc. All rights reserved.

use super::super::*;
use crate::slice_file::Span;
use crate::supported_encodings::SupportedEncodings;
use crate::utils::ptr_util::WeakPtr;

#[derive(Debug)]
pub struct Class {
    pub identifier: Identifier,
    pub members: Vec<WeakPtr<DataMember>>,
    pub compact_id: Option<u32>,
    pub base: Option<TypeRef<Class>>,
    pub parent: WeakPtr<Module>,
    pub scope: Scope,
    pub attributes: Vec<Attribute>,
    pub comment: Option<DocComment>,
    pub span: Span,
    pub(crate) supported_encodings: Option<SupportedEncodings>,
}

impl Class {
    pub fn members(&self) -> Vec<&DataMember> {
        self.members.iter().map(|member_ptr| member_ptr.borrow()).collect()
    }

    pub fn all_inherited_members(&self) -> Vec<&DataMember> {
        self.base_class()
            .iter()
            .flat_map(|base_class| base_class.members())
            .collect::<Vec<_>>()
    }

    pub fn all_members(&self) -> Vec<&DataMember> {
        let mut members = vec![];
        // Recursively add inherited data members from super-classes.
        if let Some(base_class) = self.base_class() {
            members.extend(base_class.all_members());
        }
        members.extend(self.members());
        members
    }

    pub fn base_class(&self) -> Option<&Class> {
        self.base.as_ref().map(|type_ref| type_ref.definition())
    }
}

impl Type for Class {
    fn type_string(&self) -> String {
        self.identifier().to_owned()
    }

    fn is_fixed_size(&self) -> bool {
        false // A class can always be encoded as either a full instance, or just an index.
    }

    fn min_wire_size(&self) -> u32 {
        1 // A class may be encoded as an index instead of an instance, taking up 1 byte.
    }

    fn uses_classes(&self) -> bool {
        true
    }

    fn is_class_type(&self) -> bool {
        true
    }

    fn tag_format(&self) -> Option<TagFormat> {
        Some(TagFormat::Class)
    }

    fn supported_encodings(&self) -> SupportedEncodings {
        self.supported_encodings.clone().unwrap()
    }
}

implement_Element_for!(Class, "class");
implement_Entity_for!(Class);
implement_Container_for!(Class, WeakPtr<DataMember>, members);
implement_Contained_for!(Class, Module);
