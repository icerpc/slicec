// Copyright (c) ZeroC, Inc. All rights reserved.

//! This module pulls in the parsing code generated by LALRPOP and contains private helper functions used by it.
//!
//! While many of these functions could be written directly into the parser rules, we implement them here instead, to
//! keep the rules focused on grammar instead of implementation details, making the grammar easier to read and modify.

use super::super::common::SourceBlock;

use lalrpop_util::lalrpop_mod;

// Place the code generated by LALRPOP into a submodule named 'lalrpop'.
lalrpop_mod!(
    #[allow(unused, clippy::all)] // LALRPOP generates stuff we don't use, and isn't worth linting.
    pub lalrpop,
    "/parsers/preprocessor/grammar.rs"
);

/// Evaluates an if/elif/else statement and returns the source block contained by the first true conditional.
/// If none of the conditions are true, and an else block is present, its source block is returned instead.
/// If none of the conditions are true, and no else block is present, this function returns [None].
///
/// The `if` and `elif` blocks are passed in as tuples of their values (true or false) and their source blocks.
/// Since multiple (or zero) elif blocks can be present, they are passed as a [Vec] (in order).
/// Since there can only be 0 or 1 else block, it is passed as an [Option].
fn evaluate_if_statement<'a>(
    if_block: (bool, Vec<SourceBlock<'a>>),
    elif_blocks: Vec<(bool, Vec<SourceBlock<'a>>)>,
    else_block: Option<Vec<SourceBlock<'a>>>,
) -> Vec<SourceBlock<'a>> {
    // If the if-statement was true, return its block's content.
    if if_block.0 {
        return if_block.1;
    }
    // Check the elif statements in order. If one is true, return its block's content.
    for elif_block in elif_blocks {
        if elif_block.0 {
            return elif_block.1;
        }
    }
    // Otherwise, we return the content of the else block if it was present, if not, we return an empty vector.
    else_block.unwrap_or_default()
}
