// Copyright (c) ZeroC, Inc. All rights reserved.

use super::comments::DocComment;
use super::traits::*;
use super::util::{Scope, Encoding, TagFormat};
use super::wrappers::*;
use crate::slice_file::Location;
use crate::supported_encodings::SupportedEncodings;
use crate::ptr_util::{OwnedPtr, WeakPtr};

#[derive(Debug)]
pub struct Module {
    pub identifier: Identifier,
    pub contents: Vec<Definition>,
    pub parent: Option<WeakPtr<Module>>,
    pub scope: Scope,
    pub attributes: Vec<Attribute>,
    pub comment: Option<DocComment>,
    pub location: Location,
}

impl Module {
    pub(crate) fn new(
        identifier: Identifier,
        scope: Scope,
        attributes: Vec<Attribute>,
        comment: Option<DocComment>,
        location: Location,
    ) -> Self {
        let contents = Vec::new();
        let parent = None;
        Module { identifier, contents, parent, scope, attributes, comment, location }
    }

    pub(crate) fn add_definition(&mut self, definition: Definition) {
        self.contents.push(definition);
    }

    pub fn is_top_level(&self) -> bool {
        self.parent.is_none()
    }

    pub fn submodules(&self) -> Vec<&Module> {
        self.contents.iter()
            .filter_map(|definition| {
                if let Definition::Module(module_def) = definition {
                    Some(module_def.borrow())
                } else {
                    None
                }
            })
            .collect()
    }
}

impl Contained<Module> for Module {
    fn parent(&self) -> Option<&Module> {
        self.parent.as_ref().map(|ptr| ptr.borrow())
    }
}

implement_Element_for!(Module, "module");
implement_Entity_for!(Module);
implement_Container_for!(Module, Definition, contents);

#[derive(Debug)]
pub struct Struct {
    pub identifier: Identifier,
    pub members: Vec<OwnedPtr<DataMember>>,
    pub is_compact: bool,
    pub parent: WeakPtr<Module>,
    pub scope: Scope,
    pub attributes: Vec<Attribute>,
    pub comment: Option<DocComment>,
    pub location: Location,
    pub(crate) supported_encodings: Option<SupportedEncodings>,
}

impl Struct {
    pub(crate) fn new(
        identifier: Identifier,
        is_compact: bool,
        scope: Scope,
        attributes: Vec<Attribute>,
        comment: Option<DocComment>,
        location: Location,
    ) -> Self {
        let members = Vec::new();
        let parent = WeakPtr::create_uninitialized();
        let supported_encodings = None; // Patched later by the encoding_patcher.
        Struct { identifier, members, is_compact, parent, scope, attributes, comment, location, supported_encodings }
    }

    pub(crate) fn add_member(&mut self, member: DataMember) {
        self.members.push(OwnedPtr::new(member));
    }

    pub fn members(&self) -> Vec<&DataMember> {
        self.members.iter()
            .map(|member_ptr| member_ptr.borrow())
            .collect()
    }
}

impl Type for Struct {
    fn is_fixed_size(&self) -> bool {
        // A struct is fixed size if and only if all its members are fixed size.
        self.members().iter()
            .all(|member| member.data_type.is_fixed_size())
    }

    fn min_wire_size(&self) -> u32 {
        // The min-wire-size of a struct is the min-wire-size of all its members added together.
        let min_wire_size = self.members().iter()
            .map(|member| member.data_type.min_wire_size())
            .sum();
        if self.is_compact {
            min_wire_size
        } else {
            // Non-compact structs use an extra byte to encode TagEndMarker.
            min_wire_size + 1
        }
    }

    fn uses_classes(&self) -> bool {
        self.members().iter()
            .any(|member| member.data_type.uses_classes())
    }

    fn is_class_type(&self) -> bool {
        false
    }

    fn tag_format(&self) -> TagFormat {
        if self.is_fixed_size() {
            TagFormat::VSize
        } else {
            TagFormat::FSize
        }
    }

    fn supported_encodings(&self) -> SupportedEncodings {
        self.supported_encodings.clone().unwrap()
    }
}

implement_Element_for!(Struct, "struct");
implement_Entity_for!(Struct);
implement_Container_for!(Struct, OwnedPtr<DataMember>, members);
implement_Contained_for!(Struct, Module);

#[derive(Debug)]
pub struct Class {
    pub identifier: Identifier,
    pub members: Vec<OwnedPtr<DataMember>>,
    pub compact_id: Option<u32>,
    pub base: Option<TypeRef<Class>>,
    pub parent: WeakPtr<Module>,
    pub scope: Scope,
    pub attributes: Vec<Attribute>,
    pub comment: Option<DocComment>,
    pub location: Location,
    pub(crate) supported_encodings: Option<SupportedEncodings>,
}

impl Class {
    pub(crate) fn new(
        identifier: Identifier,
        compact_id: Option<u32>,
        base: Option<TypeRef<Class>>,
        scope: Scope,
        attributes: Vec<Attribute>,
        comment: Option<DocComment>,
        location: Location,
    ) -> Self {
        let members = Vec::new();
        let parent = WeakPtr::create_uninitialized();
        let supported_encodings = None; // Patched later by the encoding_patcher.
        Class { identifier, compact_id, members, base, parent, scope, attributes, comment, location, supported_encodings }
    }

    pub(crate) fn add_member(&mut self, member: DataMember) {
        self.members.push(OwnedPtr::new(member));
    }

    pub fn members(&self) -> Vec<&DataMember> {
        self.members.iter()
            .map(|member_ptr| member_ptr.borrow())
            .collect()
    }

    pub fn all_members(&self) -> Vec<&DataMember> {
        let mut members = vec![];
        // Recursively add inherited data members from super-classes.
        if let Some(base_class) = self.base_class() {
            members.extend(base_class.all_members());
        }
        members.extend(self.members());
        members
    }

    pub fn base_class(&self) -> Option<&Class> {
        self.base.as_ref()
            .map(|type_ref| type_ref.definition())
    }
}

impl Type for Class {
    fn is_fixed_size(&self) -> bool {
        false // A class can always be encoded as either a full instance, or just an index.
    }

    fn min_wire_size(&self) -> u32 {
        1 // A class may be encoded as an index instead of an instance, taking up 1 byte.
    }

    fn uses_classes(&self) -> bool {
        true
    }

    fn is_class_type(&self) -> bool {
        true
    }

    fn tag_format(&self) -> TagFormat {
        TagFormat::Class
    }

    fn supported_encodings(&self) -> SupportedEncodings {
        self.supported_encodings.clone().unwrap()
    }
}

implement_Element_for!(Class, "class");
implement_Entity_for!(Class);
implement_Container_for!(Class, OwnedPtr<DataMember>, members);
implement_Contained_for!(Class, Module);

#[derive(Debug)]
pub struct Exception {
    pub identifier: Identifier,
    pub members: Vec<OwnedPtr<DataMember>>,
    pub base: Option<TypeRef<Exception>>,
    pub parent: WeakPtr<Module>,
    pub scope: Scope,
    pub attributes: Vec<Attribute>,
    pub comment: Option<DocComment>,
    pub location: Location,
    pub(crate) supported_encodings: Option<SupportedEncodings>,
}

impl Exception {
    pub(crate) fn new(
        identifier: Identifier,
        base: Option<TypeRef<Exception>>,
        scope: Scope,
        attributes: Vec<Attribute>,
        comment: Option<DocComment>,
        location: Location,
    ) -> Self {
        let members = Vec::new();
        let parent = WeakPtr::create_uninitialized();
        let supported_encodings = None; // Patched later by the encoding_patcher.
        Exception { identifier, members, base, parent, scope, attributes, comment, location, supported_encodings }
    }

    pub(crate) fn add_member(&mut self, member: DataMember) {
        self.members.push(OwnedPtr::new(member));
    }

    pub fn members(&self) -> Vec<&DataMember> {
        self.members.iter()
            .map(|member_ptr| member_ptr.borrow())
            .collect()
    }

    pub fn all_members(&self) -> Vec<&DataMember> {
        let mut members = vec![];
        // Recursively add inherited data members from super-exceptions.
        if let Some(base_class) = self.base_exception() {
            members.extend(base_class.all_members());
        }
        members.extend(self.members());
        members
    }

    pub fn base_exception(&self) -> Option<&Exception> {
        self.base.as_ref()
            .map(|type_ref| type_ref.definition())
    }
}

impl Type for Exception {
    fn is_fixed_size(&self) -> bool {
        // An exception is fixed size if and only if all its members are fixed size.
        self.all_members().iter()
            .all(|member| member.data_type.is_fixed_size())
    }

    fn min_wire_size(&self) -> u32 {
        // The min-wire-size of an exception is the min-wire-size of all its members added together.
        self.all_members().iter()
            .map(|member| member.data_type.min_wire_size())
            .sum()
    }

    fn uses_classes(&self) -> bool {
        self.all_members().iter()
            .any(|member| member.data_type.uses_classes())
    }

    fn is_class_type(&self) -> bool {
        false
    }

    fn tag_format(&self) -> TagFormat {
        unimplemented!("Tag formats are only used with the 1.1 encoding.\n\
                        Exceptions can only be sent as a member with the 2.0 encoding.");
    }

    fn supported_encodings(&self) -> SupportedEncodings {
        self.supported_encodings.clone().unwrap()
    }
}

implement_Element_for!(Exception, "exception");
implement_Entity_for!(Exception);
implement_Container_for!(Exception, OwnedPtr<DataMember>, members);
implement_Contained_for!(Exception, Module);

#[derive(Debug)]
pub struct DataMember {
    pub identifier: Identifier,
    pub data_type: TypeRef,
    pub tag: Option<u32>,
    pub parent: WeakPtr<dyn Container<OwnedPtr<DataMember>>>,
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
        DataMember { identifier, data_type, tag, parent, scope, attributes, comment, location }
    }
}

implement_Element_for!(DataMember, "data member");
implement_Entity_for!(DataMember);
implement_Contained_for!(DataMember, dyn Container<OwnedPtr<DataMember>> + 'static);
implement_Member_for!(DataMember);

#[derive(Debug)]
pub struct Interface {
    pub identifier: Identifier,
    pub operations: Vec<OwnedPtr<Operation>>,
    pub bases: Vec<TypeRef<Interface>>,
    pub parent: WeakPtr<Module>,
    pub scope: Scope,
    pub attributes: Vec<Attribute>,
    pub comment: Option<DocComment>,
    pub location: Location,
    pub(crate) supported_encodings: Option<SupportedEncodings>,
}

impl Interface {
    pub(crate) fn new(
        identifier: Identifier,
        bases: Vec<TypeRef<Interface>>,
        scope: Scope,
        attributes: Vec<Attribute>,
        comment: Option<DocComment>,
        location: Location,
    ) -> Self {
        let operations = Vec::new();
        let parent = WeakPtr::create_uninitialized();
        let supported_encodings = None; // Patched later by the encoding_patcher.
        Interface { identifier, operations, bases, parent, scope, attributes, comment, location, supported_encodings }
    }

    pub(crate) fn add_operation(&mut self, operation: Operation) {
        self.operations.push(OwnedPtr::new(operation));
    }

    pub fn operations(&self) -> Vec<&Operation> {
        self.operations.iter()
            .map(|operation_ptr| operation_ptr.borrow())
            .collect()
    }

    pub fn all_inherited_operations(&self) -> Vec<&Operation> {
        let mut operations = self.all_base_interfaces().iter()
            .flat_map(|base_interface| base_interface.operations())
            .collect::<Vec<&Operation>>();

        // Dedup only works on sorted collections, so we have to sort the operations first.
        operations.sort_by_key(|operation| operation.identifier());
        operations.dedup_by_key(|operation| operation.identifier());
        operations
    }

    pub fn all_operations(&self) -> Vec<&Operation> {
        let mut operations = self.operations();
        operations.extend(self.all_inherited_operations());

        // Dedup only works on sorted collections, so we have to sort the operations first.
        operations.sort_by_key(|operation| operation.identifier());
        operations.dedup_by_key(|operation| operation.identifier());
        operations
    }

    pub fn base_interfaces(&self) -> Vec<&Interface> {
        self.bases.iter()
            .map(|type_ref| type_ref.definition())
            .collect()
    }

    pub fn all_base_interfaces(&self) -> Vec<&Interface> {
        let mut bases = self.base_interfaces();
        bases.extend(
            self.bases
                .iter()
                .flat_map(|type_ref| type_ref.all_base_interfaces())
                .collect::<Vec<&Interface>>(),
        );

        // Dedup only works on sorted collections, so we have to sort the bases first.
        bases.sort_by_key(|base| base.module_scoped_identifier());
        bases.dedup_by_key(|base| base.module_scoped_identifier());
        bases
    }
}

impl Type for Interface {
    fn is_fixed_size(&self) -> bool {
        false
    }

    fn min_wire_size(&self) -> u32 {
        // TODO write a comment explaining why this is 3.
        3
    }

    fn uses_classes(&self) -> bool {
        false
    }

    fn is_class_type(&self) -> bool {
        false
    }

    fn tag_format(&self) -> TagFormat {
        TagFormat::FSize
    }

    fn supported_encodings(&self) -> SupportedEncodings {
        self.supported_encodings.clone().unwrap()
    }
}

implement_Element_for!(Interface, "interface");
implement_Entity_for!(Interface);
implement_Container_for!(Interface, OwnedPtr<Operation>, operations);
implement_Contained_for!(Interface, Module);

#[derive(Debug)]
pub struct Operation {
    pub identifier: Identifier,
    pub return_type: Vec<OwnedPtr<Parameter>>,
    pub parameters: Vec<OwnedPtr<Parameter>>,
    pub is_idempotent: bool,
    pub encoding: Encoding,
    pub parent: WeakPtr<Interface>,
    pub scope: Scope,
    pub attributes: Vec<Attribute>,
    pub comment: Option<DocComment>,
    pub location: Location,
}

impl Operation {
    pub(crate) fn new(
        identifier: Identifier,
        return_type: Vec<OwnedPtr<Parameter>>,
        is_idempotent: bool,
        encoding: Encoding,
        scope: Scope,
        attributes: Vec<Attribute>,
        comment: Option<DocComment>,
        location: Location,
    ) -> Self {
        let parameters = Vec::new();
        let parent = WeakPtr::create_uninitialized();
        Operation { identifier, return_type, parameters, is_idempotent, encoding, parent, scope, attributes, comment, location }
    }

    pub(crate) fn add_parameter(&mut self, parameter: Parameter) {
        self.parameters.push(OwnedPtr::new(parameter));
    }

    pub fn parameters(&self) -> Vec<&Parameter> {
        self.parameters.iter()
            .map(|parameter_ptr| parameter_ptr.borrow())
            .collect()
    }

    pub fn return_members(&self) -> Vec<&Parameter> {
        self.return_type.iter()
            .map(|parameter_ptr| parameter_ptr.borrow())
            .collect()
    }

    pub fn has_nonstreamed_parameters(&self) -> bool {
        // Operations can have at most 1 streamed parameter. So, if it has more than 1 parameter
        // there must be unstreamed parameters. Otherwise we check if the 1 parameter is streamed.
        match self.parameters.len() {
            0 => false,
            1 => !self.parameters[0].borrow().is_streamed,
            _ => true,
        }
    }

    pub fn has_nonstreamed_return_members(&self) -> bool {
        // Operations can have at most 1 streamed return member. So, if it has more than 1 member
        // there must be unstreamed members. Otherwise we check if the 1 member is streamed.
        match self.return_type.len() {
            0 => false,
            1 => !self.return_type[0].borrow().is_streamed,
            _ => true,
        }
    }

    pub fn nonstreamed_parameters(&self) -> Vec<&Parameter> {
        self.parameters().iter()
            .filter(|parameter| !parameter.is_streamed)
            .cloned()
            .collect()
    }

    pub fn nonstreamed_return_members(&self) -> Vec<&Parameter> {
        self.return_members().iter()
            .filter(|parameter| !parameter.is_streamed)
            .cloned()
            .collect()
    }

    pub fn streamed_parameter(&self) -> Option<&Parameter> {
        // There can be only 1 streamed parameter and it must be the last parameter.
        self.parameters().last()
            .filter(|parameter| parameter.is_streamed)
            .cloned()
    }

    pub fn streamed_return_member(&self) -> Option<&Parameter> {
        // There can be only 1 streamed return member and it must be the last member.
        self.return_members().last()
            .filter(|parameter| parameter.is_streamed)
            .cloned()
    }

    pub fn sends_classes(&self) -> bool {
        self.parameters().iter()
            .any(|parameter| parameter.data_type.uses_classes())
    }

    pub fn returns_classes(&self) -> bool {
        self.return_members().iter()
            .any(|parameter| parameter.data_type.uses_classes())
    }

    pub fn compress_arguments(&self) -> bool {
        if let Some(attribute) = self.get_attribute("compress", false) {
            attribute.contains(&"args".to_owned())
        } else {
            false
        }
    }

    pub fn compress_return(&self) -> bool {
        if let Some(attribute) = self.get_attribute("compress", false) {
            attribute.contains(&"return".to_owned())
        } else {
            false
        }
    }
}

implement_Element_for!(Operation, "operation");
implement_Entity_for!(Operation);
implement_Contained_for!(Operation, Interface);

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
        Parameter { identifier, data_type, tag, is_streamed, is_returned, parent, scope, attributes, comment, location }
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

#[derive(Debug)]
pub struct Enum {
    pub identifier: Identifier,
    pub enumerators: Vec<OwnedPtr<Enumerator>>,
    pub underlying: Option<TypeRef<Primitive>>,
    pub is_unchecked: bool,
    pub parent: WeakPtr<Module>,
    pub scope: Scope,
    pub attributes: Vec<Attribute>,
    pub comment: Option<DocComment>,
    pub location: Location,
    pub(crate) supported_encodings: Option<SupportedEncodings>,
}

impl Enum {
    pub(crate) fn new(
        identifier: Identifier,
        underlying: Option<TypeRef<Primitive>>,
        is_unchecked: bool,
        scope: Scope,
        attributes: Vec<Attribute>,
        comment: Option<DocComment>,
        location: Location,
    ) -> Self {
        let enumerators = Vec::new();
        let parent = WeakPtr::create_uninitialized();
        let supported_encodings = None; // Patched later by the encoding_patcher.
        Enum { identifier, enumerators, underlying, is_unchecked, parent, scope, attributes, comment, location, supported_encodings }
    }

    pub(crate) fn add_enumerator(&mut self, enumerator: Enumerator) {
        self.enumerators.push(OwnedPtr::new(enumerator));
    }

    pub fn enumerators(&self) -> Vec<&Enumerator> {
        self.enumerators.iter()
            .map(|enumerator_ptr| enumerator_ptr.borrow())
            .collect()
    }

    pub fn underlying_type(&self) -> &Primitive {
        // If the enum has an underlying type, return a reference to its definition.
        // Otherwise, enums have a backing type of `int` by default. Since `int` is a type
        // defined by the compiler, we fetch its definition directly from the global AST.
        self.underlying.as_ref().map_or(
            crate::borrow_ast().lookup_primitive("int").borrow(),
            |data_type| data_type.definition(),
        )
    }

    pub fn get_min_max_values(&self) -> Option<(i64, i64)> {
        let values = self.enumerators.iter().map(
            |enumerator| enumerator.borrow().value
        );

        // There might not be a minimum value if the enum is empty.
        values.clone().min().map(|min| (
            min,
            values.max().unwrap() // A 'min' guarantees a 'max' exists too, so unwrap is safe.

        ))
    }
}

impl Type for Enum {
    fn is_fixed_size(&self) -> bool {
        match &self.underlying {
            Some(underlying) => underlying.is_fixed_size(),
            _ => false
        }
    }

    fn min_wire_size(&self) -> u32 {
        match &self.underlying {
            Some(underlying) => underlying.min_wire_size(),
            _ => 1
        }
    }

    fn uses_classes(&self) -> bool {
        false
    }

    fn is_class_type(&self) -> bool {
        false
    }

    fn tag_format(&self) -> TagFormat {
        self.underlying.as_ref().map_or(
            TagFormat::Size,                    // Default value if `underlying` == None
            |data_type| data_type.tag_format(), // Expression to evaluate otherwise
        )
    }

    fn supported_encodings(&self) -> SupportedEncodings {
        self.supported_encodings.clone().unwrap()
    }
}

implement_Element_for!(Enum, "enum");
implement_Entity_for!(Enum);
implement_Container_for!(Enum, OwnedPtr<Enumerator>, enumerators);
implement_Contained_for!(Enum, Module);

#[derive(Debug)]
pub struct Enumerator {
    pub identifier: Identifier,
    pub value: i64,
    pub parent: WeakPtr<Enum>,
    pub scope: Scope,
    pub attributes: Vec<Attribute>,
    pub comment: Option<DocComment>,
    pub location: Location,
}

impl Enumerator {
    pub(crate) fn new(
        identifier: Identifier,
        value: i64,
        scope: Scope,
        attributes: Vec<Attribute>,
        comment: Option<DocComment>,
        location: Location,
    ) -> Self {
        let parent = WeakPtr::create_uninitialized();
        Enumerator { identifier, value, parent, scope, attributes, comment, location }
    }
}

implement_Element_for!(Enumerator, "enumerator");
implement_Entity_for!(Enumerator);
implement_Contained_for!(Enumerator, Enum);

#[derive(Debug)]
pub struct Trait {
    pub identifier: Identifier,
    pub parent: WeakPtr<Module>,
    pub scope: Scope,
    pub attributes: Vec<Attribute>,
    pub comment: Option<DocComment>,
    pub location: Location,
    pub(crate) supported_encodings: Option<SupportedEncodings>,
}

impl Trait {
    pub(crate) fn new(
        identifier: Identifier,
        scope: Scope,
        attributes: Vec<Attribute>,
        comment: Option<DocComment>,
        location: Location,
    ) -> Self {
        let parent = WeakPtr::create_uninitialized();
        let supported_encodings = None; // Patched later by the encoding_patcher.
        Trait { identifier, parent, scope, attributes, comment, location, supported_encodings }
    }
}

impl Type for Trait {
    fn is_fixed_size(&self) -> bool {
        false
    }

    fn min_wire_size(&self) -> u32 {
        // Traits are encoded as a type-id string followed by a concrete type, encoded normally.
        // The min wire size is an empty type-id (1 byte) and an empty concrete type.
        1
    }

    fn uses_classes(&self) -> bool {
        false
    }

    fn is_class_type(&self) -> bool {
        false
    }

    fn tag_format(&self) -> TagFormat {
        unimplemented!("Tag formats are only used with the 1.1 encoding. Traits are 2.0 only.")
    }

    fn supported_encodings(&self) -> SupportedEncodings {
        self.supported_encodings.clone().unwrap()
    }
}

implement_Element_for!(Trait, "trait");
implement_Entity_for!(Trait);
implement_Contained_for!(Trait, Module);

#[derive(Debug)]
pub struct CustomType {
    pub identifier: Identifier,
    pub parent: WeakPtr<Module>,
    pub scope: Scope,
    pub attributes: Vec<Attribute>,
    pub comment: Option<DocComment>,
    pub location: Location,
    pub(crate) supported_encodings: Option<SupportedEncodings>,
}

impl CustomType {
    pub(crate) fn new(
        identifier: Identifier,
        scope: Scope,
        attributes: Vec<Attribute>,
        comment: Option<DocComment>,
        location: Location,
    ) -> Self {
        let parent = WeakPtr::create_uninitialized();
        let supported_encodings = None; // Patched later by the encoding_patcher.
        CustomType { identifier, parent, scope, attributes, comment, location, supported_encodings }
    }
}

impl Type for CustomType {
    fn is_fixed_size(&self) -> bool {
        false
    }

    fn min_wire_size(&self) -> u32 {
        //TODO Can't we get rid of min wire size already?
        0
    }

    fn uses_classes(&self) -> bool {
        false
    }

    fn is_class_type(&self) -> bool {
        false
    }

    fn tag_format(&self) -> TagFormat {
        unimplemented!("Tag formats are only used with the 1.1 encoding. Custom types are 2.0 only.")
    }

    fn supported_encodings(&self) -> SupportedEncodings {
        self.supported_encodings.clone().unwrap()
    }
}

implement_Element_for!(CustomType, "custom type");
implement_Entity_for!(CustomType);
implement_Contained_for!(CustomType, Module);

#[derive(Debug)]
pub struct TypeAlias {
    pub identifier: Identifier,
    pub underlying: TypeRef,
    pub parent: WeakPtr<Module>,
    pub scope: Scope,
    pub attributes: Vec<Attribute>,
    pub comment: Option<DocComment>,
    pub location: Location,
}

impl TypeAlias {
    pub(crate) fn new(
        identifier: Identifier,
        underlying: TypeRef,
        scope: Scope,
        attributes: Vec<Attribute>,
        comment: Option<DocComment>,
        location: Location,
    ) -> Self {
        let parent = WeakPtr::create_uninitialized();
        TypeAlias { identifier, underlying, parent, scope, attributes, comment, location }
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

    fn tag_format(&self) -> TagFormat {
        self.underlying.tag_format()
    }

    fn supported_encodings(&self) -> SupportedEncodings {
        self.underlying.supported_encodings()
    }
}

implement_Element_for!(TypeAlias, "type alias");
implement_Entity_for!(TypeAlias);
implement_Contained_for!(TypeAlias, Module);

#[derive(Debug)]
pub struct TypeRef<T: Element + ?Sized = dyn Type> {
    pub type_string: String,
    pub definition: WeakPtr<T>,
    pub is_optional: bool,
    pub scope: Scope,
    pub attributes: Vec<Attribute>,
    pub location: Location,
}

impl<T: Element + ?Sized + 'static> TypeRef<T> {
    pub(crate) fn new(
        type_string: String,
        is_optional: bool,
        scope: Scope,
        attributes: Vec<Attribute>,
        location: Location,
    ) -> Self {
        let definition = WeakPtr::create_uninitialized();
        TypeRef { type_string, definition, is_optional, scope, attributes, location }
    }
}

impl<T: Element + ?Sized> TypeRef<T> {
    pub fn definition(&self) -> &T {
        self.definition.borrow()
    }

    pub(crate) fn downcast<U: Element + 'static>(&self) -> Result<TypeRef<U>, ()> {
        let definition = if self.definition.is_initialized() {
            match self.definition.clone().downcast::<U>() {
                Ok(ptr) => ptr,
                Err(_) => return Err(()),
            }
        } else {
            WeakPtr::create_uninitialized()
        };

        Ok(TypeRef {
            type_string: self.type_string.clone(),
            definition,
            is_optional: self.is_optional,
            scope: self.scope.clone(),
            attributes: self.attributes.clone(),
            location: self.location.clone(),
        })
    }
}

impl<T: Type + ?Sized> TypeRef<T> {
    pub fn is_bit_sequence_encodable(&self) -> bool {
        self.is_optional && self.min_wire_size() == 0
    }

    // This intentionally shadows the trait method of the same name on `Type`.
    pub fn is_fixed_size(&self) -> bool {
        !self.is_optional && T::is_fixed_size(self)
    }

    // This intentionally shadows the trait method of the same name on `Type`.
    pub fn min_wire_size(&self) -> u32 {
        if self.is_optional {
            match self.definition().concrete_type() {
                // TODO explain why still take up 1 byte.
                // TODO this is not totally correct the min_wire_size of a optional interface depends on the encoding
                Types::Class(_) => 1,
                Types::Primitive(primitive) if matches!(primitive, Primitive::AnyClass) => 1,
                _ => 0,
            }
        } else {
            T::min_wire_size(self)
        }
    }

    // This intentionally shadows the trait method of the same name on `Type`.
    pub fn supported_encodings(&self) -> SupportedEncodings {
        let mut supported_encodings = self.definition().supported_encodings();
        if self.is_optional {
            // Optional data types are not supported with the 1.1 encoding.
            // Note that this doesn't include tagged data members and parameters, which are allowed.
            // Even though they're marked with a '?' these are not technically optional types.
            supported_encodings.disable_11();
        }
        supported_encodings
    }
}

impl<T: Element + ?Sized> Clone for TypeRef<T> {
    fn clone(&self) -> Self {
        TypeRef {
            type_string: self.type_string.clone(),
            definition: self.definition.clone(),
            is_optional: self.is_optional,
            scope: self.scope.clone(),
            attributes: self.attributes.clone(),
            location: self.location.clone(),
        }
    }
}

impl<T: Element + ?Sized> Attributable for TypeRef<T> {
    fn attributes(&self) -> &Vec<Attribute> {
        &self.attributes
    }

    fn get_raw_attribute(&self, directive: &str, recurse: bool) -> Option<&Attribute> {
        if recurse {
            panic!("Cannot recursively get attributes on a typeref");
        }

        for attribute in &self.attributes {
            if attribute.prefixed_directive == directive {
                return Some(attribute);
            }
        }
        None
    }
}

impl<T: Element + ?Sized> std::ops::Deref for TypeRef<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.definition()
    }
}

implement_Element_for!(TypeRef<T>, "type reference", Element + ?Sized);
implement_Symbol_for!(TypeRef<T>, Element + ?Sized);
implement_Scoped_Symbol_for!(TypeRef<T>, Element + ?Sized);

#[derive(Debug)]
pub struct Sequence {
    pub element_type: TypeRef,
}

impl Sequence {
    pub fn has_fixed_size_numeric_elements(&self) -> bool {
        if self.element_type.is_optional {
            false
        } else {
            let mut definition = self.element_type.concrete_type();

            // If the elements are enums with an underlying type, check the underlying type instead.
            if let Types::Enum(enum_def) = definition {
                if let Some(underlying) = &enum_def.underlying {
                    definition = underlying.concrete_type();
                }
            }

            if let Types::Primitive(primitive) = definition {
                primitive.is_numeric_or_bool() && primitive.is_fixed_size()
            } else {
                false
            }
        }
    }
}

impl Type for Sequence {
    fn is_fixed_size(&self) -> bool {
        false
    }

    fn min_wire_size(&self) -> u32 {
        1
    }

    fn uses_classes(&self) -> bool {
        self.element_type.uses_classes()
    }

    fn is_class_type(&self) -> bool {
        false
    }

    fn tag_format(&self) -> TagFormat {
        if self.element_type.is_fixed_size() {
            if self.element_type.min_wire_size() == 1 {
                TagFormat::OVSize
            } else {
                TagFormat::VSize
            }
        } else {
            TagFormat::FSize
        }
    }

    fn supported_encodings(&self) -> SupportedEncodings {
        self.element_type.supported_encodings()
    }
}

implement_Element_for!(Sequence, "sequence");

#[derive(Debug)]
pub struct Dictionary {
    pub key_type: TypeRef,
    pub value_type: TypeRef,
}

impl Type for Dictionary {
    fn is_fixed_size(&self) -> bool {
        false
    }

    fn min_wire_size(&self) -> u32 {
        1
    }

    fn uses_classes(&self) -> bool {
        // It is illegal for key types to use classes, so we only have to check the value type.
        self.value_type.uses_classes()
    }

    fn is_class_type(&self) -> bool {
        false
    }

    fn tag_format(&self) -> TagFormat {
        if self.key_type.is_fixed_size() && self.value_type.is_fixed_size() {
            TagFormat::VSize
        } else {
            TagFormat::FSize
        }
    }

    fn supported_encodings(&self) -> SupportedEncodings {
        let mut encodings = self.key_type.supported_encodings();
        encodings.intersect_with(&self.value_type.supported_encodings());
        encodings
    }
}

implement_Element_for!(Dictionary, "dictionary");

#[derive(Debug)]
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
    AnyClass,
}

impl Primitive {
    pub fn is_numeric(&self) -> bool {
        matches!(self,
            Self::Byte | Self::Short | Self::UShort | Self::Int | Self::UInt | Self::VarInt |
            Self::VarUInt | Self::Long | Self::ULong | Self::VarLong | Self::VarULong |
            Self::Float | Self::Double
        )
    }

    pub fn is_unsigned_numeric(&self) -> bool {
        matches!(self,
            Self::Byte | Self::UShort | Self::UInt | Self::VarUInt | Self::ULong | Self::VarULong
        )
    }

    pub fn is_numeric_or_bool(&self) -> bool {
        self.is_numeric() || matches!(self, Self::Bool)
    }
}

impl Type for Primitive {
    fn is_fixed_size(&self) -> bool {
        matches!(self,
            Self::Bool | Self::Byte | Self::Short | Self::UShort | Self::Int | Self::UInt |
            Self::Long | Self::ULong | Self::Float | Self::Double
        )
    }

    fn min_wire_size(&self) -> u32 {
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
            Self::String => 1, // At least 1 byte for the empty string.
            Self::AnyClass => 1, // At least 1 byte to encode an index (instead of an instance).
        }
    }

    fn uses_classes(&self) -> bool {
        matches!(self, Self::AnyClass)
    }

    fn is_class_type(&self) -> bool {
        matches!(self, Self::AnyClass)
    }

    fn tag_format(&self) -> TagFormat {
        match self {
            Self::Bool     => TagFormat::F1,
            Self::Byte     => TagFormat::F1,
            Self::Short    => TagFormat::F2,
            Self::UShort   => TagFormat::F2,
            Self::Int      => TagFormat::F4,
            Self::UInt     => TagFormat::F4,
            Self::VarInt   => TagFormat::VInt,
            Self::VarUInt  => TagFormat::VInt,
            Self::Long     => TagFormat::F8,
            Self::ULong    => TagFormat::F8,
            Self::VarLong  => TagFormat::VInt,
            Self::VarULong => TagFormat::VInt,
            Self::Float    => TagFormat::F4,
            Self::Double   => TagFormat::F8,
            Self::String   => TagFormat::OVSize,
            Self::AnyClass => TagFormat::Class,
        }
    }

    fn supported_encodings(&self) -> SupportedEncodings {
        SupportedEncodings::new(match self {
            Self::Bool     => vec![Encoding::Slice11, Encoding::Slice2],
            Self::Byte     => vec![Encoding::Slice11, Encoding::Slice2],
            Self::Short    => vec![Encoding::Slice11, Encoding::Slice2],
            Self::UShort   => vec![Encoding::Slice2],
            Self::Int      => vec![Encoding::Slice11, Encoding::Slice2],
            Self::UInt     => vec![Encoding::Slice2],
            Self::VarInt   => vec![Encoding::Slice2],
            Self::VarUInt  => vec![Encoding::Slice2],
            Self::Long     => vec![Encoding::Slice11, Encoding::Slice2],
            Self::ULong    => vec![Encoding::Slice2],
            Self::VarLong  => vec![Encoding::Slice2],
            Self::VarULong => vec![Encoding::Slice2],
            Self::Float    => vec![Encoding::Slice11, Encoding::Slice2],
            Self::Double   => vec![Encoding::Slice11, Encoding::Slice2],
            Self::String   => vec![Encoding::Slice11, Encoding::Slice2],
            Self::AnyClass => vec![Encoding::Slice11],
        })
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
            Self::AnyClass => "any class",
        }
    }
}

#[derive(Clone, Debug)]
pub struct FileEncoding {
    pub version: Encoding,
    pub location: Location,
}

implement_Element_for!(FileEncoding, "file encoding");
implement_Symbol_for!(FileEncoding);

#[derive(Clone, Debug)]
pub struct Identifier {
    pub value: String,
    pub raw_value: String,
    pub location: Location,
}

impl Identifier {
    pub fn new(value: String, location: Location) -> Identifier {
        Identifier {
            value: value.trim_start_matches('\\').to_owned(), // Remove possible leading '\'.
            raw_value: value,
            location,
        }
    }
}

implement_Element_for!(Identifier, "identifier");
implement_Symbol_for!(Identifier);

#[derive(Clone, Debug)]
pub struct Attribute {
    pub prefix: Option<String>,
    pub directive: String,
    pub prefixed_directive: String,
    pub arguments: Vec<String>,
    pub location: Location,
}

impl Attribute {
    pub(crate) fn new(
        prefix: Option<String>,
        directive: String,
        arguments: Vec<String>,
        location: Location,
    ) -> Self {
        let prefixed_directive = prefix.clone().map_or(
            directive.clone(),                  // Default value if prefix == None
            |prefix| prefix + ":" + &directive, // Function to call if prefix == Some
        );
        Attribute { prefix, directive, prefixed_directive, arguments, location }
    }
}

implement_Element_for!(Attribute, "attribute");
implement_Symbol_for!(Attribute);
