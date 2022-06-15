
use std::convert::Into;

/// Prefixes the provided string with either 'a' or 'an' depending on whether it starts with a vowel.
pub fn prefix_with_article(string: impl Into<String>) -> String {
    let mut string = string.into();
    if string.starts_with(is_ascii_vowel) {
        string.insert_str(0, "an ");
    } else {
        string.insert_str(0, "a ");
    }
    string
}

/// Returns true if the char is either an ASCII vowel ('a', 'e', 'i', 'o', or 'u').
pub fn is_ascii_vowel(c: char) -> bool {
    matches!(c, 'a' | 'e' | 'i' | 'o' | 'u')
}
