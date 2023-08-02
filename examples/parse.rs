// Copyright (c) ZeroC, Inc.

use clap::Parser;
use slicec::slice_options::SliceOptions;
use std::process::exit;

pub fn main() {
    let options = SliceOptions::parse();
    let state = slicec::compile_from_options(&options, |_| {}, |_| {});
    exit(i32::from(state.emit_diagnostics(&options)));
}
