// Copyright (c) ZeroC, Inc. All rights reserved.

use slice::ast::Ast;
use slice::grammar::*;

use crate::code_block::CodeBlock;

pub fn encode_data_members(_: &Struct, _: &Ast) -> CodeBlock {
    CodeBlock::new()
}
