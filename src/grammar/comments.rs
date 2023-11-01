// Copyright (c) ZeroC, Inc.

use crate::grammar::*;
use crate::slice_file::Span;

#[derive(Debug)]
pub struct DocComment {
    pub overview: Option<Message>,
    pub params: Vec<ParamTag>,
    pub returns: Vec<ReturnsTag>,
    pub throws: Vec<ThrowsTag>,
    pub see: Vec<SeeTag>,
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
    pub thrown_type: TypeRefDefinition<Exception>,
    pub message: Message,
    pub span: Span,
}

impl ThrowsTag {
    pub fn thrown_type(&self) -> Result<&Exception, &Identifier> {
        match &self.thrown_type {
            TypeRefDefinition::Patched(ptr) => Ok(ptr.borrow()),
            TypeRefDefinition::Unpatched(identifier) => Err(identifier),
        }
    }
}

#[derive(Debug)]
pub struct SeeTag {
    pub link: TypeRefDefinition<dyn Entity>,
    pub span: Span,
}

impl SeeTag {
    pub fn linked_entity(&self) -> Result<&dyn Entity, &Identifier> {
        match &self.link {
            TypeRefDefinition::Patched(ptr) => Ok(ptr.borrow()),
            TypeRefDefinition::Unpatched(identifier) => Err(identifier),
        }
    }
}

#[derive(Debug)]
pub struct LinkTag {
    pub link: TypeRefDefinition<dyn Entity>,
    pub span: Span,
}

impl LinkTag {
    pub fn linked_entity(&self) -> Result<&dyn Entity, &Identifier> {
        match &self.link {
            TypeRefDefinition::Patched(ptr) => Ok(ptr.borrow()),
            TypeRefDefinition::Unpatched(identifier) => Err(identifier),
        }
    }
}

#[derive(Debug)]
pub enum MessageComponent {
    Text(String),
    Link(LinkTag),
}

#[derive(Debug)]
pub struct Message {
    pub value: Vec<MessageComponent>,
    pub span: Span,
}

implement_Element_for!(DocComment, "doc comment");
implement_Symbol_for!(DocComment);
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
implement_Element_for!(Message, "doc message");
implement_Symbol_for!(Message);
