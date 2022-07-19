// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::errors::*;

#[derive(Debug, Clone)]
struct Note {
    message: String,
}

impl ErrorType for Note {
    fn error_code(&self) -> u32 {
        0
    }

    fn message(&self) -> String {
        return self.message.clone();
    }

    fn severity(&self) -> ErrorLevel {
        ErrorLevel::Note
    }
}
