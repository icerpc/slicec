// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::code_map::CodeMap;
use slice::ast::Ast;
use slice::grammar::*;
use slice::slice_file::SliceFile;
use slice::visitor::Visitor;
use slice::writer::Writer;

macro_rules! write_fmt {
    ($writer:expr, $fmt:literal, $($arg:tt)*) => {{
        let content = format!($fmt, $($arg)*);
        $writer.write(&content);
    }};
}

pub struct CsWriter<'a> {
    pub output: &'a mut Writer,
    pub code_map: &'a mut CodeMap,
    pub empty_namespace_prefix: Option<String>,
}

impl Visitor for CsWriter<'_> {
    fn visit_file_start(&mut self, slice_file: &SliceFile, _: &Ast) {
        write_fmt!(
            self.output,
            "\
// Copyright (c) ZeroC, Inc. All rights reserved.

// <auto-generated/>
// slicec-cs version: '{version}'
// Generated from file: '{file}.ice'

#nullable enable

#pragma warning disable 1591 // Missing XML Comment",
            version = env!("CARGO_PKG_VERSION"),
            file = slice_file.filename
        );
    }

    fn visit_file_end(&mut self, _: &SliceFile, _: &Ast) {
        self.output.write("\n")
    }

    fn visit_module_start(&mut self, module_def: &Module, _: usize, _: &Ast) {
        let code_blocks = self.code_map.get(module_def);

        if code_blocks.is_none() {
            if let Some(prefix) = self.empty_namespace_prefix.clone() {
                self.empty_namespace_prefix = Some(prefix + "." + module_def.identifier());
            } else {
                self.empty_namespace_prefix = Some(module_def.identifier().to_owned())
            }

            return;
        }

        // TODO: Are there doc comments for C# modules?
        // write_comment(&mut self.output, module_def);

        let module = if let Some(prefix) = self.empty_namespace_prefix.clone() {
            self.empty_namespace_prefix = None;
            prefix + "." + module_def.identifier()
        } else {
            module_def.identifier().to_owned()
        };

        let content = format!("\nnamespace {}\n{{", module);
        self.output.write(&content);
        self.output.indent_by(4);

        if let Some(vec) = code_blocks {
            for code in vec {
                self.output.write("\n");
                write_fmt!(self.output, "{}", code);
                self.output.write_line_separator();
            }
        }
    }

    fn visit_module_end(&mut self, module_def: &Module, _: usize, _: &Ast) {
        let code_blocks = self.code_map.get(module_def);
        if code_blocks.is_none() {
            return;
        }

        self.output.clear_line_separator();
        self.output.indent_by(-4);
        self.output.write("\n}");
        self.output.write_line_separator();
    }
}
