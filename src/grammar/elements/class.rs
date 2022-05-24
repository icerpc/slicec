// Copyright (c) ZeroC, Inc. All rights reserved.

use super::super::*;
use crate::ptr_util::{OwnedPtr, WeakPtr};
use crate::slice_file::Location;
use crate::supported_encodings::SupportedEncodings;

#[derive(Debug)]
pub struct Class {
    pub identifier: Identifier,
    pub members: Vec<OwnedPtr<DataMember>>,
    pub compact_id: Option<u32>,
    pub base: Option<TypeRef<Class>>,
    pub parent: WeakPtr<Module>,
    pub scope: Scope,
    pub attributes: Vec<Attribute>,
    pub comment: Option<DocComment>,
    pub location: Location,
    pub(crate) supported_encodings: Option<SupportedEncodings>,
}

impl Class {
    pub(crate) fn new(
        identifier: Identifier,
        compact_id: Option<u32>,
        base: Option<TypeRef<Class>>,
        scope: Scope,
        attributes: Vec<Attribute>,
        comment: Option<DocComment>,
        location: Location,
    ) -> Self {
        let members = Vec::new();
        let parent = WeakPtr::create_uninitialized();
        let supported_encodings = None; // Patched later by the encoding_patcher.
        Class {
            identifier,
            compact_id,
            members,
            base,
            parent,
            scope,
            attributes,
            comment,
            location,
            supported_encodings,
        }
    }

    pub(crate) fn add_member(&mut self, member: DataMember) {
        self.members.push(OwnedPtr::new(member));
    }

    pub fn members(&self) -> Vec<&DataMember> {
        self.members
            .iter()
            .map(|member_ptr| member_ptr.borrow())
            .collect()
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
implement_Container_for!(Class, OwnedPtr<DataMember>, members);
implement_Contained_for!(Class, Module);
