// Copyright (c) ZeroC, Inc. All rights reserved.

/// Asserts that an diagnostic reporter contains the expected errors (or is empty).
///
/// If the number of reporter errors doesn't match, this macro will print the values of
/// diagnostic reporter errors.
#[macro_export]
macro_rules! assert_errors {
    ($diagnostics:expr) => {
        assert!(
            $diagnostics.is_empty(),
            "Expected no errors, got {}.\n{:?}",
            $diagnostics.len(),
            $diagnostics,
        );
    };

    ($diagnostics:expr, $expected_errors:expr) => {
        assert_eq!(
            $diagnostics.len(),
            $expected_errors.len(),
            "Expected {} errors, got {}.\n{:?}",
            $expected_errors.len(),
            $diagnostics.len(),
            $diagnostics,
        );
        for (i, error) in $diagnostics.iter().enumerate() {
            assert_eq!(&error.message(), &$expected_errors[i].to_string());
        }
    };
}
