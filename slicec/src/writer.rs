// Copyright (c) ZeroC, Inc. All rights reserved.

use std::fmt;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::io::BufWriter;

#[derive(Debug)]
pub struct Writer {
    file_buffer: BufWriter<File>,
    indentation: String,
    line_separator_flag: bool,
    is_valid: bool,
}

impl Writer {
    pub fn new(path: &str) -> io::Result<Self> {
        let file = File::create(path)?;

        Ok(Writer {
            file_buffer: BufWriter::new(file),
            // Indentation starts with a \n, since when we actually write the indentation to a file,
            // it will always start with a newline (since that's where indentation is applied).
            indentation: "\n".to_owned(),
            line_separator_flag: false,
            is_valid: true,
        })
    }

    pub fn write(&mut self, content: &str) {
        if self.is_valid {
            let mut indented_content = str::replace(content, "\n", self.indentation.as_str());
            if self.line_separator_flag {
                self.line_separator_flag = false;
                indented_content.insert(0, '\n');
            }
            if let Err(error) = self.file_buffer.write_all(indented_content.as_bytes()) {
                eprintln!("{}", error);
                self.is_valid = false;
            }
        }
    }

    pub fn indent_by(&mut self, spaces: isize) {
        if spaces > 0 {
            self.indentation += " ".repeat(spaces as usize).as_str();
        } else {
            let new_size = self.indentation.len() - (spaces.abs() as usize);
            self.indentation.truncate(new_size);
        }
    }

    /// Instructs the writer to place an empty newline before the next string it writes to the file,
    /// to provide some space between two definitions to separate them.
    /// Just writing '\n' to the stream will introduce trailing whitespace since it's indented.
    /// Plus, using this line separator lets the writer be smart, and omit them when not needed.
    pub fn write_line_separator(&mut self) {
        self.line_separator_flag = true;
    }

    /// Clears any line separators that were set to be written to the file.
    pub fn clear_line_separator(&mut self) {
        self.line_separator_flag = false;
    }

    pub fn close(mut self) {
        if self.is_valid {
            if let Err(error) = self.file_buffer.flush() {
                eprintln!("{}", error);
            }
        }
    }

    /// Used to write code blocks using the write! and writeln! macros
    /// without results. Note that the write_fmt defined in fmt::Write and io::Write
    /// have a Result<()> return type.
    pub fn write_fmt(&mut self, args: fmt::Arguments<'_>) {
        if let Some(s) = args.as_str() {
            self.write(s);
        } else {
            self.write(&args.to_string());
        }
    }
}
