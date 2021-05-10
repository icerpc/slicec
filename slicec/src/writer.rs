// Copyright (c) ZeroC, Inc. All rights reserved.

use std::fs::File;
use std::io;
use std::io::BufWriter;
use std::io::prelude::*;

#[derive(Debug)]
pub struct Writer {
    file_buffer: BufWriter<File>,
    indentation: String,
    is_valid: bool,
}

impl Writer {
    pub fn new(path: &str) -> io::Result<Self> {
        let file = File::create(path)?;

        Ok(Writer {
            file_buffer: BufWriter::new(file),
            indentation: "".to_owned(),
            is_valid: true,
        })
    }

    pub fn write_all(&mut self, bytes: &[u8]) {
        if self.is_valid {
            if let Err(error) = self.try_write_all(bytes) {
                eprintln!("{}", error);
                self.is_valid = false;
            }
        }
    }

    fn try_write_all(&mut self, bytes: &[u8]) -> io::Result<()> {
        self.file_buffer.write_all(self.indentation.as_bytes())?;
        self.file_buffer.write_all(bytes)
    }

    pub fn indent_by(&mut self, spaces: isize) {
        if spaces > 0 {
            self.indentation += " ".repeat(spaces as usize).as_str();
        } else {
            let new_size = self.indentation.len() - (spaces.abs() as usize);
            self.indentation.truncate(new_size);
        }
    }

    pub fn close(mut self) {
        if self.is_valid {
            if let Err(error) = self.file_buffer.flush() {
                eprintln!("{}", error);
            }
        }
    }
}
