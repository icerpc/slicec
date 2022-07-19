// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::errors::*;

#[derive(Debug, Clone)]
pub struct Note {
    pub message: String,
}

impl Note {
    pub fn new(message: impl Into<String>) -> Self {
        Note {
            message: message.into(),
        }
    }
}

impl ErrorType for Note {
    fn error_code(&self) -> u32 {
        0
    }

    fn message(&self) -> String {
        self.message.clone()
    }

    fn severity(&self) -> ErrorLevel {
        ErrorLevel::Note
    }
}
