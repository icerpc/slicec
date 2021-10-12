// Copyright (c) ZeroC, Inc. All rights reserved.

use slice::grammar::Class;
use slice::grammar::NamedSymbol;

pub fn type_id_attribute(named_symbol: &dyn NamedSymbol) -> String {
    format!(
        r#"IceRpc.Slice.TypeId("{}::{}")"#,
        named_symbol.scope(),
        named_symbol.identifier()
    )
}

pub fn compact_id_attribute(class_def: &Class) -> Option<String> {
    class_def
        .compact_id
        .map(|id| format!("IceRpc.Slice.CompactTypeId({})", id))
}

pub fn custom_attributes(named_symbol: &dyn NamedSymbol) -> Vec<String> {
    if let Some(attributes) = named_symbol.find_attribute("cs:attribute") {
        attributes.to_vec()
    } else {
        vec![]
    }
}
