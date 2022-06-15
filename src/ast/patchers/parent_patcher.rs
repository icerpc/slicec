// Copyright (c) ZeroC, Inc. All rights reserved.

// TODO delete this entire file when we switch to LALRpop.

// This is poorly written, but that's fine. This is just a stop-gap to get the new AST working with
// the old parser. Once we switch to LALRpop (which is what I'm doing next), this will be completely
// unnecessary then, since we can just keep track of the parents while parsing, instead of patching
// them in later like this.

use super::super::Ast;
use super::super::node::Node;
use crate::downgrade_as;
use crate::grammar::*;
use crate::ptr_util::WeakPtr;
use std::collections::HashMap;

pub unsafe fn patch_ast(ast: &mut Ast) {
    let mut patches: HashMap<String, Patch> = HashMap::new();

    for node in ast.as_slice() {
        match node {
            Node::Module(module_ptr) => patches.insert(
                module_ptr.borrow().parser_scoped_identifier(),
                Patch::Module(module_ptr.downgrade()),
            ),
            Node::Struct(struct_ptr) => patches.insert(
                struct_ptr.borrow().parser_scoped_identifier(),
                Patch::DataMemberContainer(downgrade_as!(struct_ptr, dyn Container<WeakPtr<DataMember>>)),
            ),
            Node::Exception(exception_ptr) => patches.insert(
                exception_ptr.borrow().parser_scoped_identifier(),
                Patch::DataMemberContainer(downgrade_as!(exception_ptr, dyn Container<WeakPtr<DataMember>>)),
            ),
            Node::Class(class_ptr) => patches.insert(
                class_ptr.borrow().parser_scoped_identifier(),
                Patch::DataMemberContainer(downgrade_as!(class_ptr, dyn Container<WeakPtr<DataMember>>)),
            ),
            Node::Interface(interface_ptr) => patches.insert(
                interface_ptr.borrow().parser_scoped_identifier(),
                Patch::Interface(interface_ptr.downgrade()),
            ),
            Node::Operation(operation_ptr) => patches.insert(
                operation_ptr.borrow().parser_scoped_identifier(),
                Patch::Operation(operation_ptr.downgrade()),
            ),
            Node::Enum(enum_ptr) => patches.insert(
                enum_ptr.borrow().parser_scoped_identifier(),
                Patch::Enum(enum_ptr.downgrade()),
            ),
            _ => None,
        };
    }

    for node in ast.as_mut_slice() {
        match node {
            Node::Module(module_ptr) => {
                let module_def = module_ptr.borrow_mut();
                let parent_module_identifier = module_def.parser_scope();
                if !parent_module_identifier.is_empty() {
                    if let Patch::Module(parent_module_ptr) = patches.get(parent_module_identifier).unwrap() {
                        module_def.parent = Some(parent_module_ptr.clone());
                    } else {
                        panic!();
                    }
                }
            }
            Node::Struct(struct_ptr) => {
                let struct_def = struct_ptr.borrow_mut();
                if let Patch::Module(module_ptr) = patches.get(struct_def.parser_scope()).unwrap() {
                    struct_def.parent = module_ptr.clone();
                } else {
                    panic!();
                }
            }
            Node::Exception(exception_ptr) => {
                let exception_def = exception_ptr.borrow_mut();
                if let Patch::Module(module_ptr) = patches.get(exception_def.parser_scope()).unwrap() {
                    exception_def.parent = module_ptr.clone();
                } else {
                    panic!();
                }
            }
            Node::Class(class_ptr) => {
                let class_def = class_ptr.borrow_mut();
                if let Patch::Module(module_ptr) = patches.get(class_def.parser_scope()).unwrap() {
                    class_def.parent = module_ptr.clone();
                } else {
                    panic!();
                }
            }
            Node::Interface(interface_ptr) => {
                let interface_def = interface_ptr.borrow_mut();
                if let Patch::Module(module_ptr) = patches.get(interface_def.parser_scope()).unwrap() {
                    interface_def.parent = module_ptr.clone();
                } else {
                    panic!();
                }
            }
            Node::Enum(enum_ptr) => {
                let enum_def = enum_ptr.borrow_mut();
                if let Patch::Module(module_ptr) = patches.get(enum_def.parser_scope()).unwrap() {
                    enum_def.parent = module_ptr.clone();
                } else {
                    panic!();
                }
            }
            Node::Trait(trait_ptr) => {
                let trait_def = trait_ptr.borrow_mut();
                if let Patch::Module(module_ptr) = patches.get(trait_def.parser_scope()).unwrap() {
                    trait_def.parent = module_ptr.clone();
                } else {
                    panic!();
                }
            }
            Node::CustomType(custom_type_ptr) => {
                let custom_type_def = custom_type_ptr.borrow_mut();
                if let Patch::Module(module_ptr) = patches.get(custom_type_def.parser_scope()).unwrap() {
                    custom_type_def.parent = module_ptr.clone();
                } else {
                    panic!();
                }
            }
            Node::TypeAlias(type_alias_ptr) => {
                let type_alias_def = type_alias_ptr.borrow_mut();
                if let Patch::Module(module_ptr) = patches.get(type_alias_def.parser_scope()).unwrap() {
                    type_alias_def.parent = module_ptr.clone();
                } else {
                    panic!();
                }
            }
            Node::DataMember(data_member_ptr) => {
                let data_member_def = data_member_ptr.borrow_mut();
                if let Patch::DataMemberContainer(ptr) = patches.get(data_member_def.parser_scope()).unwrap() {
                    data_member_def.parent = ptr.clone();
                } else {
                    panic!();
                }
            }
            Node::Operation(operation_ptr) => {
                let operation_def = operation_ptr.borrow_mut();
                if let Patch::Interface(interface_ptr) = patches.get(operation_def.parser_scope()).unwrap() {
                    operation_def.parent = interface_ptr.clone();
                } else {
                    panic!();
                }
            }
            Node::Enumerator(enumerator_ptr) => {
                let enumerator_def = enumerator_ptr.borrow_mut();
                if let Patch::Enum(enum_ptr) = patches.get(enumerator_def.parser_scope()).unwrap() {
                    enumerator_def.parent = enum_ptr.clone();
                } else {
                    panic!();
                }
            }
            Node::Parameter(parameter_ptr) => {
                let parameter_def = parameter_ptr.borrow_mut();
                if let Patch::Operation(operation_ptr) = patches.get(parameter_def.parser_scope()).unwrap() {
                    parameter_def.parent = operation_ptr.clone();
                } else {
                    panic!();
                }
            }
            _ => {}
        }
    }
}

enum Patch {
    Module(WeakPtr<Module>),
    DataMemberContainer(WeakPtr<dyn Container<WeakPtr<DataMember>>>),
    Interface(WeakPtr<Interface>),
    Enum(WeakPtr<Enum>),
    Operation(WeakPtr<Operation>),
}
