// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::grammar::*;
use crate::ptr_util::OwnedPtr;

/// The `PtrVisitor` trait is used to recursively visit through a tree of slice elements.
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
/// To actually visit an element, call `visit_ptr_with` on the element's pointer.
///
/// When a container is visited, first its `visit_x_start` method is called, then its
/// contents are recursively visited, and finally, its `visit_x_end` method is called.
/// For example, calling `visit_ptr_with` on a module containing a single struct would invoke:
/// - visit_module_start
///     - visit_struct_start
///         - visit_data_member (called once per data member, in the order they're defined)
///     - visit_struct_end
/// - visit_module_end
///
/// `PtrVisitor` is the lower-level sibling of [Visitor]. They both visit through element trees in
/// the same manner, but [Visitor] exposes immutable references to elements while visiting, whereas
/// `PtrVisitor` exposes *mutable* references to the [OwnedPtr]s that hold the elements instead.
///
/// This allows `PtrVisitor` to mutate elements while visiting, as well as work with their pointers.
/// These provide greater flexibility than `Visitor`s methods do, but introduce unsafety.
///
/// The trait methods are pre-emptively marked as unsafe, even though they aren't inherently unsafe.
/// They are marked unsafe to
///  - A) Allow unsafe behavior to occur in them without needing an extra `unsafe` block.
///  - B) To signal that you should only be using this trait if you know what you're doing.
/// It is **absolutely IMPERATIVE** that no other borrows exist to the elements that are being
/// visited, and that no mixing of mutable and immutable borrows occurs in the trait implementation.
/// This is **Undefined Behavior** that will silently sabotage the compiler.
/// For more information on this, see the documentation for [OwnedPtr].
#[allow(unused_variables)] // Keep parameter names for doc generation, even if not used in the default implementations.
pub trait PtrVisitor {
    /// This function is called by the visitor when it begins visiting a [Module],
    /// before it visits through the module's contents.
    ///
    /// This shouldn't be called by users. To visit a module, use `module_ptr.visit_ptr_with`.
    ///
    /// # Safety
    ///
    /// Implementors of this function must be able to safely borrow the module being visited,
    /// mutably and immutably. Hence, this function is only safe to call when the module isn't
    /// borrowed elsewhere. Violating this **will** lead to undefined behavior.
    unsafe fn visit_module_start(&mut self, module_ptr: &mut OwnedPtr<Module>) {}

    /// This function is called by the visitor when it finishes visiting a [Module],
    /// after it has visited through the module's contents.
    ///
    /// This shouldn't be called by users. To visit a module, use `module_ptr.visit_ptr_with`.
    ///
    /// # Safety
    ///
    /// Implementors of this function must be able to safely borrow the module being visited,
    /// mutably and immutably. Hence, this function is only safe to call when the module isn't
    /// borrowed elsewhere. Violating this **will** lead to undefined behavior.
    unsafe fn visit_module_end(&mut self, module_ptr: &mut OwnedPtr<Module>) {}

    /// This function is called by the visitor when it begins visiting a [Struct],
    /// before it visits through the structs' contents.
    ///
    /// This shouldn't be called by users. To visit a struct, use `struct_ptr.visit_ptr_with`.
    ///
    /// # Safety
    ///
    /// Implementors of this function must be able to safely borrow the struct being visited,
    /// mutably and immutably. Hence, this function is only safe to call when the struct isn't
    /// borrowed elsewhere. Violating this **will** lead to undefined behavior.
    unsafe fn visit_struct_start(&mut self, struct_ptr: &mut OwnedPtr<Struct>) {}

    /// This function is called by the visitor when it finishes visiting a [Struct],
    /// after it has visited through the struct' contents.
    ///
    /// This shouldn't be called by users. To visit a struct, use `struct_ptr.visit_ptr_with`.
    ///
    /// # Safety
    ///
    /// Implementors of this function must be able to safely borrow the struct being visited,
    /// mutably and immutably. Hence, this function is only safe to call when the struct isn't
    /// borrowed elsewhere. Violating this **will** lead to undefined behavior.
    unsafe fn visit_struct_end(&mut self, struct_ptr: &mut OwnedPtr<Struct>) {}

    /// This function is called by the visitor when it begins visiting a [Class],
    /// before it visits through the class' contents.
    ///
    /// This shouldn't be called by users. To visit a class, use `class_ptr.visit_ptr_with`.
    ///
    /// # Safety
    ///
    /// Implementors of this function must be able to safely borrow the class being visited,
    /// mutably and immutably. Hence, this function is only safe to call when the class isn't
    /// borrowed elsewhere. Violating this **will** lead to undefined behavior.
    unsafe fn visit_class_start(&mut self, class_ptr: &mut OwnedPtr<Class>) {}

    /// This function is called by the visitor when it finishes visiting a [Class],
    /// after it has visited through the class' contents.
    ///
    /// This shouldn't be called by users. To visit a class, use `class_ptr.visit_ptr_with`.
    ///
    /// # Safety
    ///
    /// Implementors of this function must be able to safely borrow the class being visited,
    /// mutably and immutably. Hence, this function is only safe to call when the class isn't
    /// borrowed elsewhere. Violating this **will** lead to undefined behavior.
    unsafe fn visit_class_end(&mut self, class_ptr: &mut OwnedPtr<Class>) {}

    /// This function is called by the visitor when it begins visiting an [Exception],
    /// before it visits through the exception's contents.
    ///
    /// This shouldn't be called by users. To visit a exception, use `exception_ptr.visit_ptr_with`.
    ///
    /// # Safety
    ///
    /// Implementors of this function must be able to safely borrow the exception being visited,
    /// mutably and immutably. Hence, this function is only safe to call when the exception isn't
    /// borrowed elsewhere. Violating this **will** lead to undefined behavior.
    unsafe fn visit_exception_start(&mut self, exception_ptr: &mut OwnedPtr<Exception>) {}

    /// This function is called by the visitor when it finishes visiting an [Exception],
    /// after it has visited through the exception's contents.
    ///
    /// This shouldn't be called by users. To visit a exception, use `exception_ptr.visit_ptr_with`.
    ///
    /// # Safety
    ///
    /// Implementors of this function must be able to safely borrow the exception being visited,
    /// mutably and immutably. Hence, this function is only safe to call when the exception isn't
    /// borrowed elsewhere. Violating this **will** lead to undefined behavior.
    unsafe fn visit_exception_end(&mut self, exception_ptr: &mut OwnedPtr<Exception>) {}

    /// This function is called by the visitor when it begins visiting an [Interface],
    /// before it visits through the interface's contents.
    ///
    /// This shouldn't be called by users. To visit a interface, use `interface_ptr.visit_ptr_with`.
    ///
    /// # Safety
    ///
    /// Implementors of this function must be able to safely borrow the interface being visited,
    /// mutably and immutably. Hence, this function is only safe to call when the interface isn't
    /// borrowed elsewhere. Violating this **will** lead to undefined behavior.
    unsafe fn visit_interface_start(&mut self, interface_ptr: &mut OwnedPtr<Interface>) {}

    /// This function is called by the visitor when it finishes visiting an [Interface],
    /// after it has visited through the interface's contents.
    ///
    /// This shouldn't be called by users. To visit a interface, use `interface_ptr.visit_ptr_with`.
    ///
    /// # Safety
    ///
    /// Implementors of this function must be able to safely borrow the interface being visited,
    /// mutably and immutably. Hence, this function is only safe to call when the interface isn't
    /// borrowed elsewhere. Violating this **will** lead to undefined behavior.
    unsafe fn visit_interface_end(&mut self, interface_ptr: &mut OwnedPtr<Interface>) {}

    /// This function is called by the visitor when it begins visiting an [Enum],
    /// before it visits through the enum's contents.
    ///
    /// This shouldn't be called by users. To visit a enum, use `enum_ptr.visit_ptr_with`.
    ///
    /// # Safety
    ///
    /// Implementors of this function must be able to safely borrow the enum being visited,
    /// mutably and immutably. Hence, this function is only safe to call when the enum isn't
    /// borrowed elsewhere. Violating this **will** lead to undefined behavior.
    unsafe fn visit_enum_start(&mut self, enum_ptr: &mut OwnedPtr<Enum>) {}

    /// This function is called by the visitor when it finishes visiting an [Enum],
    /// after it has visited through the enum's contents.
    ///
    /// This shouldn't be called by users. To visit a enum, use `enum_ptr.visit_ptr_with`.
    ///
    /// # Safety
    ///
    /// Implementors of this function must be able to safely borrow the enum being visited,
    /// mutably and immutably. Hence, this function is only safe to call when the enum isn't
    /// borrowed elsewhere. Violating this **will** lead to undefined behavior.
    unsafe fn visit_enum_end(&mut self, enum_ptr: &mut OwnedPtr<Enum>) {}

    /// This function is called by the visitor when it begins visiting an [Operation],
    /// before it visits through the operation's contents.
    ///
    /// This shouldn't be called by users. To visit a operation, use `operation_ptr.visit_ptr_with`.
    ///
    /// # Safety
    ///
    /// Implementors of this function must be able to safely borrow the operation being visited,
    /// mutably and immutably. Hence, this function is only safe to call when the operation isn't
    /// borrowed elsewhere. Violating this **will** lead to undefined behavior.
    unsafe fn visit_operation_start(&mut self, operation_ptr: &mut OwnedPtr<Operation>) {}

    /// This function is called by the visitor when it finishes visiting an [Operation],
    /// after it has visited through the operation's contents.
    ///
    /// This shouldn't be called by users. To visit a operation, use `operation_ptr.visit_ptr_with`.
    ///
    /// # Safety
    ///
    /// Implementors of this function must be able to safely borrow the operation being visited,
    /// mutably and immutably. Hence, this function is only safe to call when the operation isn't
    /// borrowed elsewhere. Violating this **will** lead to undefined behavior.
    unsafe fn visit_operation_end(&mut self, operation_ptr: &mut OwnedPtr<Operation>) {}

    /// This function is called by the visitor when it visits a [Trait].
    ///
    /// This shouldn't be called by users. To visit a trait, use `trait_ptr.visit_ptr_with`.
    ///
    /// # Safety
    ///
    /// Implementors of this function must be able to safely borrow the trait being visited,
    /// mutably and immutably. Hence, this function is only safe to call when the trait isn't
    /// borrowed elsewhere. Violating this **will** lead to undefined behavior.
    unsafe fn visit_trait(&mut self, trait_ptr: &mut OwnedPtr<Trait>) {}

    /// This function is called by the visitor when it visits a [TypeAlias].
    ///
    /// This shouldn't be called by users. To visit a type alias, use `type_alias_ptr.visit_ptr_with`.
    ///
    /// # Safety
    ///
    /// Implementors of this function must be able to safely borrow the type alias being visited,
    /// mutably and immutably. Hence, this function is only safe to call when the type alias isn't
    /// borrowed elsewhere. Violating this **will** lead to undefined behavior.
    unsafe fn visit_type_alias(&mut self, type_alias_ptr: &mut OwnedPtr<TypeAlias>) {}

    /// This function is called by the visitor when it visits a [DataMember].
    ///
    /// This shouldn't be called by users. To visit a data member, use `member_ptr.visit_ptr_with`.
    ///
    /// # Safety
    ///
    /// Implementors of this function must be able to safely borrow the data member being visited,
    /// mutably and immutably. Hence, this function is only safe to call when the data member isn't
    /// borrowed elsewhere. Violating this **will** lead to undefined behavior.
    unsafe fn visit_data_member(&mut self, data_member_ptr: &mut OwnedPtr<DataMember>) {}

    /// This function is called by the visitor when it visits a [Parameter].
    ///
    /// This shouldn't be called by users. To visit a parameter, use `parameter_ptr.visit_ptr_with`.
    ///
    /// # Safety
    ///
    /// Implementors of this function must be able to safely borrow the parameter being visited,
    /// mutably and immutably. Hence, this function is only safe to call when the parameter isn't
    /// borrowed elsewhere. Violating this **will** lead to undefined behavior.
    unsafe fn visit_parameter(&mut self, parameter_ptr: &mut OwnedPtr<Parameter>) {}

    /// This function is called by the visitor when it visits a [return member](Parameter).
    ///
    /// This shouldn't be called by users. To visit a return member, use `member_ptr.visit_ptr_with`.
    ///
    /// # Safety
    ///
    /// Implementors of this function must be able to safely borrow the return member being visited,
    /// mutably and immutably. Hence, this function is only safe to call when the return member isn't
    /// borrowed elsewhere. Violating this **will** lead to undefined behavior.
    unsafe fn visit_return_member(&mut self, parameter_ptr: &mut OwnedPtr<Parameter>) {}

    /// This function is called by the visitor when it visits an [Enumerator].
    ///
    /// This shouldn't be called by users. To visit a enumerator, use `type_alias_ptr.visit_ptr_with`.
    ///
    /// # Safety
    ///
    /// Implementors of this function must be able to safely borrow the enumerator being visited,
    /// mutably and immutably. Hence, this function is only safe to call when the enumerator isn't
    /// borrowed elsewhere. Violating this **will** lead to undefined behavior.
    unsafe fn visit_enumerator(&mut self, enumerator_ptr: &mut OwnedPtr<Enumerator>) {}
}

impl OwnedPtr<Module> {
    /// Uses the provided `visitor` to visit a [Module] through its enclosing [OwnedPtr].
    ///
    /// This function first calls `visitor.visit_module_start`, then recursively visits
    /// the contents of the module, and finally calls `visitor.visit_module_end`.
    ///
    /// It takes a mutable borrow of the pointer, to allow the visitor to modify the pointer and its
    /// contents. If you don't need mutability or pointer access, use [Module::visit_with] instead.
    ///
    /// # Safety
    ///
    /// The implementation of this function mutably borrows the underlying module to visit on.
    /// Hence, it is only safe to call this function when the module isn't borrowed elsewhere.
    /// Violating this **will** lead to undefined behavior. Check [OwnedPtr] for more information.
    pub unsafe fn visit_ptr_with(&mut self, visitor: &mut impl PtrVisitor) {
        visitor.visit_module_start(self);
        for definition in &mut self.borrow_mut().contents {
            match definition {
                Definition::Module(module_ptr)        => module_ptr.visit_ptr_with(visitor),
                Definition::Struct(struct_ptr)        => struct_ptr.visit_ptr_with(visitor),
                Definition::Class(class_ptr)          => class_ptr.visit_ptr_with(visitor),
                Definition::Exception(exception_ptr)  => exception_ptr.visit_ptr_with(visitor),
                Definition::Interface(interface_ptr)  => interface_ptr.visit_ptr_with(visitor),
                Definition::Enum(enum_ptr)            => enum_ptr.visit_ptr_with(visitor),
                Definition::Trait(trait_ptr)          => trait_ptr.visit_ptr_with(visitor),
                Definition::TypeAlias(type_alias_ptr) => type_alias_ptr.visit_ptr_with(visitor),
            }
        }
        visitor.visit_module_end(self);
    }
}

impl OwnedPtr<Struct> {
    /// Uses the provided `visitor` to visit a [Struct] through its enclosing [OwnedPtr].
    ///
    /// This function first calls `visitor.visit_struct_start`, then recursively visits
    /// the contents of the struct, and finally calls `visitor.visit_struct_end`.
    ///
    /// It takes a mutable borrow of the pointer, to allow the visitor to modify the pointer and its
    /// contents. If you don't need mutability or pointer access, use [Struct::visit_with] instead.
    ///
    /// # Safety
    ///
    /// The implementation of this function mutably borrows the underlying struct to visit on.
    /// Hence, it is only safe to call this function when the struct isn't borrowed elsewhere.
    /// Violating this **will** lead to undefined behavior. Check [OwnedPtr] for more information.
    pub unsafe fn visit_ptr_with(&mut self, visitor: &mut impl PtrVisitor) {
        visitor.visit_struct_start(self);
        for data_member in &mut self.borrow_mut().members {
            data_member.visit_ptr_with(visitor);
        }
        visitor.visit_struct_end(self);
    }
}

impl OwnedPtr<Class> {
    /// Uses the provided `visitor` to visit a [Class] through its enclosing [OwnedPtr].
    ///
    /// This function first calls `visitor.visit_class_start`, then recursively visits
    /// the contents of the class, and finally calls `visitor.visit_class_end`.
    ///
    /// It takes a mutable borrow of the pointer, to allow the visitor to modify the pointer and its
    /// contents. If you don't need mutability or pointer access, use [Class::visit_with] instead.
    ///
    /// # Safety
    ///
    /// The implementation of this function mutably borrows the underlying class to visit on.
    /// Hence, it is only safe to call this function when the class isn't borrowed elsewhere.
    /// Violating this **will** lead to undefined behavior. Check [OwnedPtr] for more information.
    pub unsafe fn visit_ptr_with(&mut self, visitor: &mut impl PtrVisitor) {
        visitor.visit_class_start(self);
        for data_member in &mut self.borrow_mut().members {
            data_member.visit_ptr_with(visitor);
        }
        visitor.visit_class_end(self);
    }
}

impl OwnedPtr<Exception> {
    /// Uses the provided `visitor` to visit an [Exception] through its enclosing [OwnedPtr].
    ///
    /// This function first calls `visitor.visit_exception_start`, then recursively visits
    /// the contents of the exception, and finally calls `visitor.visit_exception_end`.
    ///
    /// It takes a mutable borrow of the pointer, to allow the visitor to modify the pointer and its
    /// contents. If you don't need mutability or pointer access, use [Exception::visit_with] instead.
    ///
    /// # Safety
    ///
    /// The implementation of this function mutably borrows the underlying exception to visit on.
    /// Hence, it is only safe to call this function when the exception isn't borrowed elsewhere.
    /// Violating this **will** lead to undefined behavior. Check [OwnedPtr] for more information.
    pub unsafe fn visit_ptr_with(&mut self, visitor: &mut impl PtrVisitor) {
        visitor.visit_exception_start(self);
        for data_member in &mut self.borrow_mut().members {
            data_member.visit_ptr_with(visitor);
        }
        visitor.visit_exception_end(self);
    }
}

impl OwnedPtr<Interface> {
    /// Uses the provided `visitor` to visit an [Interface] through its enclosing [OwnedPtr].
    ///
    /// This function first calls `visitor.visit_interface_start`, then recursively visits
    /// the contents of the interface, and finally calls `visitor.visit_interface_end`.
    ///
    /// It takes a mutable borrow of the pointer, to allow the visitor to modify the pointer and its
    /// contents. If you don't need mutability or pointer access, use [Interface::visit_with] instead.
    ///
    /// # Safety
    ///
    /// The implementation of this function mutably borrows the underlying interface to visit on.
    /// Hence, it is only safe to call this function when the interface isn't borrowed elsewhere.
    /// Violating this **will** lead to undefined behavior. Check [OwnedPtr] for more information.
    pub unsafe fn visit_ptr_with(&mut self, visitor: &mut impl PtrVisitor) {
        visitor.visit_interface_start(self);
        for operation in &mut self.borrow_mut().operations {
            operation.visit_ptr_with(visitor);
        }
        visitor.visit_interface_end(self);
    }
}

impl OwnedPtr<Enum> {
    /// Uses the provided `visitor` to visit an [Enum] through its enclosing [OwnedPtr].
    ///
    /// This function first calls `visitor.visit_enum_start`, then recursively visits
    /// the contents of the enum, and finally calls `visitor.visit_enum_end`.
    ///
    /// It takes a mutable borrow of the pointer, to allow the visitor to modify the pointer and its
    /// contents. If you don't need mutability or pointer access, use [Enum::visit_with] instead.
    ///
    /// # Safety
    ///
    /// The implementation of this function mutably borrows the underlying enum to visit on.
    /// Hence, it is only safe to call this function when the enum isn't borrowed elsewhere.
    /// Violating this **will** lead to undefined behavior. Check [OwnedPtr] for more information.
    pub unsafe fn visit_ptr_with(&mut self, visitor: &mut impl PtrVisitor) {
        visitor.visit_enum_start(self);
        for enumerators in &mut self.borrow_mut().enumerators {
            enumerators.visit_ptr_with(visitor);
        }
        visitor.visit_enum_end(self);
    }
}

impl OwnedPtr<Operation> {
    /// Uses the provided `visitor` to visit an [Operation] through its enclosing [OwnedPtr].
    ///
    /// This function first calls `visitor.visit_operation_start`, then recursively visits
    /// the contents of the operation, and finally calls `visitor.visit_operation_end`.
    ///
    /// It takes a mutable borrow of the pointer, to allow the visitor to modify the pointer and its
    /// contents. If you don't need mutability or pointer access, use [Operation::visit_with] instead.
    ///
    /// # Safety
    ///
    /// The implementation of this function mutably borrows the underlying operation to visit on.
    /// Hence, it is only safe to call this function when the operation isn't borrowed elsewhere.
    /// Violating this **will** lead to undefined behavior. Check [OwnedPtr] for more information.
    pub unsafe fn visit_ptr_with(&mut self, visitor: &mut impl PtrVisitor) {
        visitor.visit_operation_start(self);
        for parameter in &mut self.borrow_mut().parameters {
            parameter.visit_ptr_with(visitor, true);
        }
        for return_members in &mut self.borrow_mut().return_type {
            return_members.visit_ptr_with(visitor, false);
        }
        visitor.visit_operation_end(self);
    }
}

impl OwnedPtr<Trait> {
    /// Uses the provided `visitor` to visit a [Trait] through its enclosing [OwnedPtr].
    ///
    /// This function delegates to `visitor.visit_trait`.
    ///
    /// It takes a mutable borrow of the pointer, to allow the visitor to modify the pointer and its
    /// contents. If you don't need mutability or pointer access, use [Trait::visit_with] instead.
    ///
    /// # Safety
    ///
    /// The implementation of this function mutably borrows the underlying trait to visit on.
    /// Hence, it is only safe to call this function when the trait isn't borrowed elsewhere.
    /// Violating this **will** lead to undefined behavior. Check [OwnedPtr] for more information.
    pub unsafe fn visit_ptr_with(&mut self, visitor: &mut impl PtrVisitor) {
        visitor.visit_trait(self);
    }
}

impl OwnedPtr<TypeAlias> {
    /// Uses the provided `visitor` to visit a [TypeAlias] through its enclosing [OwnedPtr].
    ///
    /// This function delegates to `visitor.visit_type_alias`.
    ///
    /// It takes a mutable borrow of the pointer, to allow the visitor to modify the pointer and its
    /// contents. If you don't need mutability or pointer access, use [TypeAlias::visit_with] instead.
    ///
    /// # Safety
    ///
    /// The implementation of this function mutably borrows the underlying type alias to visit on.
    /// Hence, it is only safe to call this function when the type alias isn't borrowed elsewhere.
    /// Violating this **will** lead to undefined behavior. Check [OwnedPtr] for more information.
    pub unsafe fn visit_ptr_with(&mut self, visitor: &mut impl PtrVisitor) {
        visitor.visit_type_alias(self);
    }
}

impl OwnedPtr<DataMember> {
    /// Uses the provided `visitor` to visit a [DataMember] through its enclosing [OwnedPtr].
    ///
    /// This function delegates to `visitor.visit_data_member`.
    ///
    /// It takes a mutable borrow of the pointer, to allow the visitor to modify the pointer and its
    /// contents. If you don't need mutability or pointer access, use [DataMember::visit_with] instead.
    ///
    /// # Safety
    ///
    /// The implementation of this function mutably borrows the underlying data member to visit on.
    /// Hence, it is only safe to call this function when the data member isn't borrowed elsewhere.
    /// Violating this **will** lead to undefined behavior. Check [OwnedPtr] for more information.
    pub unsafe fn visit_ptr_with(&mut self, visitor: &mut impl PtrVisitor) {
        visitor.visit_data_member(self);
    }
}

impl OwnedPtr<Parameter> {
    /// Uses the provided `visitor` to visit a [Parameter] through its enclosing [OwnedPtr].
    ///
    /// This function delegates to `visitor.visit_parameter` for parameters,
    /// and `visitor.visit_return_member` for return members. It handles both
    /// cases because both semantic types are implemented by the [Parameter] struct.
    ///
    /// It takes a mutable borrow of the pointer, to allow the visitor to modify the pointer and its
    /// contents. If you don't need mutability or pointer access, use [Parameter::visit_with] instead.
    ///
    /// # Safety
    ///
    /// The implementation of this function mutably borrows the underlying parameter to visit on.
    /// Hence, it is only safe to call this function when the parameter isn't borrowed elsewhere.
    /// Violating this **will** lead to undefined behavior. Check [OwnedPtr] for more information.
    pub unsafe fn visit_ptr_with(&mut self, visitor: &mut impl PtrVisitor, is_parameter: bool) {
        if is_parameter {
            visitor.visit_parameter(self);
        } else {
            visitor.visit_return_member(self);
        }
    }
}

impl OwnedPtr<Enumerator> {
    /// Uses the provided `visitor` to visit an [Enumerator] through its enclosing [OwnedPtr].
    ///
    /// This function delegates to `visitor.visit_enumerator`.
    ///
    /// It takes a mutable borrow of the pointer, to allow the visitor to modify the pointer and its
    /// contents. If you don't need mutability or pointer access, use [Enumerator::visit_with] instead.
    ///
    /// # Safety
    ///
    /// The implementation of this function mutably borrows the underlying enumerator to visit on.
    /// Hence, it is only safe to call this function when the enumerator isn't borrowed elsewhere.
    /// Violating this **will** lead to undefined behavior. Check [OwnedPtr] for more information.
    pub unsafe fn visit_ptr_with(&mut self, visitor: &mut impl PtrVisitor) {
        visitor.visit_enumerator(self);
    }
}
