// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::ast::Ast;
use crate::downgrade_as;
use crate::grammar::*;
use crate::ptr_util::{OwnedPtr, WeakPtr};
use crate::ptr_visitor::PtrVisitor;

pub(super) fn patch_parents(ast: &mut Ast) {
    let mut patcher = ParentPatcher;

    for module in &mut ast.ast {
        unsafe { module.visit_ptr_with(&mut patcher); }
    }
}

struct ParentPatcher;

impl PtrVisitor for ParentPatcher {
    unsafe fn visit_module_start(&mut self, module_ptr: &mut OwnedPtr<Module>) {
        let parent_ptr = module_ptr.downgrade();
        for definition in &mut module_ptr.borrow_mut().contents {
            match definition {
                Definition::Module(x)     => x.borrow_mut().parent = Some(parent_ptr.clone()),
                Definition::Struct(x)     => x.borrow_mut().parent = parent_ptr.clone(),
                Definition::Class(x)      => x.borrow_mut().parent = parent_ptr.clone(),
                Definition::Exception(x)  => x.borrow_mut().parent = parent_ptr.clone(),
                Definition::Interface(x)  => x.borrow_mut().parent = parent_ptr.clone(),
                Definition::Enum(x)       => x.borrow_mut().parent = parent_ptr.clone(),
                Definition::Trait(x)      => x.borrow_mut().parent = parent_ptr.clone(),
                Definition::CustomType(x) => x.borrow_mut().parent = parent_ptr.clone(),
                Definition::TypeAlias(x)  => x.borrow_mut().parent = parent_ptr.clone(),
            }
        }
    }

    unsafe fn visit_struct_start(&mut self, struct_ptr: &mut OwnedPtr<Struct>) {
        let parent_ptr = downgrade_as!(struct_ptr, dyn Container<OwnedPtr<DataMember>>);
        for data_member in &mut struct_ptr.borrow_mut().members {
            data_member.borrow_mut().parent = parent_ptr.clone();
        }
    }

    unsafe fn visit_class_start(&mut self, class_ptr: &mut OwnedPtr<Class>) {
        let parent_ptr = downgrade_as!(class_ptr, dyn Container<OwnedPtr<DataMember>>);
        for data_member in &mut class_ptr.borrow_mut().members {
            data_member.borrow_mut().parent = parent_ptr.clone();
        }
    }

    unsafe fn visit_exception_start(&mut self, exception_ptr: &mut OwnedPtr<Exception>) {
        let parent_ptr = downgrade_as!(exception_ptr, dyn Container<OwnedPtr<DataMember>>);
        for data_member in &mut exception_ptr.borrow_mut().members {
            data_member.borrow_mut().parent = parent_ptr.clone();
        }
    }

    unsafe fn visit_interface_start(&mut self, interface_ptr: &mut OwnedPtr<Interface>) {
        let parent_ptr = interface_ptr.downgrade();
        for operation in &mut interface_ptr.borrow_mut().operations {
            operation.borrow_mut().parent = parent_ptr.clone();
        }
    }

    unsafe fn visit_enum_start(&mut self, enum_ptr: &mut OwnedPtr<Enum>) {
        let parent_ptr = enum_ptr.downgrade();
        for enumerator in &mut enum_ptr.borrow_mut().enumerators {
            enumerator.borrow_mut().parent = parent_ptr.clone();
        }
    }

    unsafe fn visit_operation_start(&mut self, operation_ptr: &mut OwnedPtr<Operation>) {
        let parent_ptr = operation_ptr.downgrade();
        for parameter in &mut operation_ptr.borrow_mut().parameters {
            parameter.borrow_mut().parent = parent_ptr.clone();
        }
    }
}
