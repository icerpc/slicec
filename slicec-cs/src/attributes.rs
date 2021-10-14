// Copyright (c) ZeroC, Inc. All rights reserved.

use slice::grammar::{Class, NamedSymbol};

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

pub fn obsolete_attribute(named_symbol: &dyn NamedSymbol, check_parent: bool) -> Option<String> {
    get_deprecate_reason(named_symbol, check_parent)
        .map(|r| format!(r#"[global::System.Obsolete("{}")]"#, r))
}

pub fn get_deprecate_reason(named_symbol: &dyn NamedSymbol, _check_parent: bool) -> Option<String> {
    // TODO: check parent once we no longer need the ast)

    if let Some(deprecate) = named_symbol.find_attribute("deprecate") {
        match deprecate.as_slice() {
            [] => Some(format!("This {} has been deprecated", named_symbol.kind())),
            _ => Some(deprecate.to_vec().join("\n")),
        }
    // } else if check_parent {
    // get_deprecate_reason(named_symbol.parent(), false)
    } else {
        None
    }
}
