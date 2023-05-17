// Copyright (c) ZeroC, Inc.

use clap::Parser;
use slice::slice_options::SliceOptions;
use std::process::exit;

pub fn main() {
    let options = SliceOptions::parse();
    let state = slice::compile_from_options(&options, |_| {}, |_| {});
    exit(state.into_exit_code());
}
