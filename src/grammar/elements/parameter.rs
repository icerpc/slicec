// Copyright (c) ZeroC, Inc. All rights reserved.

use super::super::*;
use crate::ptr_util::WeakPtr;
use crate::slice_file::Location;

#[derive(Debug)]
pub struct Parameter {
    pub identifier: Identifier,
    pub data_type: TypeRef,
    pub tag: Option<u32>,
    pub is_streamed: bool,
    pub is_returned: bool,
    pub parent: WeakPtr<Operation>,
    pub scope: Scope,
    pub attributes: Vec<Attribute>,
    pub comment: Option<DocComment>,
    pub location: Location,
}

impl Parameter {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        identifier: Identifier,
        data_type: TypeRef,
        tag: Option<u32>,
        is_streamed: bool,
        is_returned: bool,
        scope: Scope,
        attributes: Vec<Attribute>,
        comment: Option<DocComment>,
        location: Location,
    ) -> Self {
        let parent = WeakPtr::create_uninitialized();
        Parameter {
            identifier,
            data_type,
            tag,
            is_streamed,
            is_returned,
            parent,
            scope,
            attributes,
            comment,
            location,
        }
    }
}

impl Element for Parameter {
    fn kind(&self) -> &'static str {
        if self.is_returned {
            "return element"
        } else {
            "parameter"
        }
    }
}

implement_Entity_for!(Parameter);
implement_Contained_for!(Parameter, Operation);
implement_Member_for!(Parameter);
