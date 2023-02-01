// Copyright (c) ZeroC, Inc. All rights reserved.

// TODO Add comments everywhere!

use crate::grammar::{implement_Element_for, implement_Symbol_for, Element, Entity, Identifier, Symbol};
use crate::slice_file::Span;
use crate::utils::ptr_util::WeakPtr;

#[derive(Debug)]
pub struct DocComment {
    pub overview: Option<Overview>,
    pub params: Vec<ParamTag>,
    pub returns: Vec<ReturnsTag>,
    pub throws: Vec<ThrowsTag>,
    pub see: Vec<SeeTag>,
    pub span: Span,
}

#[derive(Debug)]
pub struct Overview {
    pub message: Message,
    pub span: Span,
}

#[derive(Debug)]
pub struct ParamTag {
    pub identifier: Identifier,
    pub message: Message,
    pub span: Span,
}

#[derive(Debug)]
pub struct ReturnsTag {
    pub identifier: Option<Identifier>,
    pub message: Message,
    pub span: Span,
}

#[derive(Debug)]
pub struct ThrowsTag {
    pub identifier: Option<Identifier>,
    pub message: Message,
    pub span: Span,
}

#[derive(Debug)]
pub struct SeeTag {
    pub link: Identifier,
    pub definition: LinkDefinition,
    pub span: Span,
}

impl SeeTag {
    pub fn linked_entity(&self) -> Option<&dyn Entity> {
        match &self.definition {
            LinkDefinition::Patched(ptr) => Some(ptr.borrow()),
            LinkDefinition::Unpatched => None,
        }
    }
}

#[derive(Debug)]
pub struct LinkTag {
    pub link: Identifier,
    pub definition: LinkDefinition,
    pub span: Span,
}

impl LinkTag {
    pub fn linked_entity(&self) -> Option<&dyn Entity> {
        match &self.definition {
            LinkDefinition::Patched(ptr) => Some(ptr.borrow()),
            LinkDefinition::Unpatched => None,
        }
    }
}

#[derive(Debug)]
pub enum LinkDefinition {
    Patched(WeakPtr<dyn Entity>),
    Unpatched,
}

#[derive(Debug)]
pub enum MessageComponent {
    Text(String),
    Link(LinkTag),
}

pub type Message = Vec<MessageComponent>;

implement_Element_for!(DocComment, "doc comment");
implement_Symbol_for!(DocComment);
implement_Element_for!(Overview, "overview");
implement_Symbol_for!(Overview);
implement_Element_for!(ParamTag, "param tag");
implement_Symbol_for!(ParamTag);
implement_Element_for!(ReturnsTag, "returns tag");
implement_Symbol_for!(ReturnsTag);
implement_Element_for!(ThrowsTag, "throws tag");
implement_Symbol_for!(ThrowsTag);
implement_Element_for!(SeeTag, "see tag");
implement_Symbol_for!(SeeTag);
implement_Element_for!(LinkTag, "link tag");
implement_Symbol_for!(LinkTag);
