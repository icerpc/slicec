// Copyright (c) ZeroC, Inc. All rights reserved.

use super::super::*;
use crate::slice_file::Location;

#[derive(Clone, Debug)]
pub struct FileEncoding {
    pub version: Encoding,
    pub location: Location,
}

implement_Element_for!(FileEncoding, "file encoding");
implement_Symbol_for!(FileEncoding);
