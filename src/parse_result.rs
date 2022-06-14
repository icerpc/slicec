// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::ast::Ast;
use crate::error::ErrorReporter;
use crate::slice_file::SliceFile;
use std::collections::HashMap;

pub struct ParsedData {
    pub ast: Ast,
    pub error_reporter: ErrorReporter,
    pub files: HashMap<String, SliceFile>,
}

impl ParsedData {
    pub fn has_errors(&self) -> bool {
        self.error_reporter.has_errors()
    }
}

impl From<ParsedData> for ParserResult {
    fn from(parsed_data: ParsedData) -> Self {
        match parsed_data.has_errors() {
            false => Ok(parsed_data),
            true => Err(parsed_data),
        }
    }
}

pub type ParserResult = Result<ParsedData, ParsedData>;
