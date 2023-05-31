// Copyright (c) ZeroC, Inc.

/// Returns the indefinite article for the given word.
pub fn indefinite_article(s: &str) -> String {
    in_definite::get_a_or_an(s).to_lowercase()
}
