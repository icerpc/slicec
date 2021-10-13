// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::ast::{Ast, Node};
use crate::ref_from_node;
use crate::slice_file::Location;
use crate::tag_format::TagFormat;

/// The lowest base trait in the compiler, which all symbols and types implement.
pub trait Element {
    fn kind(&self) -> &'static str;
}

macro_rules! implement_element_for {
    ($a:ty, $b:literal) => {
        impl Element for $a {
            fn kind(&self) -> &'static str {
                $b
            }
        }
    };
}

implement_element_for!(Module, "module");
implement_element_for!(Struct, "struct");
implement_element_for!(Class, "class");
implement_element_for!(Exception, "exception");
implement_element_for!(Interface, "interface");
implement_element_for!(Enum, "enum");
implement_element_for!(Operation, "operation");
// Member has its own custom implementation of Element which depends on its member type.
implement_element_for!(Enumerator, "enumerator");
implement_element_for!(Identifier, "identifier");
implement_element_for!(TypeRef, "type ref");
implement_element_for!(Sequence, "sequence");
implement_element_for!(Dictionary, "dictionary");
// Primitive has its own custom implementation of Element which returns the primitive's type name.
implement_element_for!(Attribute, "attribute");
implement_element_for!(DocComment, "comment");

/// Symbols represent elements of the actual source code written in the slice file.
pub trait Symbol: Element {
    fn location(&self) -> &Location;
}

macro_rules! implement_symbol_for {
    ($a:ty) => {
        impl Symbol for $a {
            fn location(&self) -> &Location {
                &self.location
            }
        }
    };
}

implement_symbol_for!(Module);
implement_symbol_for!(Struct);
implement_symbol_for!(Class);
implement_symbol_for!(Exception);
implement_symbol_for!(Interface);
implement_symbol_for!(Enum);
implement_symbol_for!(Operation);
implement_symbol_for!(Member);
implement_symbol_for!(Enumerator);
implement_symbol_for!(Identifier);
implement_symbol_for!(TypeRef);
implement_symbol_for!(Attribute);
implement_symbol_for!(DocComment);

/// Scoped symbols are symbols that are sensitive to their enclosing scopes.
/// These also support having attributes placed on them, and provide methods for handling them.
pub trait ScopedSymbol: Symbol {
    fn attributes(&self) -> &Vec<Attribute>;
    fn find_attribute(&self, directive: &str) -> Option<&Vec<String>>;
    fn has_attribute(&self, directive: &str) -> bool;
    // TODOAUSTIN re-implement to discard parser scopes to make code generation easier.
    // EX: datamembers shouldn't have their struct as a scope in C#. That's purely for parsing.
    fn scope(&self) -> &str;
}

macro_rules! implement_scoped_symbol_for {
    ($a:ty) => {
        impl ScopedSymbol for $a {
            fn attributes(&self) -> &Vec<Attribute> {
                &self.attributes
            }

            /// Checks if the symbol has the specified attribute, and if so, returns it's
            /// arguments as a string vector. If it doesn't, it returns 'None'.
            fn find_attribute(&self, directive: &str) -> Option<&Vec<String>> {
                for m in &self.attributes {
                    if m.qualified_directive == directive {
                        return Some(&m.arguments);
                    }
                }
                return None;
            }

            /// Returns true if the symbol has the specified attribute on it,
            /// and false otherwise.
            fn has_attribute(&self, directive: &str) -> bool {
                self.find_attribute(directive).is_some()
            }

            fn scope(&self) -> &str {
                self.scope.as_ref().unwrap()
            }
        }
    };
}

implement_scoped_symbol_for!(Module);
implement_scoped_symbol_for!(Struct);
implement_scoped_symbol_for!(Class);
implement_scoped_symbol_for!(Exception);
implement_scoped_symbol_for!(Interface);
implement_scoped_symbol_for!(Enum);
implement_scoped_symbol_for!(Operation);
implement_scoped_symbol_for!(Member);
implement_scoped_symbol_for!(Enumerator);
implement_scoped_symbol_for!(TypeRef);

/// NamedSymbols are scoped symbols that have an identifier attached to them.
pub trait NamedSymbol: ScopedSymbol {
    fn identifier(&self) -> &str;
    fn comment(&self) -> Option<&DocComment>;
}

macro_rules! implement_named_symbol_for {
    ($a:ty) => {
        impl NamedSymbol for $a {
            fn identifier(&self) -> &str {
                &self.identifier.value
            }

            fn comment(&self) -> Option<&DocComment> {
                self.comment.as_ref()
            }
        }
    };
}

implement_named_symbol_for!(Module);
implement_named_symbol_for!(Struct);
implement_named_symbol_for!(Class);
implement_named_symbol_for!(Exception);
implement_named_symbol_for!(Interface);
implement_named_symbol_for!(Enum);
implement_named_symbol_for!(Operation);
implement_named_symbol_for!(Member);
implement_named_symbol_for!(Enumerator);

/// Base trait that all elements representing types implement.
pub trait Type {
    fn is_fixed_size(&self, ast: &Ast) -> bool;
    fn min_wire_size(&self, ast: &Ast) -> u32;
    fn tag_format(&self, ast: &Ast) -> TagFormat;
    fn uses_classes(&self, ast: &Ast) -> bool;
}

#[derive(Clone, Debug)]
pub struct Module {
    pub identifier: Identifier,
    pub contents: Vec<usize>,
    pub scope: Option<String>,
    pub location: Location,
    pub attributes: Vec<Attribute>,
    pub comment: Option<DocComment>,
}

impl Module {
    pub fn new(
        identifier: Identifier,
        contents: Vec<usize>,
        attributes: Vec<Attribute>,
        comment: Option<DocComment>,
        location: Location,
    ) -> Self {
        Module { identifier, contents, scope: None, attributes, comment, location }
    }

    pub fn contents<'a>(&self, ast: &'a Ast) -> Vec<&'a dyn NamedSymbol> {
        self.contents
            .iter()
            .map(|id| ast.resolve_index(*id).as_named_symbol().unwrap())
            .collect()
    }

    pub fn is_top_level(&self) -> bool {
        self.scope() == "::"
    }
}

#[derive(Clone, Debug)]
pub struct Struct {
    pub identifier: Identifier,
    pub members: Vec<usize>,
    pub scope: Option<String>,
    pub attributes: Vec<Attribute>,
    pub comment: Option<DocComment>,
    pub location: Location,
}

impl Struct {
    pub fn new(
        identifier: Identifier,
        members: Vec<usize>,
        attributes: Vec<Attribute>,
        comment: Option<DocComment>,
        location: Location,
    ) -> Self {
        Struct { identifier, members, scope: None, attributes, comment, location }
    }

    pub fn members<'a>(&self, ast: &'a Ast) -> Vec<&'a Member> {
        self.members
            .iter()
            .map(|id| ref_from_node!(Node::Member, ast, *id))
            .collect()
    }
}

impl Type for Struct {
    fn is_fixed_size(&self, ast: &Ast) -> bool {
        for member in self.members(ast) {
            let data_type = ast
                .resolve_index(member.data_type.definition.unwrap())
                .as_type();
            if !data_type.unwrap().is_fixed_size(ast) {
                return false;
            }
        }
        true
    }

    fn min_wire_size(&self, ast: &Ast) -> u32 {
        let mut size = 0;
        for member in self.members(ast) {
            size += member
                .data_type
                .definition(ast)
                .as_type()
                .unwrap()
                .min_wire_size(ast);
        }
        size
    }

    fn tag_format(&self, ast: &Ast) -> TagFormat {
        if self.is_fixed_size(ast) {
            TagFormat::VSize
        } else {
            TagFormat::FSize
        }
    }

    fn uses_classes(&self, ast: &Ast) -> bool {
        self.members(ast).iter().any(|m| {
            m.data_type
                .definition(ast)
                .as_type()
                .unwrap()
                .uses_classes(ast)
        })
    }
}

#[derive(Clone, Debug)]
pub struct Class {
    pub identifier: Identifier,
    pub members: Vec<usize>,
    pub base: Option<TypeRef>,
    pub scope: Option<String>,
    pub attributes: Vec<Attribute>,
    pub comment: Option<DocComment>,
    pub location: Location,
    pub compact_id: Option<u32>,
}

impl Class {
    pub fn new(
        identifier: Identifier,
        members: Vec<usize>,
        base: Option<TypeRef>,
        attributes: Vec<Attribute>,
        comment: Option<DocComment>,
        location: Location,
    ) -> Self {
        Class {
            identifier,
            members,
            base,
            scope: None,
            attributes,
            comment,
            location,
            compact_id: None, // TODO: parse compact id
        }
    }

    pub fn all_data_members<'a>(&self, ast: &'a Ast) -> Vec<&'a Member> {
        let mut members = self.members(ast);

        if let Some(base) = &self.base {
            let mut base_members = ref_from_node!(Node::Class, ast, base.definition.unwrap())
                .all_data_members(ast);

            members.append(&mut base_members);
        }

        members
    }

    pub fn members<'a>(&self, ast: &'a Ast) -> Vec<&'a Member> {
        self.members
            .iter()
            .map(|id| ref_from_node!(Node::Member, ast, *id))
            .collect()
    }

    pub fn base<'a>(&self, ast: &'a Ast) -> Option<&'a Class> {
        match self.base {
            Some(ref base) => Some(ref_from_node!(
                Node::Class,
                ast,
                base.definition.unwrap()
            )),
            None => None,
        }
    }
}

impl Type for Class {
    fn is_fixed_size(&self, ast: &Ast) -> bool {
        false
    }

    fn min_wire_size(&self, ast: &Ast) -> u32 {
        1
    }

    fn tag_format(&self, _: &Ast) -> TagFormat {
        TagFormat::Class
    }

    fn uses_classes(&self, ast: &Ast) -> bool {
        true
    }
}

#[derive(Clone, Debug)]
pub struct Exception {
    pub identifier: Identifier,
    pub members: Vec<usize>,
    pub base: Option<TypeRef>,
    pub scope: Option<String>,
    pub attributes: Vec<Attribute>,
    pub comment: Option<DocComment>,
    pub location: Location,
}

impl Exception {
    pub fn new(
        identifier: Identifier,
        members: Vec<usize>,
        base: Option<TypeRef>,
        attributes: Vec<Attribute>,
        comment: Option<DocComment>,
        location: Location,
    ) -> Self {
        Exception { identifier, members, base, scope: None, attributes, comment, location }
    }

    pub fn base<'a>(&self, ast: &'a Ast) -> Option<&'a Exception> {
        match self.base {
            Some(ref base) => Some(ref_from_node!(
                Node::Exception,
                ast,
                base.definition.unwrap()
            )),
            None => None,
        }
    }

    pub fn all_data_members<'a>(&self, ast: &'a Ast) -> Vec<&'a Member> {
        let mut members = self.members(ast);

        if let Some(base) = &self.base {
            let mut base_members = ref_from_node!(Node::Exception, ast, base.definition.unwrap())
                .all_data_members(ast);

            members.append(&mut base_members);
        }

        members
    }

    pub fn members<'a>(&self, ast: &'a Ast) -> Vec<&'a Member> {
        self.members
            .iter()
            .map(|id| ref_from_node!(Node::Member, ast, *id))
            .collect()
    }

    // TODO: Since Exception doesn't implement the Type trait we need to implement this manually.
    // It would be nice to have a shared trait (MemberHolder)
    pub fn uses_classes(&self, ast: &Ast) -> bool {
        self.all_data_members(ast).iter().any(|m| {
            m.data_type
                .definition(ast)
                .as_type()
                .unwrap()
                .uses_classes(ast)
        })
    }
}

#[derive(Clone, Debug)]
pub struct Interface {
    pub identifier: Identifier,
    pub operations: Vec<usize>,
    pub bases: Vec<TypeRef>,
    pub scope: Option<String>,
    pub attributes: Vec<Attribute>,
    pub comment: Option<DocComment>,
    pub location: Location,
}

impl Interface {
    pub fn new(
        identifier: Identifier,
        operations: Vec<usize>,
        bases: Vec<TypeRef>,
        attributes: Vec<Attribute>,
        comment: Option<DocComment>,
        location: Location,
    ) -> Self {
        Interface {
            identifier,
            operations,
            bases,
            scope: None,
            attributes,
            comment,
            location,
        }
    }

    pub fn all_bases<'a>(&self, ast: &'a Ast) -> Vec<&'a Interface> {
        let mut bases = self
            .bases(ast)
            .iter()
            .flat_map(|base| base.bases(ast))
            .collect::<Vec<_>>();

        bases.sort_by_key(|b| b.scoped_identifier());
        bases.dedup_by_key(|b| b.scoped_identifier());

        bases
    }

    pub fn all_base_operations<'a>(&self, ast: &'a Ast) -> Vec<&'a Operation> {
        let mut operations = self
            .all_bases(ast)
            .iter()
            .map(|base| base.operations(ast))
            .flatten()
            .collect::<Vec<_>>();

        operations.sort_by_key(|op| op.identifier());
        operations.dedup_by_key(|op| op.identifier());

        operations
    }

    pub fn all_operations<'a>(&self, ast: &'a Ast) -> Vec<&'a Operation> {
        let mut operations = self.all_base_operations(ast);
        operations.extend_from_slice(&self.operations(ast));
        operations.sort_by_key(|op| op.identifier());
        operations.dedup_by_key(|op| op.identifier());

        operations
    }

    pub fn bases<'a>(&self, ast: &'a Ast) -> Vec<&'a Interface> {
        self.bases
            .iter()
            .map(|base| ref_from_node!(Node::Interface, ast, base.definition.unwrap()))
            .collect()
    }

    pub fn operations<'a>(&self, ast: &'a Ast) -> Vec<&'a Operation> {
        self.operations
            .iter()
            .map(|id| ref_from_node!(Node::Operation, ast, *id))
            .collect()
    }

    pub fn scoped_identifier(&self) -> String {
        self.scope.clone().unwrap() + "::" + &self.identifier()
    }
}

impl Type for Interface {
    fn is_fixed_size(&self, _: &Ast) -> bool {
        false
    }

    fn min_wire_size(&self, _: &Ast) -> u32 {
        3
    }

    fn tag_format(&self, _: &Ast) -> TagFormat {
        TagFormat::FSize
    }

    fn uses_classes(&self, _: &Ast) -> bool {
        false
    }
}

#[derive(Clone, Debug)]
pub struct Enum {
    pub identifier: Identifier,
    pub enumerators: Vec<usize>,
    pub is_unchecked: bool,
    pub underlying: Option<TypeRef>,
    pub scope: Option<String>,
    pub attributes: Vec<Attribute>,
    pub comment: Option<DocComment>,
    pub location: Location,
}

impl Enum {
    pub fn new(
        identifier: Identifier,
        enumerators: Vec<usize>,
        is_unchecked: bool,
        underlying: Option<TypeRef>,
        attributes: Vec<Attribute>,
        comment: Option<DocComment>,
        location: Location,
    ) -> Self {
        Enum {
            identifier,
            enumerators,
            is_unchecked,
            underlying,
            scope: None,
            attributes,
            comment,
            location,
        }
    }

    /// Returns the min enum value if the enum is non-empty.
    pub fn min_value(&self, ast: &Ast) -> Option<i64> {
        self.enumerators(ast)
            .iter()
            .map(|enumerator| enumerator.value)
            .min()
    }

    /// Returns the max enum value if the enum is non-empty.
    pub fn max_value(&self, ast: &Ast) -> Option<i64> {
        self.enumerators(ast)
            .iter()
            .map(|enumerator| enumerator.value)
            .max()
    }

    pub fn enumerators<'a>(&self, ast: &'a Ast) -> Vec<&'a Enumerator> {
        self.enumerators
            .iter()
            .map(|id| ref_from_node!(Node::Enumerator, ast, *id))
            .collect()
    }
}

impl Type for Enum {
    fn is_fixed_size(&self, ast: &Ast) -> bool {
        if let Some(typeref) = &self.underlying {
            typeref
                .definition(ast)
                .as_type()
                .unwrap()
                .is_fixed_size(ast)
        } else {
            true
        }
    }

    fn min_wire_size(&self, ast: &Ast) -> u32 {
        if let Some(typeref) = &self.underlying {
            typeref
                .definition(ast)
                .as_type()
                .unwrap()
                .min_wire_size(ast)
        } else {
            1
        }
    }

    fn tag_format(&self, ast: &Ast) -> TagFormat {
        if let Some(underlying) = &self.underlying {
            underlying
                .definition(ast)
                .as_type()
                .unwrap()
                .tag_format(ast)
        } else {
            TagFormat::Size
        }
    }

    fn uses_classes(&self, _: &Ast) -> bool {
        false
    }
}

#[derive(Clone, Debug)]
pub struct Operation {
    pub return_type: Vec<usize>,
    pub parameters: Vec<usize>,
    pub identifier: Identifier,
    pub scope: Option<String>,
    pub attributes: Vec<Attribute>,
    pub comment: Option<DocComment>,
    pub location: Location,
}

impl Operation {
    pub fn new(
        return_type: Vec<usize>,
        identifier: Identifier,
        parameters: Vec<usize>,
        attributes: Vec<Attribute>,
        comment: Option<DocComment>,
        location: Location,
    ) -> Self {
        Operation {
            return_type,
            parameters,
            identifier,
            scope: None,
            attributes,
            comment,
            location,
        }
    }

    pub fn parameters<'a>(&self, ast: &'a Ast) -> Vec<&'a Member> {
        self.parameters
            .iter()
            .map(|index| ref_from_node!(Node::Member, ast, *index))
            .collect()
    }

    pub fn return_members<'a>(&self, ast: &'a Ast) -> Vec<&'a Member> {
        self.return_type
            .iter()
            .map(|index| ref_from_node!(Node::Member, ast, *index))
            .collect()
    }

    pub fn has_non_streamed_params(&self, ast: &Ast) -> bool {
        let parameters = self.parameters(ast);
        // An operation can only have 1 streamed parameter; if it has more than 1 parameter, there
        // must be non-streamed parameters. Otherwise we check if the 1 parameter is streamed
        // (if it has any parameters at all).
        match parameters.len() {
            0 => false,
            1 => !parameters[0].data_type.is_streamed,
            _ => true,
        }
    }

    pub fn has_non_streamed_return(&self, ast: &Ast) -> bool {
        let return_members = self.return_members(ast);
        // An operation can only have 1 streamed return member; if it has more than 1 parameter,
        // there must be non-streamed return members. Otherwise we check if the 1 parameter is
        // streamed (if it has any parameters at all).
        match return_members.len() {
            0 => false,
            1 => !return_members[0].data_type.is_streamed,
            _ => true,
        }
    }

    pub fn non_streamed_params<'a>(&self, ast: &'a Ast) -> Vec<&'a Member> {
        self.parameters(ast)
            .iter()
            .filter(|p| !p.data_type.is_streamed)
            .cloned()
            .collect()
    }

    pub fn non_streamed_returns<'a>(&self, ast: &'a Ast) -> Vec<&'a Member> {
        self.return_members(ast)
            .iter()
            .filter(|p| !p.data_type.is_streamed)
            .cloned()
            .collect()
    }

    pub fn stream_parameter<'a>(&self, ast: &'a Ast) -> Option<&'a Member> {
        let params = self.parameters(ast);

        match params.last() {
            Some(p) if p.data_type.is_streamed => Some(p),
            _ => None,
        }
    }

    pub fn stream_return<'a>(&self, ast: &'a Ast) -> Option<&'a Member> {
        let params = self.return_members(ast);

        match params.last() {
            Some(p) if p.data_type.is_streamed => Some(p),
            _ => None,
        }
    }

    pub fn sends_classes(&self, ast: &Ast) -> bool {
        self.parameters(ast).iter().any(|p| {
            p.data_type
                .definition(ast)
                .as_type()
                .unwrap()
                .uses_classes(ast)
        })
    }

    pub fn returns_classes(&self, ast: &Ast) -> bool {
        self.return_members(ast).iter().any(|p| {
            p.data_type
                .definition(ast)
                .as_type()
                .unwrap()
                .uses_classes(ast)
        })
    }

    pub fn compress_arguments(&self) -> bool {
        if let Some(compress_attribute) = self.find_attribute("compress") {
            compress_attribute.contains(&"args".to_owned())
        } else {
            false
        }
    }

    pub fn compress_return(&self) -> bool {
        if let Some(compress_attribute) = self.find_attribute("compress") {
            compress_attribute.contains(&"return".to_owned())
        } else {
            false
        }
    }

    pub fn is_idempotent(&self) -> bool {
        // TODO: implement
        false
    }
}

#[derive(Clone, Debug)]
pub struct Member {
    pub data_type: TypeRef,
    pub identifier: Identifier,
    pub tag: Option<u32>,
    pub member_type: MemberType,
    pub scope: Option<String>,
    pub attributes: Vec<Attribute>,
    pub comment: Option<DocComment>,
    pub location: Location,
}

impl Member {
    pub fn new(
        data_type: TypeRef,
        identifier: Identifier,
        tag: Option<u32>,
        member_type: MemberType,
        attributes: Vec<Attribute>,
        comment: Option<DocComment>,
        location: Location,
    ) -> Self {
        Member {
            data_type,
            identifier,
            tag,
            member_type,
            scope: None,
            attributes,
            comment,
            location,
        }
    }
}

impl Element for Member {
    fn kind(&self) -> &'static str {
        match self.member_type {
            MemberType::DataMember => "data member",
            MemberType::Parameter => "parameter",
            MemberType::ReturnElement => "return element",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum MemberType {
    DataMember,
    Parameter,
    ReturnElement,
}

#[derive(Clone, Debug)]
pub struct Enumerator {
    pub identifier: Identifier,
    pub value: i64,
    pub scope: Option<String>,
    pub attributes: Vec<Attribute>,
    pub comment: Option<DocComment>,
    pub location: Location,
}

impl Enumerator {
    pub fn new(
        identifier: Identifier,
        value: i64,
        attributes: Vec<Attribute>,
        comment: Option<DocComment>,
        location: Location,
    ) -> Self {
        Enumerator { identifier, value, scope: None, attributes, comment, location }
    }
}

#[derive(Clone, Debug)]
pub struct Identifier {
    pub value: String,
    pub location: Location,
}

impl Identifier {
    pub fn new(value: String, location: Location) -> Self {
        Identifier { value, location }
    }
}

#[derive(Clone, Debug)]
pub struct TypeRef {
    pub type_name: String,
    pub is_optional: bool,
    pub is_streamed: bool,
    pub definition: Option<usize>,
    pub scope: Option<String>,
    pub attributes: Vec<Attribute>,
    pub location: Location,
}

impl TypeRef {
    pub fn new(
        type_name: String,
        is_optional: bool,
        attributes: Vec<Attribute>,
        location: Location,
    ) -> Self {
        TypeRef {
            type_name,
            is_optional,
            is_streamed: false,
            definition: None,
            scope: None,
            attributes,
            location,
        }
    }

    pub fn definition<'a>(&self, ast: &'a Ast) -> &'a Node {
        ast.resolve_index(self.definition.unwrap())
    }

    pub fn min_wire_size(&self, ast: &Ast) -> u32 {
        let node = self.definition(ast);

        if self.is_optional {
            match node {
                Node::Interface(_, _) => 1,
                Node::Class(_, _) => 1,
                _ => 0,
            }
        } else {
            node.as_type().unwrap().min_wire_size(ast)
        }
    }

    pub fn encode_using_bit_sequence(&self, ast: &Ast) -> bool {
        self.is_optional && self.min_wire_size(ast) == 0
    }

    pub fn is_fixed_size(&self, ast: &Ast) -> bool {
        self.definition(ast).as_type().unwrap().is_fixed_size(ast)
    }

    pub fn tag_format(&self, ast: &Ast) -> TagFormat {
        self.definition(ast).as_type().unwrap().tag_format(ast)
    }
}

#[derive(Clone, Debug)]
pub struct Sequence {
    pub element_type: TypeRef,
    pub scope: Option<String>,
}

impl Sequence {
    pub fn new(element_type: TypeRef) -> Self {
        Sequence { element_type, scope: None }
    }

    pub fn is_element_fixed_sized_numeric(&self, ast: &Ast) -> bool {
        let mut element_node = self.element_type.definition(ast);

        // If the elements are an enum with an underlying type, check the underlying type instead.
        if let Node::Enum(_, enum_def) = element_node {
            if let Some(underlying) = &enum_def.underlying {
                element_node = underlying.definition(ast);
            }
        }

        if let Node::Primitive(_, primitive) = element_node {
            primitive.is_numeric_or_bool() && primitive.is_fixed_size(ast)
        } else {
            false
        }
    }
}

impl Type for Sequence {
    fn is_fixed_size(&self, _: &Ast) -> bool {
        false
    }

    fn min_wire_size(&self, _: &Ast) -> u32 {
        1
    }

    fn tag_format(&self, ast: &Ast) -> TagFormat {
        if self.element_type.is_fixed_size(ast) {
            if self.element_type.min_wire_size(ast) == 1 {
                TagFormat::OVSize
            } else {
                TagFormat::VSize
            }
        } else {
            TagFormat::FSize
        }
    }

    fn uses_classes(&self, ast: &Ast) -> bool {
        self.element_type
            .definition(ast)
            .as_type()
            .unwrap()
            .uses_classes(ast)
    }
}

#[derive(Clone, Debug)]
pub struct Dictionary {
    pub key_type: TypeRef,
    pub value_type: TypeRef,
    pub scope: Option<String>,
}

impl Dictionary {
    pub fn new(key_type: TypeRef, value_type: TypeRef) -> Self {
        Dictionary { key_type, value_type, scope: None }
    }
}

impl Type for Dictionary {
    fn is_fixed_size(&self, _: &Ast) -> bool {
        false
    }

    fn min_wire_size(&self, _: &Ast) -> u32 {
        1
    }

    fn tag_format(&self, ast: &Ast) -> TagFormat {
        if self.key_type.is_fixed_size(ast) || self.value_type.is_fixed_size(ast) {
            TagFormat::FSize
        } else {
            TagFormat::VSize
        }
    }

    fn uses_classes(&self, ast: &Ast) -> bool {
        self.value_type
            .definition(ast)
            .as_type()
            .unwrap()
            .uses_classes(ast)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Primitive {
    Bool,
    Byte,
    Short,
    UShort,
    Int,
    UInt,
    VarInt,
    VarUInt,
    Long,
    ULong,
    VarLong,
    VarULong,
    Float,
    Double,
    String,
}

impl Primitive {
    pub fn is_numeric_or_bool(&self) -> bool {
        !matches!(&self, Self::String)
    }

    pub fn is_unsigned_numeric(&self) -> bool {
        matches!(
            self,
            Self::Byte | Self::UShort | Self::UInt | Self::ULong | Self::VarUInt | Self::VarULong
        )
    }
}

impl Element for Primitive {
    fn kind(&self) -> &'static str {
        match self {
            Self::Bool => "bool",
            Self::Byte => "byte",
            Self::Short => "short",
            Self::UShort => "ushort",
            Self::Int => "int",
            Self::UInt => "uint",
            Self::VarInt => "varint",
            Self::VarUInt => "varuint",
            Self::Long => "long",
            Self::ULong => "ulong",
            Self::VarLong => "varlong",
            Self::VarULong => "varulong",
            Self::Float => "float",
            Self::Double => "double",
            Self::String => "string",
        }
    }
}

impl Type for Primitive {
    fn is_fixed_size(&self, _: &Ast) -> bool {
        !matches!(
            self,
            Self::VarInt | Self::VarUInt | Self::VarLong | Self::VarULong | Self::String
        )
    }

    fn min_wire_size(&self, _: &Ast) -> u32 {
        match self {
            Self::Bool => 1,
            Self::Byte => 1,
            Self::Short => 2,
            Self::UShort => 2,
            Self::Int => 4,
            Self::UInt => 4,
            Self::VarInt => 1,
            Self::VarUInt => 1,
            Self::Long => 8,
            Self::ULong => 8,
            Self::VarLong => 1,
            Self::VarULong => 1,
            Self::Float => 4,
            Self::Double => 8,
            Self::String => 1,
        }
    }

    fn tag_format(&self, _: &Ast) -> TagFormat {
        match self {
            Self::Bool | Self::Byte => TagFormat::F1,
            Self::Short | Self::UShort => TagFormat::F2,
            Self::Int | Self::UInt | Self::Float => TagFormat::F4,
            Self::Long | Self::ULong | Self::Double => TagFormat::F8,
            Self::VarInt | Self::VarUInt | Self::VarLong | Self::VarULong => TagFormat::VInt,
            Self::String => TagFormat::OVSize,
        }
    }

    fn uses_classes(&self, _: &Ast) -> bool {
        false
    }
}

#[derive(Clone, Debug)]
pub struct Attribute {
    /// If the attribute's directive had a language mapping prefix, it is stored here, otherwise
    /// this is `None`. Ex: the prefix for `cs::readonly` would be `cs`.
    pub prefix: Option<String>,
    /// The attribute's directive, without it's prefix if one was present.
    pub directive: String,
    /// Stores the fully qualified directive (the prefix and directive with a `::` separator).
    /// We compute this up-front, to make searching for fully qualified metadata more efficient.
    pub qualified_directive: String,
    /// Stores all the arguments passed into the directive, in the order they were passed.
    /// For directives that don't take any arguments, this should always be empty.
    pub arguments: Vec<String>,
    pub location: Location,
}

impl Attribute {
    pub fn new(
        prefix: Option<String>,
        directive: String,
        arguments: Vec<String>,
        location: Location,
    ) -> Self {
        // Combine the prefix and directive together to make searching qualified directives easier.
        let qualified_directive = match &prefix {
            Some(prefix) => format!("{}:{}", prefix, directive),
            _ => directive.clone(),
        };
        Attribute { prefix, directive, qualified_directive, arguments, location }
    }
}

#[derive(Clone, Debug)]
pub struct DocComment {
    pub message: String,
    pub references: Vec<String>,
    pub deprecate_reason: Option<String>,
    pub params: Vec<(String, String)>,
    pub returns: Option<String>,
    pub throws: Vec<(String, String)>,
    pub location: Location,
}
