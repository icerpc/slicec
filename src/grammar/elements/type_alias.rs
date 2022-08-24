// Copyright (c) ZeroC, Inc. All rights reserved.

use super::super::*;
use crate::slice_file::Span;
use crate::supported_encodings::SupportedEncodings;
use crate::utils::ptr_util::WeakPtr;

#[derive(Debug)]
pub struct TypeAlias {
    pub identifier: Identifier,
    pub underlying: TypeRef,
    pub parent: WeakPtr<Module>,
    pub scope: Scope,
    pub attributes: Vec<Attribute>,
    pub comment: Option<DocComment>,
    pub span: Span,
}

impl TypeAlias {
    pub(crate) fn new(
        identifier: Identifier,
        underlying: TypeRef,
        scope: Scope,
        attributes: Vec<Attribute>,
        comment: Option<DocComment>,
        span: Span,
    ) -> Self {
        let parent = WeakPtr::create_uninitialized();
        TypeAlias {
            identifier,
            underlying,
            parent,
            scope,
            attributes,
            comment,
            span,
        }
    }
}

impl AsTypes for TypeAlias {
    fn concrete_type(&self) -> Types {
        self.underlying.concrete_type()
    }

    fn concrete_type_mut(&mut self) -> TypesMut {
        panic!("This has always been broken apparently");
    }
}

impl Type for TypeAlias {
    fn type_string(&self) -> String {
        self.identifier().to_owned()
    }

    // TODO most of these should panic. Since type-aliases are transparent and removed during
    // type-patching, most of these should never actually be called.
    fn is_fixed_size(&self) -> bool {
        self.underlying.is_fixed_size()
    }

    fn min_wire_size(&self) -> u32 {
        self.underlying.min_wire_size()
    }

    fn uses_classes(&self) -> bool {
        self.underlying.uses_classes()
    }

    fn is_class_type(&self) -> bool {
        self.underlying.is_class_type()
    }

    fn tag_format(&self) -> Option<TagFormat> {
        self.underlying.tag_format()
    }

    fn supported_encodings(&self) -> SupportedEncodings {
        self.underlying.supported_encodings()
    }
}

implement_Element_for!(TypeAlias, "type alias");
implement_Entity_for!(TypeAlias);
implement_Contained_for!(TypeAlias, Module);
