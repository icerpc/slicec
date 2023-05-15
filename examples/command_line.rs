// Copyright (c) ZeroC, Inc.

use clap::Parser;
use slice::command_line::SliceOptions;
use std::process::exit;

pub fn main() {
    let options = SliceOptions::parse();
    let state = slice::compile_from_options(&options);
    exit(state.into_exit_code());
}
