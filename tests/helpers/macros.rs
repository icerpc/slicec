// Copyright (c) ZeroC, Inc. All rights reserved.

/// Asserts that an error reporter contains the expected errors (or is empty).
///
/// If the number of reporter errors doesn't match, this macro will print the values of
/// error reporter errors.
#[macro_export]
macro_rules! assert_errors {
    ($error_reporter:expr) => {
        let errors = $error_reporter.into_errors();
        assert!(
            errors.is_empty(),
            "Expected no errors, got {}.\n{:?}",
            errors.len(),
            errors,
        );
    };

    ($error_reporter:expr, $expected_errors:expr) => {
        let errors = $error_reporter.into_errors();
        assert_eq!(
            errors.len(),
            $expected_errors.len(),
            "Expected {} errors, got {}.\n{:?}",
            $expected_errors.len(),
            errors.len(),
            errors,
        );
        for (i, error) in errors.iter().enumerate() {
            assert_eq!(error.message, $expected_errors[i]);
        }
    };
}

#[macro_export]
macro_rules! assert_errors_new {
    ($error_reporter:expr, $expected_errors:expr) => {
        let errors = $error_reporter.into_errors();
        assert_eq!(
            errors.len(),
            $expected_errors.len(),
            "Expected {} errors, got {}.\n{:?}",
            $expected_errors.len(),
            errors.len(),
            errors,
        );
        for (i, error) in errors.iter().enumerate() {
            assert_eq!(&error.message, &$expected_errors[i].message());
        }
    };
}
