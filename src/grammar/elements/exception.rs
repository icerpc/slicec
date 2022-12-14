// Copyright (c) ZeroC, Inc. All rights reserved.

use super::super::*;
use crate::slice_file::Span;
use crate::supported_encodings::SupportedEncodings;
use crate::utils::ptr_util::WeakPtr;

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
    fn type_string(&self) -> String {
        self.identifier().to_owned()
    }

    fn fixed_wire_size(&self) -> Option<u32> {
        // An exception is fixed size if and only if all its members are fixed size.
        self.all_members()
            .iter()
            .map(|member| member.data_type.fixed_wire_size())
            .collect::<Option<Vec<u32>>>()
            .map(|sizes| sizes.iter().sum())
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
