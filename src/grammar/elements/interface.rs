// Copyright (c) ZeroC, Inc. All rights reserved.

use super::super::*;
use crate::ptr_util::WeakPtr;
use crate::slice_file::Location;
use crate::supported_encodings::SupportedEncodings;

#[derive(Debug)]
pub struct Interface {
    pub identifier: Identifier,
    pub operations: Vec<WeakPtr<Operation>>,
    pub bases: Vec<TypeRef<Interface>>,
    pub parent: WeakPtr<Module>,
    pub scope: Scope,
    pub attributes: Vec<Attribute>,
    pub comment: Option<DocComment>,
    pub location: Location,
    pub(crate) supported_encodings: Option<SupportedEncodings>,
}

impl Interface {
    pub(crate) fn new(
        identifier: Identifier,
        bases: Vec<TypeRef<Interface>>,
        scope: Scope,
        attributes: Vec<Attribute>,
        comment: Option<DocComment>,
        location: Location,
    ) -> Self {
        let operations = Vec::new();
        let parent = WeakPtr::create_uninitialized();
        let supported_encodings = None; // Patched later by the encoding_patcher.
        Interface {
            identifier,
            operations,
            bases,
            parent,
            scope,
            attributes,
            comment,
            location,
            supported_encodings,
        }
    }

    pub(crate) fn add_operation(&mut self, operation: WeakPtr<Operation>) {
        self.operations.push(operation);
    }

    pub fn operations(&self) -> Vec<&Operation> {
        self.operations
            .iter()
            .map(|operation_ptr| operation_ptr.borrow())
            .collect()
    }

    pub fn all_inherited_operations(&self) -> Vec<&Operation> {
        let mut operations = self
            .all_base_interfaces()
            .iter()
            .flat_map(|base_interface| base_interface.operations())
            .collect::<Vec<&Operation>>();

        // Filter duplicates created by diamond inheritance.
        // Dedup only works on sorted collections, so we have to sort the operations first.
        operations.sort_by_key(|operation| &operation.identifier);
        operations.dedup_by_key(|operation| &operation.identifier);
        operations
    }

    pub fn all_operations(&self) -> Vec<&Operation> {
        let mut operations = self.operations();
        operations.extend(self.all_inherited_operations());

        // Filter duplicates created by diamond inheritance.
        // Dedup only works on sorted collections, so we have to sort the operations first.
        operations.sort_by_key(|operation| &operation.identifier);
        operations.dedup_by_key(|operation| &operation.identifier);
        operations
    }

    pub fn base_interfaces(&self) -> Vec<&Interface> {
        self.bases.iter().map(|type_ref| type_ref.definition()).collect()
    }

    pub fn all_base_interfaces(&self) -> Vec<&Interface> {
        let mut bases = self.base_interfaces();
        bases.extend(
            self.bases
                .iter()
                .flat_map(|type_ref| type_ref.all_base_interfaces())
                .collect::<Vec<&Interface>>(),
        );

        // Filter duplicates created by diamond inheritance.
        // Dedup only works on sorted collections, so we have to sort the bases first.
        bases.sort_by_key(|base| base.module_scoped_identifier());
        bases.dedup_by_key(|base| base.module_scoped_identifier());
        bases
    }
}

impl Type for Interface {
    fn is_fixed_size(&self) -> bool {
        false
    }

    fn min_wire_size(&self) -> u32 {
        // Interfaces are passed on the wire as proxies, and the smallest valid proxy (with Slice2)
        // is "/". Taking up 1 byte for the length of the string, and 1 byte for the '/' character.
        // Note the min_wire_size for a Slice1 encoded proxy is 3, but we take the minimum of both.
        2
    }

    fn uses_classes(&self) -> bool {
        false
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
implement_Entity_for!(Interface);
implement_Container_for!(Interface, WeakPtr<Operation>, operations);
implement_Contained_for!(Interface, Module);
