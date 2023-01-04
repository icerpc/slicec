// Copyright (c) ZeroC, Inc. All rights reserved.

/// Asserts that an diagnostic reporter contains the expected errors (or is empty).
///
/// If the number of reporter errors doesn't match, this macro will print the values of
/// diagnostic reporter errors.
#[macro_export]
macro_rules! assert_errors {
    ($diagnostic_reporter:expr) => {
        let errors = $diagnostic_reporter.into_diagnostics();
        assert!(
            errors.is_empty(),
            "Expected no errors, got {}.\n{:?}",
            errors.len(),
            errors,
        );
    };

    ($diagnostic_reporter:expr, $expected_errors:expr) => {
        let errors = $diagnostic_reporter.into_diagnostics();
        assert_eq!(
            errors.len(),
            $expected_errors.len(),
            "Expected {} errors, got {}.\n{:?}",
            $expected_errors.len(),
            errors.len(),
            errors,
        );
        for (i, error) in errors.iter().enumerate() {
            assert_eq!(&error.message(), &$expected_errors[i].to_string());
        }
    };
}
