// Copyright (c) ZeroC, Inc.

use super::super::common::SourceBlock;
use super::tokens::*;
use crate::slice_file::Location;

use std::iter::Peekable;
use std::str::Chars;

type LexerResult<'a> = Result<Token<'a>, Error>;

/// Converts a string into a stream of tokens representing blocks of source code and preprocessor tokens.
///
/// This token stream is in turn consumed by the [preprocessor parser](super::parser::Preprocessor) which parses the
/// tokens and evaluates the preprocessor directives represented by them.
#[derive(Debug)]
pub struct Lexer<'input> {
    /// The string that this lexer is lexing over.
    input: &'input str,

    /// Iterator over the characters in the input string.
    /// This is what the lexer actually operates on, by peeking at and consuming codepoints from this buffer.
    buffer: Peekable<Chars<'input>>,

    /// The lexer's current position in the buffer.
    position: usize,

    /// The lexer's current [location](crate::slice_file::Location) in the input string.
    /// Used to tag tokens with their starting and ending locations in the input.
    cursor: Location,

    /// The current mode of the lexer; controls how the input is tokenized in a context-dependent manner.
    mode: LexerMode,
}

impl<'input> Lexer<'input> {
    /// Creates a new lexer over the provided input.
    pub fn new(input: &'input str) -> Self {
        Lexer {
            input,
            buffer: input.chars().peekable(),
            position: 0,
            cursor: Location::default(),
            mode: LexerMode::Unknown,
        }
    }

    /// Consumes the next character in the buffer and moves the lexer's cursor forward accordingly.
    fn advance_buffer(&mut self) {
        // Consume the next character and check if it's a newline.
        if let Some(c) = self.buffer.next() {
            self.position += 1;
            if c == '\n' {
                self.cursor.row += 1;
                self.cursor.col = 1;
            } else {
                self.cursor.col += 1;
            }
        }
    }

    /// Skips characters in the buffer until end-of-line (doesn't consume the EOL) or end-of-buffer is reached.
    /// After calling this function, the next char will be '\n' or `None` (end-of-buffer).
    fn advance_to_end_of_line(&mut self) {
        // Loop while the next character is not '\n'.
        while matches!(self.buffer.peek(), Some(c) if *c != '\n') {
            self.advance_buffer(); // Consume the character.
        }
    }

    /// Skips over inline whitespace characters (whitespace other than '\n') in the buffer.
    /// After calling this function, the next char will be '\n', a non-whitespace character, or `None` (end-of-buffer).
    fn skip_inline_whitespace(&mut self) {
        // Loop while the next character in the buffer is whitespace (except '\n').
        while matches!(self.buffer.peek(), Some(c) if (c.is_whitespace() && *c != '\n')) {
            self.advance_buffer(); // Consume the character.
        }
    }

    /// Reads, consumes, and returns a string of alphanumeric characters from the buffer.
    /// After calling this function, the next char will be a non-alphanumeric character or `None` (end-of-buffer).
    fn read_identifier(&mut self) -> &'input str {
        let start_position = self.position;

        // Loop while the next character in the buffer is an alphanumeric or underscore.
        while matches!(self.buffer.peek(), Some(c) if (c.is_alphanumeric() || *c == '_')) {
            self.advance_buffer(); // Consume the character.
        }

        &self.input[start_position..self.position]
    }

    /// Constructs and returns a preprocessor token representing a block of source code.
    /// This function assumes that the lexer's cursor is at the end of the token being created.
    fn create_source_block_token(
        &self,
        start_location: Location,
        start_position: usize,
        end_position: usize,
    ) -> Token<'input> {
        let source_block = TokenKind::SourceBlock(SourceBlock {
            content: &self.input[start_position..end_position],
            start: start_location,
            end: self.cursor,
        });
        (start_location, source_block, self.cursor)
    }

    /// Consumes a single character from the lexer's buffer and returns a token of the specified kind.
    /// This is a convenience function for the common case where a token's lexeme is a single character.
    fn return_simple_token(&mut self, token: TokenKind<'input>, start: Location) -> LexerResult<'input> {
        self.advance_buffer(); // Consume the token from the buffer.
        Ok((start, token, self.cursor)) // Return it.
    }

    /// Attempts to read and return a preprocessor directive token from the buffer.
    /// Returns `Ok(x)` to indicate success (`x` is the next token), and `Err(y)` to indicate an error occurred.
    fn lex_next_preprocessor_token(&mut self, c: char) -> Option<LexerResult<'input>> {
        let start_location = self.cursor;
        match c {
            '(' => Some(self.return_simple_token(TokenKind::LeftParenthesis, start_location)),
            ')' => Some(self.return_simple_token(TokenKind::RightParenthesis, start_location)),
            '!' => Some(self.return_simple_token(TokenKind::Not, start_location)),
            '&' => {
                self.advance_buffer(); // Consume the '&' character.
                                       // Ensure the next character is also an '&' (since the whole token should be "&&").
                if matches!(self.buffer.peek(), Some('&')) {
                    Some(self.return_simple_token(TokenKind::And, start_location))
                } else {
                    let error = ErrorKind::UnknownSymbol {
                        symbol: "&".to_owned(),
                        suggestion: Some("&&".to_owned()),
                    };
                    Some(Err((start_location, error, self.cursor)))
                }
            }
            '|' => {
                self.advance_buffer(); // Consume the '|' character.
                                       // Ensure the next character is also a '|' (since the whole token should be "||").
                if matches!(self.buffer.peek(), Some('|')) {
                    Some(self.return_simple_token(TokenKind::Or, start_location))
                } else {
                    let error = ErrorKind::UnknownSymbol {
                        symbol: "|".to_owned(),
                        suggestion: Some("||".to_owned()),
                    };
                    Some(Err((start_location, error, self.cursor)))
                }
            }
            '#' => {
                self.advance_buffer(); // Consume the '#' character.
                self.skip_inline_whitespace(); // Consume any inline whitespace characters
                let identifier = self.read_identifier(); // Reads and consumes an identifier from the buffer.
                match identifier {
                    "define" => Some(Ok((start_location, TokenKind::DefineKeyword, self.cursor))),
                    "undef" => Some(Ok((start_location, TokenKind::UndefineKeyword, self.cursor))),
                    "if" => Some(Ok((start_location, TokenKind::IfKeyword, self.cursor))),
                    "elif" => Some(Ok((start_location, TokenKind::ElifKeyword, self.cursor))),
                    "else" => Some(Ok((start_location, TokenKind::ElseKeyword, self.cursor))),
                    "endif" => Some(Ok((start_location, TokenKind::EndifKeyword, self.cursor))),
                    "" => Some(Err((start_location, ErrorKind::MissingDirective, self.cursor))),
                    keyword => {
                        let error = ErrorKind::UnknownDirective {
                            keyword: keyword.to_owned(),
                        };
                        Some(Err((start_location, error, self.cursor)))
                    }
                }
            }
            '/' => {
                self.advance_buffer(); // Consume the '/' character.

                match self.buffer.peek() {
                    Some('/') => {
                        // Consume the rest of the line, ending at either `\n` or `EOF`.
                        self.advance_to_end_of_line();
                        None
                    }
                    _ => {
                        let error = ErrorKind::UnknownSymbol {
                            symbol: "/".to_owned(),
                            suggestion: Some("//".to_owned()),
                        };
                        Some(Err((start_location, error, self.cursor)))
                    }
                }
            }
            ch if ch.is_alphabetic() || ch == '_' => {
                let identifier = self.read_identifier();
                Some(Ok((start_location, TokenKind::Identifier(identifier), self.cursor)))
            }
            ch if !ch.is_whitespace() => {
                self.advance_buffer(); // Consume the unknown character.
                let error = ErrorKind::UnknownSymbol {
                    symbol: c.to_string(),
                    suggestion: None,
                };
                Some(Err((start_location, error, self.cursor)))
            }
            '\n' => {
                // End of line also means the end of a preprocessor directive.
                self.mode = LexerMode::Unknown;
                Some(Ok((start_location, TokenKind::DirectiveEnd, start_location)))
            }
            _ => panic!("'lex_next_preprocessor_token' encountered whitespace that should of been skipped"),
        }
    }
}

impl<'input> Iterator for Lexer<'input> {
    type Item = LexerResult<'input>;

    /// Attempts to lex and return the next token in this lexer's token stream.
    /// Returns `None` to indicate end-of-stream, `Some(Ok(x))` to indicate success (where `x` is the next token),
    /// and `Some(Err(y))` to indicate that an error occurred during lexing.
    fn next(&mut self) -> Option<Self::Item> {
        // The starting location of a token.
        let mut start_location = None;
        // The starting buffer position of a token.
        let mut start_position = None;

        self.skip_inline_whitespace();

        while let Some(c) = self.buffer.peek().cloned() {
            if self.mode == LexerMode::PreprocessorDirective {
                if let Some(token) = self.lex_next_preprocessor_token(c) {
                    return Some(token);
                };
            } else if c == '\n' {
                self.advance_buffer();
            } else if c == '#' {
                // The first non-whitespace character on this line is '#'. This line must be a directive.

                // If the lexer's mode is currently `SourceBlock`, this is the end of that source block.
                // We create and return a `SourceBlock` as the next token; otherwise, we `continue`.
                // Either way, we skip the rest of the loop to ensure we don't consume the '#', so it's
                // preserved for preprocessor directive lexing.
                let next_token = match self.mode {
                    LexerMode::SourceBlock => Some(Ok(self.create_source_block_token(
                        start_location.take().unwrap(),
                        start_position.take().unwrap(),
                        self.position,
                    ))),
                    _ => self.lex_next_preprocessor_token('#'),
                };

                self.mode = LexerMode::PreprocessorDirective;
                return next_token;
            } else {
                // The first non-whitespace character on this line isn't '#'. This line must be source code.

                // If the lexer's mode is currently `Unknown`, this is the start of a new source block.
                // We switch lexing modes to `SourceBlock` and store information about the start of the block.
                if self.mode == LexerMode::Unknown {
                    self.mode = LexerMode::SourceBlock;
                    // Store the starting position (in buffer) and location (row, col) of the source block.
                    debug_assert!(start_location.is_none());
                    debug_assert!(start_position.is_none());
                    start_location = Some(self.cursor);
                    start_position = Some(self.position);
                }

                // We know that this line is purely source code, so we skip the rest of the line.
                self.advance_to_end_of_line();
            }

            self.skip_inline_whitespace();
        }
        // We've reached the end of the input.

        match self.mode {
            // If the lexer was in the middle of lexing a source block, return the source block as the final token.
            LexerMode::SourceBlock => {
                self.mode = LexerMode::Unknown;
                Some(Ok(self.create_source_block_token(
                    start_location.take().unwrap(),
                    start_position.take().unwrap(),
                    self.input.len(),
                )))
            }
            // If the lexer was in the middle of lexing a preprocessor directive, return a `DirectiveEnd` token.
            LexerMode::PreprocessorDirective => {
                self.mode = LexerMode::Unknown;
                Some(Ok((self.cursor, TokenKind::DirectiveEnd, self.cursor)))
            }
            LexerMode::Unknown => {
                debug_assert!(start_location.is_none());
                debug_assert!(start_position.is_none());
                None
            }
        }
    }
}

// Allows string slices to be converted into `Lexer`s.
impl<'input> From<&'input str> for Lexer<'input> {
    fn from(s: &'input str) -> Self {
        Lexer::new(s)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum LexerMode {
    /// The lexer doesn't have enough context to know what mode it should be in. This is the initial mode of a newly
    /// created lexer, and the mode lexers switch to after reaching the end of a preprocessor directive.
    ///
    /// No lexing is performed in this state. The lexer simply checks the first non-whitespace character of the next
    /// line to determine which mode to switch into, before consuming input. If the character is '#' it switches to
    /// [`PreprocessorDirective`](LexerMode::PreprocessorDirective) mode, otherwise it switches to
    /// [`SourceBlock`](LexerMode::SourceBlock) mode.
    Unknown,

    /// Indicates that the lexer is currently lexing a block of source code.
    /// While in this mode, the lexer treats everything as string literals and performs no tokenization of the input.
    ///
    /// This mode ends when the lexer sees a line where the first non-whitespace character is a '#', at which point it
    /// switches into [`PreprocessorDirective`](LexerMode::PreprocessorDirective) mode.
    SourceBlock,

    /// Indicates that the lexer is currently lexing a preprocessor directive.
    /// While in this mode, the lexer tokenizes input as preprocessor keywords and expressions.
    ///
    /// This mode ends when the lexer hits end-of-line, at which point it switches into
    /// [`Unknown`](LexerMode::Unknown) mode.
    PreprocessorDirective,
}
