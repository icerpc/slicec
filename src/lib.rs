// Copyright (c) ZeroC, Inc. All rights reserved.

pub mod ast;
pub mod code_gen_util;
pub mod command_line;
pub mod error;
pub mod grammar;
pub mod parser;
pub mod ptr_util;
pub mod ptr_visitor;
pub mod slice_file;
pub mod supported_encodings;
pub mod validator;
pub mod visitor;

use crate::ast::Ast;
use crate::command_line::SliceOptions;
use crate::error::{Error, ErrorLevel, ErrorReporter};
use crate::parser::parse_string;
use crate::slice_file::SliceFile;
use crate::validator::Validator;
use std::collections::HashMap;

pub fn parse_from_options(options: &SliceOptions) -> Result<(ErrorReporter, HashMap<String, SliceFile>), Error> {
    // Initialize the global instance of the `Ast`.
    global_state::initialize();

    let mut error_reporter = ErrorReporter::default();

    let slice_files = unsafe {
        parser::parse_files(options, borrow_mut_ast(), &mut error_reporter)?
    };
    handle_errors(options.warn_as_error, &slice_files, &mut error_reporter)?;

    let mut validator = Validator { error_reporter: &mut error_reporter };
    for slice_file in slice_files.values() {
        slice_file.visit_with(&mut validator);
    }

    Ok((error_reporter, slice_files))
}

pub fn parse_from_string(input: &str) -> Result<(Ast, ErrorReporter), Error> {
    let mut ast = Ast::new();
    let mut error_reporter = ErrorReporter::default();

    let slice_files = parse_string(input, &mut ast, &mut error_reporter)?;

    let mut validator = Validator { error_reporter: &mut error_reporter };

    for slice_file in slice_files.values() {
        slice_file.visit_with(&mut validator);
    }

    Ok((ast, error_reporter))
}

// TODO comments
pub fn borrow_ast() -> &'static Ast {
    unsafe { &*global_state::AST.get().unwrap().get() }
}

// TODO comments
/// # Safety
///
/// This can only be called when there are no other borrows.
pub unsafe fn borrow_mut_ast() -> &'static mut Ast {
    &mut *global_state::AST.get().unwrap().get()
}

pub fn handle_errors(
    warn_as_error: bool,
    slice_files: &HashMap<String, SliceFile>,
    error_reporter: &mut ErrorReporter,
) -> Result<(), Error> {
    error_reporter.print_errors(slice_files);
    if error_reporter.has_errors(warn_as_error) {
        let counts = error_reporter.get_totals();
        let message = format!(
            "Compilation failed with {} error(s) and {} warning(s).\n",
            counts.0, counts.1
        );

        println!("{}", &message);
        Err(Error{
            message,
            location: None,
            severity: ErrorLevel::Critical,
        })
    } else {
        Ok(())
    }
}

mod global_state {
    use crate::ast::Ast;
    use crate::ptr_util::ThreadSafe;
    use once_cell::unsync::OnceCell;
    use std::cell::UnsafeCell;

    type ThreadSafeCell<T> = ThreadSafe<OnceCell<UnsafeCell<T>>>;

    pub(super) static AST: ThreadSafeCell<Ast> = ThreadSafe(OnceCell::new());

    pub(super) fn initialize() {
        let _ = AST.set(UnsafeCell::new(Ast::new()));
    }
}
