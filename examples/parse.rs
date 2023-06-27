// Copyright (c) ZeroC, Inc.

use clap::Parser;
use slicec::slice_options::SliceOptions;
use std::process::exit;

pub fn main() {
    let options = SliceOptions::parse();
    let state = slicec::compile_from_options(&options, |_| {}, |_| {});
    exit(state.into_exit_code(&mut console::Term::stderr()));
}
