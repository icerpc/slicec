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
pub mod validator;
pub mod visitor;

use crate::ast::Ast;
use crate::command_line::SliceOptions;
use crate::error::ErrorLevel;
use crate::slice_file::{Location, SliceFile};
use std::collections::HashMap;

pub fn parse_from_options(options: &SliceOptions) -> Result<HashMap<String, SliceFile>, ()> {
    // Initialize the global instances of the `Ast` and the `ErrorHandler`.
    global_state::initialize();
    let slice_files = unsafe {
        parser::parse_files(borrow_mut_ast(), options)
    };
    handle_errors(options.warn_as_error, &slice_files)?;

    Ok(slice_files)
}

// TODO comments
pub fn borrow_ast() -> &'static Ast {
    unsafe { &*global_state::AST.get().unwrap().get() }
}

// TODO comments
pub unsafe fn borrow_mut_ast() -> &'static mut Ast {
    &mut *global_state::AST.get().unwrap().get()
}

pub fn report_note(message: String, location: Option<&Location>) {
    report_error_impl(message, location, ErrorLevel::Note);
}

pub fn report_warning(message: String, location: Option<&Location>) {
    report_error_impl(message, location, ErrorLevel::Warning);
}

pub fn report_error(message: String, location: Option<&Location>) {
    report_error_impl(message, location, ErrorLevel::Error);
}

pub fn report_critical(message: String, location: Option<&Location>) {
    report_error_impl(message, location, ErrorLevel::Critical);
}

fn report_error_impl(message: String, location: Option<&Location>, severity: ErrorLevel) {
    let error_reporter = unsafe { &mut *global_state::ERROR_REPORTER.get().unwrap().get() };
    error_reporter.report_error(message, location, severity);
}

pub fn handle_errors(
    warn_as_error: bool,
    slice_files: &HashMap<String, SliceFile>,
) -> Result<(), ()> {
    let error_reporter = unsafe { &mut *global_state::ERROR_REPORTER.get().unwrap().get() };

    error_reporter.print_errors(slice_files);
    if error_reporter.has_errors(warn_as_error) {
        let counts = error_reporter.get_totals();
        println!(
            "Compilation failed with {} error(s) and {} warning(s).\n",
            counts.0, counts.1
        );
        Err(())
    } else {
        Ok(())
    }
}

mod global_state {
    use crate::ast::Ast;
    use crate::error::ErrorReporter;
    use crate::ptr_util::ThreadSafe;
    use once_cell::unsync::OnceCell;
    use std::cell::UnsafeCell;

    type ThreadSafeCell<T> = ThreadSafe<OnceCell<UnsafeCell<T>>>;

    pub(super) static AST: ThreadSafeCell<Ast> = ThreadSafe(OnceCell::new());
    // TODO the error handler can be made a singleton, or put behind a RefCell even.
    pub(super) static ERROR_REPORTER: ThreadSafeCell<ErrorReporter> = ThreadSafe(OnceCell::new());

    pub(super) fn initialize() {
        let _ = AST.set(UnsafeCell::new(Ast::new()));
        let _ = ERROR_REPORTER.set(UnsafeCell::new(ErrorReporter::new()));
    }
}
