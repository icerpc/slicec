// Copyright (c) ZeroC, Inc. All rights reserved.

use super::super::*;
use crate::ptr_util::WeakPtr;
use crate::slice_file::Location;

#[derive(Debug)]
pub struct Operation {
    pub identifier: Identifier,
    pub return_type: Vec<WeakPtr<Parameter>>,
    pub parameters: Vec<WeakPtr<Parameter>>,
    pub is_idempotent: bool,
    pub encoding: Encoding,
    pub parent: WeakPtr<Interface>,
    pub scope: Scope,
    pub attributes: Vec<Attribute>,
    pub comment: Option<DocComment>,
    pub location: Location,
}

impl Operation {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        identifier: Identifier,
        return_type: Vec<WeakPtr<Parameter>>,
        is_idempotent: bool,
        encoding: Encoding,
        scope: Scope,
        attributes: Vec<Attribute>,
        comment: Option<DocComment>,
        location: Location,
    ) -> Self {
        let parameters = Vec::new();
        let parent = WeakPtr::create_uninitialized();
        Operation {
            identifier,
            return_type,
            parameters,
            is_idempotent,
            encoding,
            parent,
            scope,
            attributes,
            comment,
            location,
        }
    }

    pub(crate) fn add_parameter(&mut self, parameter: WeakPtr<Parameter>) {
        self.parameters.push(parameter);
    }

    pub fn parameters(&self) -> Vec<&Parameter> {
        self.parameters
            .iter()
            .map(|parameter_ptr| parameter_ptr.borrow())
            .collect()
    }

    pub fn return_members(&self) -> Vec<&Parameter> {
        self.return_type
            .iter()
            .map(|parameter_ptr| parameter_ptr.borrow())
            .collect()
    }

    pub fn parameters_and_return_members(&self) -> Vec<&Parameter> {
        let parameters = self.parameters.iter();
        let return_members = self.return_type.iter();

        parameters
            .chain(return_members)
            .map(|parameter_ptr| parameter_ptr.borrow())
            .collect()
    }

    pub fn has_nonstreamed_parameters(&self) -> bool {
        // Operations can have at most 1 streamed parameter. So, if it has more than 1 parameter
        // there must be non streamed parameters. Otherwise we check if the 1 parameter is
        // streamed.
        match self.parameters.len() {
            0 => false,
            1 => !self.parameters[0].borrow().is_streamed,
            _ => true,
        }
    }

    pub fn has_nonstreamed_return_members(&self) -> bool {
        // Operations can have at most 1 streamed return member. So, if it has more than 1 member
        // there must be non streamed members. Otherwise we check if the 1 member is streamed.
        match self.return_type.len() {
            0 => false,
            1 => !self.return_type[0].borrow().is_streamed,
            _ => true,
        }
    }

    pub fn nonstreamed_parameters(&self) -> Vec<&Parameter> {
        self.parameters()
            .iter()
            .filter(|parameter| !parameter.is_streamed)
            .cloned()
            .collect()
    }

    pub fn nonstreamed_return_members(&self) -> Vec<&Parameter> {
        self.return_members()
            .iter()
            .filter(|parameter| !parameter.is_streamed)
            .cloned()
            .collect()
    }

    pub fn streamed_parameter(&self) -> Option<&Parameter> {
        // There can be only 1 streamed parameter and it must be the last parameter.
        self.parameters()
            .last()
            .filter(|parameter| parameter.is_streamed)
            .cloned()
    }

    pub fn streamed_return_member(&self) -> Option<&Parameter> {
        // There can be only 1 streamed return member and it must be the last member.
        self.return_members()
            .last()
            .filter(|parameter| parameter.is_streamed)
            .cloned()
    }

    pub fn sends_classes(&self) -> bool {
        self.parameters()
            .iter()
            .any(|parameter| parameter.data_type.uses_classes())
    }

    pub fn returns_classes(&self) -> bool {
        self.return_members()
            .iter()
            .any(|parameter| parameter.data_type.uses_classes())
    }

    pub fn compress_arguments(&self) -> bool {
        if let Some(attribute) = self.get_attribute("compress", false) {
            attribute.contains(&"Args".to_owned())
        } else {
            false
        }
    }

    pub fn compress_return(&self) -> bool {
        if let Some(attribute) = self.get_attribute("compress", false) {
            attribute.contains(&"Return".to_owned())
        } else {
            false
        }
    }

    pub fn class_format(&self) -> ClassFormat {
        if let Some(format) = self.get_attribute("format", true) {
            match format[0].as_str() {
                "Compact" => ClassFormat::Compact,
                "Sliced" => ClassFormat::Sliced,
                _ => panic!("unknown format type"),
            }
        } else {
            // Compact is the default format for classes.
            ClassFormat::Compact
        }
    }

    pub fn is_oneway(&self) -> bool {
        self.has_attribute("oneway", false)
    }
}

implement_Element_for!(Operation, "operation");
implement_Entity_for!(Operation);
implement_Contained_for!(Operation, Interface);
