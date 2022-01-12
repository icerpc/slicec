// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::grammar::*;
use crate::slice_file::SliceFile;

/// The `Visitor` trait is used to recursively visit through a tree of slice elements.
///
/// It automatically traverses through the tree, calling the various `visit_x` methods as applicable.
/// Elements that implement [Container] have 2 corresponding methods, `visit_x_start` and `visit_x_end`.
/// Non-container elements only have a single method: `visit_x`.
///
/// These methods are default implemented as no-ops, so implementors are free to only implement the
/// methods they need. Implementors also don't need to implement the tree traversal or recursive
/// visitation. This is handled automatically.
///
/// These methods are purely for the visitor's use, and shouldn't be called directly.
/// To actually visit an element, call `visit_with` on the element.
///
/// When a container is visited, first its `visit_x_start` method is called, then its
/// contents are recursively visited, and finally, its `visit_x_end` method is called.
/// For example, calling `visit_with` on a module containing a single struct would invoke:
/// - visit_module_start
///     - visit_struct_start
///         - visit_data_member (called once per data member, in the order they're defined)
///     - visit_struct_end
/// - visit_module_end
///
/// `Visitor` visits through and exposes immutable references to elements.
/// If mutability or access to the element's enclosing pointers is needed, use [PtrVisitor] instead.
#[allow(unused_variables)] // Keep parameter names for doc generation, even if not used in the default implementations.
pub trait Visitor {
    /// This function is called by the visitor when it begins visiting a slice file,
    /// before it visits through the file's contents.
    ///
    /// This shouldn't be called by users. To visit a slice file, use `[SliceFile::visit_with]`.
    fn visit_file_start(&mut self, slice_file: &SliceFile) {}

    /// This function is called by the visitor when it finishes visiting a slice file,
    /// after it has visited through the file's contents.
    ///
    /// This shouldn't be called by users. To visit a slice file, use `[SliceFile::visit_with]`.
    fn visit_file_end(&mut self, slice_file: &SliceFile) {}

    /// This function is called by the visitor when it begins visiting a [Module],
    /// before it visits through the module's contents.
    ///
    /// This shouldn't be called by users. To visit a module, use `[Module::visit_with]`.
    fn visit_module_start(&mut self, module_def: &Module) {}

    /// This function is called by the visitor when it finishes visiting a [Module],
    /// after it has visited through the module's contents.
    ///
    /// This shouldn't be called by users. To visit a module, use `[Module::visit_with]`.
    fn visit_module_end(&mut self, module_def: &Module) {}

    /// This function is called by the visitor when it begins visiting a [Struct],
    /// before it visits through the struct's contents.
    ///
    /// This shouldn't be called by users. To visit a struct, use `[Struct::visit_with]`.
    fn visit_struct_start(&mut self, struct_def: &Struct) {}

    /// This function is called by the visitor when it finishes visiting a [Struct],
    /// after it has visited through the struct's contents.
    ///
    /// This shouldn't be called by users. To visit a struct, use `[Struct::visit_with]`.
    fn visit_struct_end(&mut self, struct_def: &Struct) {}

    /// This function is called by the visitor when it begins visiting a [Class],
    /// before it visits through the class' contents.
    ///
    /// This shouldn't be called by users. To visit a class, use `[Class::visit_with]`.
    fn visit_class_start(&mut self, class_def: &Class) {}

    /// This function is called by the visitor when it finishes visiting a [Class],
    /// after it has visited through the class' contents.
    ///
    /// This shouldn't be called by users. To visit a class, use `[Class::visit_with]`.
    fn visit_class_end(&mut self, class_def: &Class) {}

    /// This function is called by the visitor when it begins visiting an [Exception],
    /// before it visits through the exception's contents.
    ///
    /// This shouldn't be called by users. To visit an exception, use `[Exception::visit_with]`.
    fn visit_exception_start(&mut self, exception_def: &Exception) {}

    /// This function is called by the visitor when it finishes visiting an [Exception],
    /// after it has visited through the exception's contents.
    ///
    /// This shouldn't be called by users. To visit an exception, use `[Exception::visit_with]`.
    fn visit_exception_end(&mut self, exception_def: &Exception) {}

    /// This function is called by the visitor when it begins visiting an [Interface],
    /// before it visits through the interface's contents.
    ///
    /// This shouldn't be called by users. To visit an interface, use `[Interface::visit_with]`.
    fn visit_interface_start(&mut self, interface_def: &Interface) {}

    /// This function is called by the visitor when it finishes visiting an [Interface],
    /// after it has visited through the interface's contents.
    ///
    /// This shouldn't be called by users. To visit an interface, use `[Interface::visit_with]`.
    fn visit_interface_end(&mut self, interface_def: &Interface) {}

    /// This function is called by the visitor when it begins visiting an [Enum],
    /// before it visits through the enum's contents.
    ///
    /// This shouldn't be called by users. To visit an enum, use `[Enum::visit_with]`.
    fn visit_enum_start(&mut self, enum_def: &Enum) {}

    /// This function is called by the visitor when it finishes visiting an [Enum],
    /// after it has visited through the enum's contents.
    ///
    /// This shouldn't be called by users. To visit an enum, use `[Enum::visit_with]`.
    fn visit_enum_end(&mut self, enum_def: &Enum) {}

    /// This function is called by the visitor when it begins visiting an [Operation],
    /// before it visits through the operation's contents.
    ///
    /// This shouldn't be called by users. To visit an operation, use `[Operation::visit_with]`.
    fn visit_operation_start(&mut self, operation: &Operation) {}

    /// This function is called by the visitor when it finishes visiting an [Operation],
    /// after it has visited through the operation's contents.
    ///
    /// This shouldn't be called by users. To visit an operation, use `[Operation::visit_with]`.
    fn visit_operation_end(&mut self, operation: &Operation) {}

    /// This function is called by the visitor when it visits a [Trait],
    ///
    /// This shouldn't be called by users. To visit a trait, use `[Trait::visit_with]`.
    fn visit_trait(&mut self, trait_def: &Trait) {}

    /// This function is called by the visitor when it visits a [TypeAlias],
    ///
    /// This shouldn't be called by users. To visit a type alias, use `[TypeAlias::visit_with]`.
    fn visit_type_alias(&mut self, type_alias: &TypeAlias) {}

    /// This function is called by the visitor when it visits a [DataMember],
    ///
    /// This shouldn't be called by users. To visit a data member, use `[DataMember::visit_with]`.
    fn visit_data_member(&mut self, data_member: &DataMember) {}

    /// This function is called by the visitor when it visits a [Parameter],
    ///
    /// This shouldn't be called by users. To visit a parameter, use `[Parameter::visit_with]`.
    fn visit_parameter(&mut self, parameter: &Parameter) {}

    /// This function is called by the visitor when it visits a [return member](Parameter),
    ///
    /// This shouldn't be called by users. To visit a return member, use `[Parameter::visit_with]`.
    fn visit_return_member(&mut self, parameter: &Parameter) {}

    /// This function is called by the visitor when it visits a [Enumerator],
    ///
    /// This shouldn't be called by users. To visit an enumerator, use `[Enumerator::visit_with]`.
    fn visit_enumerator(&mut self, enumerator: &Enumerator) {}
}

impl SliceFile {
    /// Visits the [SliceFile] with the provided `visitor`.
    ///
    /// This function first calls `visitor.visit_file_start`, then recursively visits
    /// the top level modules in the file, and finally calls `visitor.visit_file_end`.
    pub fn visit_with(&self, visitor: &mut impl Visitor) {
        visitor.visit_file_start(self);
        for module_def in &self.contents {
            module_def.borrow().visit_with(visitor);
        }
        visitor.visit_file_end(self);
    }
}

impl Module {
    /// Visits the [Module] with the provided `visitor`.
    ///
    /// This function first calls `visitor.visit_module_start`, then recursively visits
    /// the contents of the module, and finally calls `visitor.visit_module_end`.
    ///
    /// If mutability or access to the module's owning pointer are needed,
    /// use [OwnedPtr<Module>::visit_ptr_with] instead.
    pub fn visit_with(&self, visitor: &mut impl Visitor) {
        visitor.visit_module_start(self);
        for definition in &self.contents {
            match definition {
                Definition::Module(module_def)       => module_def.borrow().visit_with(visitor),
                Definition::Struct(struct_def)       => struct_def.borrow().visit_with(visitor),
                Definition::Class(class_def)         => class_def.borrow().visit_with(visitor),
                Definition::Exception(exception_def) => exception_def.borrow().visit_with(visitor),
                Definition::Interface(interface_def) => interface_def.borrow().visit_with(visitor),
                Definition::Enum(enum_def)           => enum_def.borrow().visit_with(visitor),
                Definition::Trait(trait_def)         => trait_def.borrow().visit_with(visitor),
                Definition::TypeAlias(type_alias)    => type_alias.borrow().visit_with(visitor),
            }
        }
        visitor.visit_module_end(self);
    }
}

impl Struct {
    /// Visits the [Struct] with the provided `visitor`.
    ///
    /// This function first calls `visitor.visit_struct_start`, then recursively visits
    /// the contents of the struct, and finally calls `visitor.visit_struct_end`.
    ///
    /// If mutability or access to the struct's owning pointer are needed,
    /// use [OwnedPtr<Struct>::visit_ptr_with] instead.
    pub fn visit_with(&self, visitor: &mut impl Visitor) {
        visitor.visit_struct_start(self);
        for data_member in &self.members {
            data_member.borrow().visit_with(visitor);
        }
        visitor.visit_struct_end(self);
    }
}

impl Class {
    /// Visits the [Class] with the provided `visitor`.
    ///
    /// This function first calls `visitor.visit_class_start`, then recursively visits
    /// the contents of the class, and finally calls `visitor.visit_class_end`.
    ///
    /// If mutability or access to the class' owning pointer are needed,
    /// use [OwnedPtr<Class>::visit_ptr_with] instead.
    pub fn visit_with(&self, visitor: &mut impl Visitor) {
        visitor.visit_class_start(self);
        for data_member in &self.members {
            data_member.borrow().visit_with(visitor);
        }
        visitor.visit_class_end(self);
    }
}

impl Exception {
    /// Visits the [Exception] with the provided `visitor`.
    ///
    /// This function first calls `visitor.visit_exception_start`, then recursively visits
    /// the contents of the exception, and finally calls `visitor.visit_exception_end`.
    ///
    /// If mutability or access to the exception's owning pointer are needed,
    /// use [OwnedPtr<Exception>::visit_ptr_with] instead.
    pub fn visit_with(&self, visitor: &mut impl Visitor) {
        visitor.visit_exception_start(self);
        for data_member in &self.members {
            data_member.borrow().visit_with(visitor);
        }
        visitor.visit_exception_end(self);
    }
}

impl Interface {
    /// Visits the [Interface] with the provided `visitor`.
    ///
    /// This function first calls `visitor.visit_interface_start`, then recursively visits
    /// the contents of the interface, and finally calls `visitor.visit_interface_end`.
    ///
    /// If mutability or access to the interface's owning pointer are needed,
    /// use [OwnedPtr<Interface>::visit_ptr_with] instead.
    pub fn visit_with(&self, visitor: &mut impl Visitor) {
        visitor.visit_interface_start(self);
        for operation in &self.operations {
            operation.borrow().visit_with(visitor);
        }
        visitor.visit_interface_end(self);
    }
}

impl Enum {
    /// Visits the [Enum] with the provided `visitor`.
    ///
    /// This function first calls `visitor.visit_enum_start`, then recursively visits
    /// the contents of the enum, and finally calls `visitor.visit_enum_end`.
    ///
    /// If mutability or access to the enum's owning pointer are needed,
    /// use [OwnedPtr<Enum>::visit_ptr_with] instead.
    pub fn visit_with(&self, visitor: &mut impl Visitor) {
        visitor.visit_enum_start(self);
        for enumerators in &self.enumerators {
            enumerators.borrow().visit_with(visitor);
        }
        visitor.visit_enum_end(self);
    }
}

impl Operation {
    /// Visits the [Operation] with the provided `visitor`.
    ///
    /// This function first calls `visitor.visit_operation_start`, then recursively visits
    /// the contents of the operation, and finally calls `visitor.visit_operation_end`.
    ///
    /// If mutability or access to the operation's owning pointer are needed,
    /// use [OwnedPtr<Operation>::visit_ptr_with] instead.
    pub fn visit_with(&self, visitor: &mut impl Visitor) {
        visitor.visit_operation_start(self);
        for parameter in &self.parameters {
            parameter.borrow().visit_with(visitor, true);
        }
        for return_members in &self.return_type {
            return_members.borrow().visit_with(visitor, false);
        }
        visitor.visit_operation_end(self);
    }
}

impl Trait {
    /// Visits the [Trait] with the provided `visitor`.
    ///
    /// This function delegates to `visitor.visit_trait`.
    ///
    /// If mutability or access to the trait's owning pointer are needed,
    /// use [OwnedPtr<Trait>::visit_ptr_with] instead.
    pub fn visit_with(&self, visitor: &mut impl Visitor) {
        visitor.visit_trait(self);
    }
}

impl TypeAlias {
    /// Visits the [TypeAlias] with the provided `visitor`.
    ///
    /// This function delegates to `visitor.visit_type_alias`.
    ///
    /// If mutability or access to the type alias' owning pointer are needed,
    /// use [OwnedPtr<TypeAlias>::visit_ptr_with] instead.
    pub fn visit_with(&self, visitor: &mut impl Visitor) {
        visitor.visit_type_alias(self);
    }
}

impl DataMember {
    /// Visits the [DataMember] with the provided `visitor`.
    ///
    /// This function delegates to `visitor.visit_data_member`.
    ///
    /// If mutability or access to the data member's owning pointer are needed,
    /// use [OwnedPtr<DataMember>::visit_ptr_with] instead.
    pub fn visit_with(&self, visitor: &mut impl Visitor) {
        visitor.visit_data_member(self);
    }
}

impl Parameter {
    /// Visits the [Parameter] with the provided `visitor`.
    ///
    /// This function delegates to `visitor.visit_parameter` for parameters,
    /// and `visitor.visit_return_member` for return members. It handles both
    /// cases because both semantic types are implemented by the [Parameter] struct.
    ///
    /// If mutability or access to the parameter's owning pointer are needed,
    /// use [OwnedPtr<Parameter>::visit_ptr_with] instead.
    pub fn visit_with(&self, visitor: &mut impl Visitor, is_parameter: bool) {
        if is_parameter {
            visitor.visit_parameter(self);
        } else {
            visitor.visit_return_member(self);
        }
    }
}

impl Enumerator {
    /// Visits the [Enumerator] with the provided `visitor`.
    ///
    /// This function delegates to `visitor.visit_enumerator`.
    ///
    /// If mutability or access to the enumerator's owning pointer are needed,
    /// use [OwnedPtr<Enumerator>::visit_ptr_with] instead.
    pub fn visit_with(&self, visitor: &mut impl Visitor) {
        visitor.visit_enumerator(self);
    }
}
