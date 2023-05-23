// Copyright (c) ZeroC, Inc.

use crate::grammar::*;
use crate::slice_file::SliceFile;

/// The `Visitor` trait is used to recursively visit through a tree of slice elements.
/// It automatically traverses through the tree, calling the various `visit_x` methods as applicable.
///
/// Implementors don't need to implement the tree traversal or recursive visitation. This is handled automatically.
///
/// These methods are purely for the visitor's use, and shouldn't be called directly.
/// To actually visit an element, call `visit_with` on the element.
///
/// When a container is visited, first its `visit_x` method is called, then its
/// contents are recursively visited.
/// For example, calling `visit_with` on a module containing a single struct would invoke:
/// - visit_module
///     - visit_struct
///         - visit_field (called once per field, in the order they're defined)
pub trait Visitor {
    /// This function is called by the visitor when it begins visiting a slice file,
    /// before it visits through the file's contents.
    ///
    /// This shouldn't be called by users. To visit a slice file, use `[SliceFile::visit_with]`.
    fn visit_file(&mut self, slice_file: &SliceFile);

    /// This function is called by the visitor when it begins visiting a [Module],
    /// before it visits through the module's contents.
    ///
    /// This shouldn't be called by users. To visit a module, use `[Module::visit_with]`.
    fn visit_module(&mut self, module_def: &Module);

    /// This function is called by the visitor when it begins visiting a [Struct],
    /// before it visits through the struct's contents.
    ///
    /// This shouldn't be called by users. To visit a struct, use `[Struct::visit_with]`.
    fn visit_struct(&mut self, struct_def: &Struct);

    /// This function is called by the visitor when it begins visiting a [Class],
    /// before it visits through the class' contents.
    ///
    /// This shouldn't be called by users. To visit a class, use `[Class::visit_with]`.
    fn visit_class(&mut self, class_def: &Class);

    /// This function is called by the visitor when it begins visiting an [Exception],
    /// before it visits through the exception's contents.
    ///
    /// This shouldn't be called by users. To visit an exception, use `[Exception::visit_with]`.
    fn visit_exception(&mut self, exception_def: &Exception);

    /// This function is called by the visitor when it begins visiting an [Interface],
    /// before it visits through the interface's contents.
    ///
    /// This shouldn't be called by users. To visit an interface, use `[Interface::visit_with]`.
    fn visit_interface(&mut self, interface_def: &Interface);

    /// This function is called by the visitor when it begins visiting an [Enum],
    /// before it visits through the enum's contents.
    ///
    /// This shouldn't be called by users. To visit an enum, use `[Enum::visit_with]`.
    fn visit_enum(&mut self, enum_def: &Enum);

    /// This function is called by the visitor when it begins visiting an [Operation],
    /// before it visits through the operation's contents.
    ///
    /// This shouldn't be called by users. To visit an operation, use `[Operation::visit_with]`.
    fn visit_operation(&mut self, operation: &Operation);

    /// This function is called by the visitor when it visits a [CustomType],
    ///
    /// This shouldn't be called by users. To visit a custom type, use `[CustomType::visit_with]`.
    fn visit_custom_type(&mut self, custom_type: &CustomType);

    /// This function is called by the visitor when it visits a [TypeAlias],
    ///
    /// This shouldn't be called by users. To visit a type alias, use `[TypeAlias::visit_with]`.
    fn visit_type_alias(&mut self, type_alias: &TypeAlias);

    /// This function is called by the visitor when it visits a [Field],
    ///
    /// This shouldn't be called by users. To visit a field, use `[Field::visit_with]`.
    fn visit_field(&mut self, field: &Field);

    /// This function is called by the visitor when it visits a [Parameter],
    ///
    /// This shouldn't be called by users. To visit a parameter, use `[Parameter::visit_with]`.
    fn visit_parameter(&mut self, parameter: &Parameter);

    /// This function is called by the visitor when it visits a [Enumerator],
    ///
    /// This shouldn't be called by users. To visit an enumerator, use `[Enumerator::visit_with]`.
    fn visit_enumerator(&mut self, enumerator: &Enumerator);

    // TODO: This can probably be improved after splitting `TypeRef`. See https://github.com/icerpc/slicec/issues/452.
    /// This function is called by the visitor when it visits a [TypeRef].
    ///
    /// This shouldn't be called by users. To visit a type reference, use `[TypeRef::visit_with]`.
    fn visit_type_ref(&mut self, type_ref: &TypeRef);
}

impl SliceFile {
    /// Visits the [SliceFile] with the provided `visitor`.
    ///
    /// This function first calls `visitor.visit_file`, then recursively visits
    /// the top level modules in the file.
    pub fn visit_with(&self, visitor: &mut impl Visitor) {
        visitor.visit_file(self);
        for module_def in &self.contents {
            module_def.borrow().visit_with(visitor);
        }
    }
}

impl Module {
    /// Visits the [Module] with the provided `visitor`.
    ///
    /// This function first calls `visitor.visit_module`, then recursively visits
    /// the contents of the module.
    pub fn visit_with(&self, visitor: &mut impl Visitor) {
        visitor.visit_module(self);
        for definition in &self.contents {
            match definition {
                Definition::Struct(struct_def) => struct_def.borrow().visit_with(visitor),
                Definition::Class(class_def) => class_def.borrow().visit_with(visitor),
                Definition::Exception(exception_def) => exception_def.borrow().visit_with(visitor),
                Definition::Interface(interface_def) => interface_def.borrow().visit_with(visitor),
                Definition::Enum(enum_def) => enum_def.borrow().visit_with(visitor),
                Definition::CustomType(custom_type) => custom_type.borrow().visit_with(visitor),
                Definition::TypeAlias(type_alias) => type_alias.borrow().visit_with(visitor),
            }
        }
    }
}

impl Struct {
    /// Visits the [Struct] with the provided `visitor`.
    ///
    /// This function first calls `visitor.visit_struct`, then recursively visits
    /// the contents of the struct.
    pub fn visit_with(&self, visitor: &mut impl Visitor) {
        visitor.visit_struct(self);
        for field in &self.fields {
            field.borrow().visit_with(visitor);
        }
    }
}

impl Class {
    /// Visits the [Class] with the provided `visitor`.
    ///
    /// This function first calls `visitor.visit_class`, then recursively visits
    /// the contents of the class.
    pub fn visit_with(&self, visitor: &mut impl Visitor) {
        visitor.visit_class(self);
        for field in &self.fields {
            field.borrow().visit_with(visitor);
        }
    }
}

impl Exception {
    /// Visits the [Exception] with the provided `visitor`.
    ///
    /// This function first calls `visitor.visit_exception`, then recursively visits
    /// the contents of the exception.
    pub fn visit_with(&self, visitor: &mut impl Visitor) {
        visitor.visit_exception(self);
        for field in &self.fields {
            field.borrow().visit_with(visitor);
        }
    }
}

impl Interface {
    /// Visits the [Interface] with the provided `visitor`.
    ///
    /// This function first calls `visitor.visit_interface`, then recursively visits
    /// the contents of the interface.
    pub fn visit_with(&self, visitor: &mut impl Visitor) {
        visitor.visit_interface(self);
        for operation in &self.operations {
            operation.borrow().visit_with(visitor);
        }
    }
}

impl Enum {
    /// Visits the [Enum] with the provided `visitor`.
    ///
    /// This function first calls `visitor.visit_enum`, then recursively visits
    /// the contents of the enum.
    pub fn visit_with(&self, visitor: &mut impl Visitor) {
        visitor.visit_enum(self);
        for enumerators in &self.enumerators {
            enumerators.borrow().visit_with(visitor);
        }
    }
}

impl Operation {
    /// Visits the [Operation] with the provided `visitor`.
    ///
    /// This function first calls `visitor.visit_operation`, then recursively visits
    /// the contents of the operation.
    pub fn visit_with(&self, visitor: &mut impl Visitor) {
        visitor.visit_operation(self);
        for parameter in &self.parameters {
            parameter.borrow().visit_with(visitor)
        }
        for return_member in &self.return_type {
            return_member.borrow().visit_with(visitor)
        }
    }
}

impl CustomType {
    /// Visits the [CustomType] with the provided `visitor`.
    ///
    /// This function delegates to `visitor.visit_custom_type`.
    pub fn visit_with(&self, visitor: &mut impl Visitor) {
        visitor.visit_custom_type(self);
    }
}

impl TypeAlias {
    /// Visits the [TypeAlias] with the provided `visitor`.
    ///
    /// This function delegates to `visitor.visit_type_alias`.
    pub fn visit_with(&self, visitor: &mut impl Visitor) {
        visitor.visit_type_alias(self);
        self.underlying.visit_with(visitor);
    }
}

impl Field {
    /// Visits the [Field] with the provided `visitor`.
    ///
    /// This function delegates to `visitor.visit_field`.
    pub fn visit_with(&self, visitor: &mut impl Visitor) {
        visitor.visit_field(self);
        self.data_type.visit_with(visitor);
    }
}

impl Parameter {
    /// Visits the [Parameter] with the provided `visitor`.
    ///
    /// This function delegates to `visitor.visit_parameter`
    pub fn visit_with(&self, visitor: &mut impl Visitor) {
        visitor.visit_parameter(self);
        self.data_type.visit_with(visitor);
    }
}

impl Enumerator {
    /// Visits the [Enumerator] with the provided `visitor`.
    ///
    /// This function delegates to `visitor.visit_enumerator`.
    pub fn visit_with(&self, visitor: &mut impl Visitor) {
        visitor.visit_enumerator(self);
    }
}

impl TypeRef {
    /// Visits the [TypeRef] with the provided `visitor`.
    ///
    /// This function first calls `visitor.visit_type_ref`, then if the type being referenced is a sequence or
    /// dictionary, it recursively calls itself on their underlying element, key, and value types.
    pub fn visit_with(&self, visitor: &mut impl Visitor) {
        visitor.visit_type_ref(self);
        match self.concrete_type() {
            Types::Sequence(sequence_ref) => sequence_ref.element_type.visit_with(visitor),
            Types::Dictionary(dictionary_ref) => {
                dictionary_ref.key_type.visit_with(visitor);
                dictionary_ref.value_type.visit_with(visitor);
            }
            _ => {}
        }
    }
}
