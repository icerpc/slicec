// Copyright (c) ZeroC, Inc.

use super::parser::Preprocessor;
use crate::diagnostics::Diagnostics;
use std::collections::HashSet;

#[test]
fn preprocessor_executes_directives_in_included_conditional_block() {
    // Arrange
    let slice = "
        #define FOO

        #if FOO
            #define BAR
            #undef FOO
            #if BAR
                #define BAZ
            #endif
        #endif
    ";
    let mut symbols = HashSet::new();
    let mut diagnostics = Diagnostics::new();
    let preprocessor = Preprocessor::new("string-0", &mut symbols, &mut diagnostics);

    // Act
    preprocessor.parse_slice_file(slice).unwrap().last();

    // Assert
    assert!(diagnostics.is_empty());
    assert_eq!(symbols, HashSet::from(["BAR".to_owned(), "BAZ".to_owned()]));
}

#[test]
fn preprocessor_skips_directives_in_omitted_conditional_block() {
    // Arrange
    let slice = "
        #define FOO

        // This entire block should be skipped.
        #if BAR
            #define BAZ
            #if FOO
                #undef FOO
            #endif
        #endif
    ";
    let mut symbols = HashSet::new();
    let mut diagnostics = Diagnostics::new();
    let preprocessor = Preprocessor::new("string-0", &mut symbols, &mut diagnostics);

    // Act
    preprocessor.parse_slice_file(slice).unwrap().last();

    // Assert
    assert!(diagnostics.is_empty());
    assert_eq!(symbols, HashSet::from(["FOO".to_owned()]));
}
