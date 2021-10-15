// Copyright (c) ZeroC, Inc. All rights reserved.

use slice::grammar::NamedSymbol;
use slice::util::{fix_case, CaseStyle};

use crate::cs_util::{escape_keyword, fix_scope};
pub trait CsNamedSymbol {
    /// Escapes and returns the definition's identifier, without any scoping.
    /// If the identifier is a C# keyword, a '@' prefix is appended to it.
    fn escape_identifier(&self, case: CaseStyle) -> String;

    /// Escapes and returns the definition's identifier, fully scoped.
    /// If the identifier or any of the scopes are C# keywords, a '@' prefix is appended to them.
    /// Note: The case style is applied to all scope segments, not just the last one.
    ///
    /// If scope is non-empty, this also qualifies the identifier's scope relative to the provided
    /// one.
    fn escape_scoped_identifier(&self, case: CaseStyle, scope: &str) -> String;

    /// The helper name for this NamedSymbol
    fn helper_name(&self, scope: &str) -> String;

    /// The C# namespace of this NamedSymbol
    fn namespace(&self) -> String;

    fn type_id_attribute(&self) -> String;
}

impl<T: NamedSymbol + 'static> CsNamedSymbol for T {
    /// Escapes and returns the definition's identifier, without any scoping.
    /// If the identifier is a C# keyword, a '@' prefix is appended to it.
    fn escape_identifier(&self, case: CaseStyle) -> String {
        escape_keyword(&fix_case(self.identifier(), case))
    }

    /// Escapes and returns the definition's identifier, fully scoped.
    /// If the identifier or any of the scopes are C# keywords, a '@' prefix is appended to them.
    /// Note: The case style is applied to all scope segments, not just the last one.
    ///
    /// If scope is non-empty, this also qualifies the identifier's scope relative to the provided
    /// one.
    fn escape_scoped_identifier(&self, case: CaseStyle, scope: &str) -> String {
        let mut scoped_identifier = String::new();

        // Escape any keywords in the scope identifiers.
        // We skip the first scope segment, since it is always an empty string because all scopes
        // start with '::' (to represent global scope).
        for segment in self.scope().split("::").skip(1) {
            scoped_identifier += &(escape_keyword(&fix_case(segment, case)) + ".");
        }
        scoped_identifier += &self.escape_identifier(case);
        fix_scope(&scoped_identifier, scope)
    }

    /// The helper name for this NamedSymbol
    fn helper_name(&self, scope: &str) -> String {
        self.escape_scoped_identifier(CaseStyle::Pascal, scope) + "Helper"
    }

    /// The C# namespace of this NamedSymbol
    fn namespace(&self) -> String {
        // TODO: check metadata
        // TODO: not all types need to remove just one "::" (we use this currently for operations)
        self.scope().strip_prefix("::").unwrap().replace("::", ".")
    }

    fn type_id_attribute(&self) -> String {
        format!(
            r#"IceRpc.Slice.TypeId("{}::{}")"#,
            self.scope(),
            self.identifier()
        )
    }
}
