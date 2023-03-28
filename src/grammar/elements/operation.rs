// Copyright (c) ZeroC, Inc.

use super::super::*;
use crate::slice_file::Span;
use crate::utils::ptr_util::WeakPtr;

#[derive(Debug)]
pub struct Operation {
    pub identifier: Identifier,
    pub return_type: Vec<WeakPtr<Parameter>>,
    pub parameters: Vec<WeakPtr<Parameter>>,
    pub throws: Throws,
    pub is_idempotent: bool,
    pub encoding: Encoding,
    pub parent: WeakPtr<Interface>,
    pub scope: Scope,
    pub attributes: Vec<Attribute>,
    pub comment: Option<DocComment>,
    pub span: Span,
}

impl Operation {
    pub fn parameters(&self) -> Vec<&Parameter> {
        self.parameters.iter().map(WeakPtr::borrow).collect()
    }

    pub fn return_members(&self) -> Vec<&Parameter> {
        self.return_type.iter().map(WeakPtr::borrow).collect()
    }

    pub fn parameters_and_return_members(&self) -> Vec<&Parameter> {
        let parameters = self.parameters.iter();
        let return_members = self.return_type.iter();

        parameters.chain(return_members).map(WeakPtr::borrow).collect()
    }

    pub fn has_non_streamed_parameters(&self) -> bool {
        // Operations can have at most 1 streamed parameter. So, if it has more than 1 parameter
        // there must be non streamed parameters. Otherwise we check if the 1 parameter is
        // streamed.
        match self.parameters.len() {
            0 => false,
            1 => !self.parameters[0].borrow().is_streamed,
            _ => true,
        }
    }

    pub fn has_non_streamed_return_members(&self) -> bool {
        // Operations can have at most 1 streamed return member. So, if it has more than 1 member
        // there must be non streamed members. Otherwise we check if the 1 member is streamed.
        match self.return_type.len() {
            0 => false,
            1 => !self.return_type[0].borrow().is_streamed,
            _ => true,
        }
    }

    pub fn non_streamed_parameters(&self) -> Vec<&Parameter> {
        self.parameters()
            .into_iter()
            .filter(|parameter| !parameter.is_streamed)
            .collect()
    }

    pub fn non_streamed_return_members(&self) -> Vec<&Parameter> {
        self.return_members()
            .into_iter()
            .filter(|return_member| !return_member.is_streamed)
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

    pub fn compress_arguments(&self) -> bool {
        match self.find_attribute(false, Attribute::match_compress) {
            Some((compress_args, ..)) => compress_args,
            None => false,
        }
    }

    pub fn compress_return(&self) -> bool {
        match self.find_attribute(false, Attribute::match_compress) {
            Some((.., compress_return)) => compress_return,
            None => false,
        }
    }

    pub fn class_format(&self) -> ClassFormat {
        self.find_attribute(true, Attribute::match_class_format)
            .unwrap_or(ClassFormat::Compact)
    }

    pub fn is_oneway(&self) -> bool {
        self.attributes(false)
            .iter()
            .any(|a| matches!(&a.kind, AttributeKind::Oneway))
    }
}

implement_Element_for!(Operation, "operation");
implement_Entity_for!(Operation);
implement_Contained_for!(Operation, Interface);

/// Stores which exceptions an operation can throw.
#[derive(Debug)]
pub enum Throws {
    /// The operation doesn't throw any Slice exceptions.
    None,

    /// The operation can only throw a specific Slice exception.
    /// With Slice1, it can also throw any exceptions derived from it.
    Specific(TypeRef<Exception>),

    /// The operation can throw any Slice exception.
    /// This is only supported by the Slice1 encoding.
    AnyException,
}
