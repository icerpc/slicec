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
    pub(crate) fn new(
        identifier: Identifier,
        is_compact: bool,
        scope: Scope,
        attributes: Vec<Attribute>,
        comment: Option<DocComment>,
        span: Span,
    ) -> Self {
        let members = Vec::new();
        let parent = WeakPtr::create_uninitialized();
        let supported_encodings = None; // Patched later by the encoding_patcher.
        Struct {
            identifier,
            members,
            is_compact,
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
}

impl Type for Struct {
    fn type_string(&self) -> String {
        self.identifier().to_owned()
    }

    fn is_fixed_size(&self) -> bool {
        // A struct is fixed size if and only if all its members are fixed size.
        self.members().iter().all(|member| member.data_type.is_fixed_size())
    }

    fn min_wire_size(&self) -> u32 {
        // The min-wire-size of a struct is the min-wire-size of all its members added together.
        let min_wire_size = self
            .members()
            .iter()
            .map(|member| member.data_type.min_wire_size())
            .sum();
        if self.is_compact {
            min_wire_size
        } else {
            // Non-compact structs use an extra byte to encode TagEndMarker.
            min_wire_size + 1
        }
    }

    fn uses_classes(&self) -> bool {
        self.members().iter().any(|member| member.data_type.uses_classes())
    }

    fn is_class_type(&self) -> bool {
        false
    }

    fn tag_format(&self) -> Option<TagFormat> {
        if self.is_fixed_size() {
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
