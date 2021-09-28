// Copyright (c) ZeroC, Inc. All rights reserved.

use std::fmt;

use lazy_static::lazy_static;
use regex::{Captures, Regex};

#[derive(Clone, Debug)]
pub struct CodeBlock {
    pub content: String,
}

impl CodeBlock {
    pub fn new() -> CodeBlock {
        CodeBlock { content: String::new() }
    }

    pub fn write<T: fmt::Display + ?Sized>(&mut self, s: &T) {
        lazy_static! {
            // Match `<# condition statement #>` and anything that occurs before it
            static ref RE: Regex = Regex::new(
                r"(?s)(?P<before>.*?)<#\s+(?P<not>!)?(?P<bool>true|false)\s+(?P<statement>.*?)\s#>"
            )
            .unwrap();
        }

        let text = s.to_string();

        let result = RE.replace_all(&text, |caps: &Captures| {
            let before = caps.name("before").unwrap();

            let has_not_symbol = caps.name("not").is_some();

            let ok = match caps.name("bool").unwrap().as_str() {
                "true" => !has_not_symbol,
                "false" => has_not_symbol,
                _ => panic!("unexpected bool value"),
            };

            if ok {
                format!(
                    "{before}{statement}",
                    before = before.as_str(),
                    statement = caps.name("statement").unwrap().as_str()
                )
            } else {
                format!("{before}", before = before.as_str().trim_end())
            }
        });

        self.content.push_str(&result);
    }

    pub fn writeln<T: fmt::Display + ?Sized>(&mut self, s: &T) {
        self.write(&format!("{}\n", s));
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

    pub fn indent(&mut self) -> &mut Self {
        self.content = self.content.replace("\n", "\n    ");
        self
    }

    pub fn add_block(&mut self, block: &mut Self) {
        self.write(&format!("\n{}\n", block));
    }
}

impl fmt::Display for CodeBlock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.content.trim_matches(char::is_whitespace))
    }
}

impl std::iter::FromIterator<std::string::String> for CodeBlock {
    fn from_iter<T: IntoIterator<Item = std::string::String>>(iter: T) -> Self {
        let mut code = CodeBlock::new();
        for i in iter {
            code.writeln(&i);
        }
        code
    }
}

impl From<String> for CodeBlock {
    fn from(s: String) -> Self {
        CodeBlock { content: s }
    }
}

impl From<&str> for CodeBlock {
    fn from(s: &str) -> Self {
        CodeBlock { content: s.to_owned() }
    }
}
