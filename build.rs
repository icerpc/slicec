// Copyright (c) ZeroC, Inc.

extern crate lalrpop;

fn main() {
    // Recursively finds any files ending with `.lalrpop` in the `src` directory and generates parsers from them.
    lalrpop::process_src().unwrap();
}
