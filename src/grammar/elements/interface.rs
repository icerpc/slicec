// Copyright (c) ZeroC, Inc.

use super::super::*;
use crate::slice_file::Span;
use crate::supported_encodings::SupportedEncodings;
use crate::utils::ptr_util::WeakPtr;

#[derive(Debug)]
pub struct Interface {
    pub identifier: Identifier,
    pub operations: Vec<WeakPtr<Operation>>,
    pub bases: Vec<TypeRef<Interface>>,
    pub scope: Scope,
    pub attributes: Vec<WeakPtr<Attribute>>,
    pub comment: Option<DocComment>,
    pub span: Span,
    pub(crate) supported_encodings: Option<SupportedEncodings>,
}

impl Interface {
    pub fn operations(&self) -> Vec<&Operation> {
        self.operations.iter().map(WeakPtr::borrow).collect()
    }

    pub fn all_inherited_operations(&self) -> Vec<&Operation> {
        let mut operations = self
            .all_base_interfaces()
            .into_iter()
            .flat_map(Interface::operations)
            .collect::<Vec<_>>();

        // Filter duplicates created by diamond inheritance in-place.
        let mut seen_identifiers = std::collections::HashSet::new();
        operations.retain(|op| seen_identifiers.insert(op.parser_scoped_identifier()));

        operations
    }

    pub fn all_operations(&self) -> Vec<&Operation> {
        let mut operations = self.operations();
        operations.extend(self.all_inherited_operations());

        // Filter duplicates created by diamond inheritance in-place.
        let mut seen_identifiers = std::collections::HashSet::new();
        operations.retain(|op| seen_identifiers.insert(op.parser_scoped_identifier()));

        operations
    }

    pub fn base_interfaces(&self) -> Vec<&Interface> {
        self.bases.iter().map(TypeRef::definition).collect()
    }

    pub fn all_base_interfaces(&self) -> Vec<&Interface> {
        let mut all_bases = self.base_interfaces();
        all_bases.extend(self.bases.iter().flat_map(|type_ref| type_ref.all_base_interfaces()));

        // Filter duplicates created by diamond inheritance in-place.
        let mut seen_identifiers = std::collections::HashSet::new();
        all_bases.retain(|base| seen_identifiers.insert(base.parser_scoped_identifier()));

        all_bases
    }
}

impl Type for Interface {
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

    fn supported_encodings(&self) -> SupportedEncodings {
        self.supported_encodings.clone().unwrap()
    }
}

implement_Element_for!(Interface, "interface");
implement_Attributable_for!(Interface);
implement_Entity_for!(Interface);
implement_Commentable_for!(Interface);
implement_Container_for!(Interface, Operation, operations);
