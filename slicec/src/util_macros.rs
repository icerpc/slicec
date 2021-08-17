// Copyright (c) ZeroC, Inc. All rights reserved.

/// Compare the TypeRef's underlying type
#[macro_export]
macro_rules! is_underlying_type {
    ($type_ref:expr, $ast:expr, $of_type:path) => {{
        let node = $ast.resolve_index($type_ref.definition.unwrap());
        if let $of_type(_, _) = node {
            true
        } else {
            false
        }
    }};
}
