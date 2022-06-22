// Copyright (c) ZeroC, Inc. All rights reserved.

use super::super::*;
use crate::ptr_util::WeakPtr;
use crate::slice_file::Location;

#[derive(Debug)]
pub struct DataMember {
    pub identifier: Identifier,
    pub data_type: TypeRef,
    pub tag: Option<u32>,
    pub parent: WeakPtr<dyn Container<WeakPtr<DataMember>>>,
    pub scope: Scope,
    pub attributes: Vec<Attribute>,
    pub comment: Option<DocComment>,
    pub location: Location,
}

impl DataMember {
    pub(crate) fn new(
        identifier: Identifier,
        data_type: TypeRef,
        tag: Option<u32>,
        scope: Scope,
        attributes: Vec<Attribute>,
        comment: Option<DocComment>,
        location: Location,
    ) -> Self {
        let parent = WeakPtr::create_uninitialized();
        DataMember {
            identifier,
            data_type,
            tag,
            parent,
            scope,
            attributes,
            comment,
            location,
        }
    }
}

implement_Element_for!(DataMember, "data member");
implement_Entity_for!(DataMember);
implement_Contained_for!(DataMember, dyn Container<WeakPtr<DataMember>> + 'static);
implement_Member_for!(DataMember);
