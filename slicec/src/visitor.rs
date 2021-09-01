// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::ast::{Ast, Node};
use crate::grammar::*;
use crate::slice_file::SliceFile;

/// Base trait for all visitors. It provides default no-op implementations for all visitor methods.
///
/// These functions should never be called directly by code outside of this module.
/// Instead, visiting should be done by calling `visit_with` on grammar symbols, which will
/// automatically call the symbol's `x_start` and `x_end` visit functions, and visit through its
/// contents (if any).
// Keep parameter names for doc generation, even if they're unused in the default implementations.
#[allow(unused_variables)]
pub trait Visitor {
    fn visit_file_start(&mut self, file: &SliceFile, ast: &Ast) {}
    fn visit_file_end(&mut self, file: &SliceFile, ast: &Ast) {}

    fn visit_module_start(&mut self, module_def: &Module, index: usize, ast: &Ast) {}
    fn visit_module_end(&mut self, module_def: &Module, index: usize, ast: &Ast) {}
    fn visit_struct_start(&mut self, struct_def: &Struct, index: usize, ast: &Ast) {}
    fn visit_struct_end(&mut self, struct_def: &Struct, index: usize, ast: &Ast) {}
    fn visit_class_start(&mut self, class_def: &Class, index: usize, ast: &Ast) {}
    fn visit_class_end(&mut self, class_def: &Class, index: usize, ast: &Ast) {}
    fn visit_exception_start(&mut self, exception_def: &Exception, index: usize, ast: &Ast) {}
    fn visit_exception_end(&mut self, exception_def: &Exception, index: usize, ast: &Ast) {}
    fn visit_interface_start(&mut self, interface_def: &Interface, index: usize, ast: &Ast) {}
    fn visit_interface_end(&mut self, interface_def: &Interface, index: usize, ast: &Ast) {}
    fn visit_enum_start(&mut self, enum_def: &Enum, index: usize, ast: &Ast) {}
    fn visit_enum_end(&mut self, enum_def: &Enum, index: usize, ast: &Ast) {}

    fn visit_operation_start(&mut self, operation: &Operation, index: usize, ast: &Ast) {}
    fn visit_operation_end(&mut self, operation: &Operation, index: usize, ast: &Ast) {}

    fn visit_data_member(&mut self, data_member: &Member, index: usize, ast: &Ast) {}
    fn visit_parameter(&mut self, parameter: &Member, index: usize, ast: &Ast) {}
    fn visit_enumerator(&mut self, enumerator: &Enumerator, index: usize, ast: &Ast) {}

    fn visit_identifier(&mut self, identifier: &Identifier, ast: &Ast) {}

    // This only shallowly visits type-uses, and won't visit nested types. For instance, only the
    // outermost sequence will be visited in 'sequence<sequence<int>>'. Neither the inner sequence,
    // or int types will be visited by default.
    fn visit_type_use(&mut self, type_use: &TypeRef, ast: &Ast) {}
}

impl Node {
    pub fn visit_with(&self, visitor: &mut dyn Visitor, ast: &Ast) {
        // Forward the `visit` call to the underlying element.
        match self {
            Self::Module(index, module_def)       => module_def.visit_with(visitor, ast, *index),
            Self::Struct(index, struct_def)       => struct_def.visit_with(visitor, ast, *index),
            Self::Class(index, class_def)         => class_def.visit_with(visitor, ast, *index),
            Self::Exception(index, exception_def) => exception_def.visit_with(visitor, ast, *index),
            Self::Interface(index, interface_def) => interface_def.visit_with(visitor, ast, *index),
            Self::Enum(index, enum_def)           => enum_def.visit_with(visitor, ast, *index),
            Self::Operation(index, operation)     => operation.visit_with(visitor, ast, *index),
            Self::Member(index, member)           => member.visit_with(visitor, ast, *index),
            Self::Enumerator(index, enumerator)   => enumerator.visit_with(visitor, ast, *index),
            _ => {
                panic!("Node cannot be visited!\n{:?}", self)
            }
        }
    }
}

impl SliceFile {
    pub fn visit_with(&self, visitor: &mut dyn Visitor, ast: &Ast) {
        visitor.visit_file_start(self, ast);
        for id in self.contents.iter() {
            ast.resolve_index(*id).visit_with(visitor, ast);
        }
        visitor.visit_file_end(self, ast);
    }
}

impl Module {
    pub fn visit_with(&self, visitor: &mut dyn Visitor, ast: &Ast, index: usize) {
        visitor.visit_module_start(self, index, ast);
        for id in self.contents.iter() {
            ast.resolve_index(*id).visit_with(visitor, ast);
        }
        visitor.visit_module_end(self, index, ast);
    }
}

impl Struct {
    pub fn visit_with(&self, visitor: &mut dyn Visitor, ast: &Ast, index: usize) {
        visitor.visit_struct_start(self, index, ast);
        for id in self.members.iter() {
            ast.resolve_index(*id).visit_with(visitor, ast);
        }
        visitor.visit_struct_end(self, index, ast);
    }
}

impl Class {
    pub fn visit_with(&self, visitor: &mut dyn Visitor, ast: &Ast, index: usize) {
        visitor.visit_class_start(self, index, ast);
        for id in self.members.iter() {
            ast.resolve_index(*id).visit_with(visitor, ast);
        }
        visitor.visit_class_end(self, index, ast);
    }
}

impl Exception {
    pub fn visit_with(&self, visitor: &mut dyn Visitor, ast: &Ast, index: usize) {
        visitor.visit_exception_start(self, index, ast);
        for id in self.members.iter() {
            ast.resolve_index(*id).visit_with(visitor, ast);
        }
        visitor.visit_exception_end(self, index, ast);
    }
}

impl Interface {
    pub fn visit_with(&self, visitor: &mut dyn Visitor, ast: &Ast, index: usize) {
        visitor.visit_interface_start(self, index, ast);
        for id in self.operations.iter() {
            ast.resolve_index(*id).visit_with(visitor, ast);
        }
        visitor.visit_interface_end(self, index, ast);
    }
}

impl Enum {
    pub fn visit_with(&self, visitor: &mut dyn Visitor, ast: &Ast, index: usize) {
        visitor.visit_enum_start(self, index, ast);
        for id in self.enumerators.iter() {
            ast.resolve_index(*id).visit_with(visitor, ast);
        }
        visitor.visit_enum_end(self, index, ast);
    }
}

impl Operation {
    pub fn visit_with(&self, visitor: &mut dyn Visitor, ast: &Ast, index: usize) {
        visitor.visit_operation_start(self, index, ast);
        // We only visit the operation's parameters. Return types need to be visited manually.
        // This is because return types are not AST nodes, but are directly owned by operations.
        for id in self.parameters.iter() {
            ast.resolve_index(*id).visit_with(visitor, ast);
        }
        visitor.visit_operation_end(self, index, ast);
    }
}

impl Member {
    pub fn visit_with(&self, visitor: &mut dyn Visitor, ast: &Ast, index: usize) {
        match self.member_type {
            MemberType::DataMember => {
                visitor.visit_data_member(self, index, ast);
            }
            MemberType::Parameter => {
                visitor.visit_parameter(self, index, ast);
            }
            MemberType::ReturnElement => {
                panic!("return elements cannot be automatically visited");
            }
        }
        self.data_type.visit_with(visitor, ast);
    }
}

impl Enumerator {
    pub fn visit_with(&self, visitor: &mut dyn Visitor, ast: &Ast, index: usize) {
        visitor.visit_enumerator(self, index, ast);
    }
}

impl Identifier {
    pub fn visit_with(&self, visitor: &mut dyn Visitor, ast: &Ast) {
        visitor.visit_identifier(self, ast);
    }
}

impl TypeRef {
    pub fn visit_with(&self, visitor: &mut dyn Visitor, ast: &Ast) {
        visitor.visit_type_use(self, ast);
    }
}
