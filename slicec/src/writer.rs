// Copyright (c) ZeroC, Inc. All rights reserved.

use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::io::BufWriter;

#[derive(Debug)]
pub struct Writer {
    file_buffer: BufWriter<File>,
    indentation: String,
    write_line_separator: bool,
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
            write_line_separator: false,
            is_valid: true,
        })
    }

    pub fn write_all(&mut self, content: &str) {
        if self.is_valid {
            let mut indented_content = str::replace(content, "\n", self.indentation.as_str());
            if self.write_line_separator {
                self.write_line_separator = false;
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
    pub fn write_line_seperator(&mut self) {
        self.write_line_separator = true;
    }

    /// Clears any line separators that were set to be written to the file.
    pub fn clear_line_separator(&mut self) {
        self.write_line_separator = false;
    }

    pub fn close(mut self) {
        if self.is_valid {
            if let Err(error) = self.file_buffer.flush() {
                eprintln!("{}", error);
            }
        }
    }
}
