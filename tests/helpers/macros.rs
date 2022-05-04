// Copyright (c) ZeroC, Inc. All rights reserved.

/// Asserts that an error reporter contains the expected errors (or is empty).
///
/// If the number of reporter errors doesn't match, this macro will print the values of
/// error reporter errors.
#[macro_export]
macro_rules! assert_errors {
    ($error_reporter:expr) => {
        assert!(
            !$error_reporter.has_errors(true),
            "Expected no errors, got {}.\n{:?}",
            $error_reporter.errors().len(),
            $error_reporter.errors()
        );
    };

    ($error_reporter:expr, $expected_errors:expr) => {
        assert_eq!(
            $error_reporter.errors().len(),
            $expected_errors.len(),
            "Expected {} errors, got {}.\n{:?}",
            $expected_errors.len(),
            $error_reporter.errors().len(),
            $error_reporter.errors(),
        );

        for (i, error) in $error_reporter.errors().iter().enumerate() {
            assert_eq!(error.message, $expected_errors[i]);
        }
    };
}
