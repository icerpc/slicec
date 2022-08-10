// Copyright (c) ZeroC, Inc. All rights reserved.

use super::super::*;
use crate::utils::ptr_util::WeakPtr;
use crate::utils::slice_file::Span;
use crate::utils::supported_encodings::SupportedEncodings;

#[derive(Debug)]
pub struct Exception {
    pub identifier: Identifier,
    pub members: Vec<WeakPtr<DataMember>>,
    pub base: Option<TypeRef<Exception>>,
    pub parent: WeakPtr<Module>,
    pub scope: Scope,
    pub attributes: Vec<Attribute>,
    pub comment: Option<DocComment>,
    pub span: Span,
    pub(crate) supported_encodings: Option<SupportedEncodings>,
}

impl Exception {
    pub(crate) fn new(
        identifier: Identifier,
        base: Option<TypeRef<Exception>>,
        scope: Scope,
        attributes: Vec<Attribute>,
        comment: Option<DocComment>,
        span: Span,
    ) -> Self {
        let members = Vec::new();
        let parent = WeakPtr::create_uninitialized();
        let supported_encodings = None; // Patched later by the encoding_patcher.
        Exception {
            identifier,
            members,
            base,
            parent,
            scope,
            attributes,
            comment,
            span,
            supported_encodings,
        }
    }

    pub(crate) fn add_member(&mut self, member: WeakPtr<DataMember>) {
        self.members.push(member);
    }

    pub fn members(&self) -> Vec<&DataMember> {
        self.members.iter().map(|member_ptr| member_ptr.borrow()).collect()
    }

    pub fn all_inherited_members(&self) -> Vec<&DataMember> {
        self.base_exception()
            .iter()
            .flat_map(|base_exception| base_exception.members())
            .collect::<Vec<_>>()
    }

    pub fn all_members(&self) -> Vec<&DataMember> {
        let mut members = vec![];
        // Recursively add inherited data members from super-exceptions.
        if let Some(base_exception) = self.base_exception() {
            members.extend(base_exception.all_members());
        }
        members.extend(self.members());
        members
    }

    pub fn base_exception(&self) -> Option<&Exception> {
        self.base.as_ref().map(|type_ref| type_ref.definition())
    }
}

impl Type for Exception {
    fn is_fixed_size(&self) -> bool {
        // An exception is fixed size if and only if all its members are fixed size.
        self.all_members().iter().all(|member| member.data_type.is_fixed_size())
    }

    fn min_wire_size(&self) -> u32 {
        // The min-wire-size of an exception is the min-wire-size of all its members added together.
        self.all_members()
            .iter()
            .map(|member| member.data_type.min_wire_size())
            .sum()
    }

    fn uses_classes(&self) -> bool {
        self.all_members().iter().any(|member| member.data_type.uses_classes())
    }

    fn is_class_type(&self) -> bool {
        false
    }

    fn tag_format(&self) -> Option<TagFormat> {
        // Exceptions as a data type are only supported with Slice2, which doesn't use tag formats.
        None
    }

    fn supported_encodings(&self) -> SupportedEncodings {
        self.supported_encodings.clone().unwrap()
    }
}

implement_Element_for!(Exception, "exception");
implement_Entity_for!(Exception);
implement_Container_for!(Exception, WeakPtr<DataMember>, members);
implement_Contained_for!(Exception, Module);
