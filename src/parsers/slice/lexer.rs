// Copyright (c) ZeroC, Inc.

use super::super::common::SourceBlock;
use super::tokens::*;
use crate::slice_file::Location;

use std::iter::Peekable;
use std::str::CharIndices;

type LexerResult<'a> = Result<Token<'a>, Error>;

/// Converts a stream of [source blocks](super::super::common::SourceBlock) (blocks of source code) into a stream of
/// Slice tokens.
///
/// This token stream is in turn consumed by the [Slice parser](super::parser::Parser) which parses the tokens into an
/// [AST](crate::ast::Ast).
#[derive(Debug)]
pub struct Lexer<'input, T>
where
    T: Iterator<Item = SourceBlock<'input>>,
{
    /// Iterator over the source blocks that this lexer is operating on.
    source_blocks: Peekable<T>,

    /// The source block that the lexer is currently lexing within.
    current_block: SourceBlock<'input>,

    /// Iterator over the characters in the current block.
    /// This is what the lexer actually operates on, by peeking at and consuming codepoints from this buffer.
    buffer: Peekable<CharIndices<'input>>,

    /// The lexer's current [location](crate::slice_file::Location) in the slice file.
    /// Used to tag tokens with their starting and ending locations in the source input.
    ///
    /// Since source blocks can be non-adjacent (separated by a preprocessor directive) in a slice file,
    /// it's value can jump forward when switching to a new source block, making it unreliable for indexing.
    cursor: Location,

    /// This flag stores whether the lexer is currently lexing the inside of an attribute.
    /// It is set to true upon encountering an '[' character, and false upon an ']' character.
    attribute_mode: bool,
}

impl<'input, T> Lexer<'input, T>
where
    T: Iterator<Item = SourceBlock<'input>>,
{
    fn new(mut input: T) -> Self {
        // If the input is empty create a dummy source block that's empty and will cause the lexer to immediately exit.
        let current_block = input.next().unwrap_or_else(|| SourceBlock {
            content: "",
            start: Location::default(),
            end: Location::default(),
        });
        let buffer = current_block.content.char_indices().peekable();
        let start_location = current_block.start;

        Lexer {
            source_blocks: input.peekable(),
            current_block,
            buffer,
            cursor: start_location,
            attribute_mode: false,
        }
    }

    /// Returns the lexer's position in the buffer of the source block it's currently lexing.
    fn get_position(&mut self) -> usize {
        if let Some((i, _)) = self.buffer.peek() {
            *i
        } else {
            // `None` means we're at the end of the current source block's buffer.
            self.current_block.content.len()
        }
    }

    /// Consumes the next character in the buffer and moves the lexer's cursor forward accordingly.
    fn advance_buffer(&mut self) {
        // Consume the next character and check if it's a newline.
        if let Some((_, c)) = self.buffer.next() {
            if c == '\n' {
                self.cursor.row += 1;
                self.cursor.col = 1;
            } else {
                self.cursor.col += 1;
            }
        }
    }

    /// Consumes characters in the buffer until end-of-line (doesn't consume the EOL) or end-of-buffer is reached.
    fn advance_to_end_of_line(&mut self) {
        // Loop until the next character is '\n'.
        while matches!(self.buffer.peek(), Some((_, c)) if *c != '\n') {
            self.advance_buffer(); // Consume the character.
        }
    }

    /// Consumes whitespace characters in the buffer until a non-whitespace character is reached.
    /// After calling this function, the next character will be non-whitespace or `None` (end of buffer).
    fn skip_whitespace(&mut self) {
        // Loop while the next character in the buffer is whitespace.
        while matches!(self.buffer.peek(), Some((_, c)) if c.is_whitespace()) {
            self.advance_buffer(); // Consume the character.
        }
    }

    /// Reads, consumes, and returns a string of alphanumeric characters from the buffer.
    /// After calling this function, the next character will be a non-alphanumeric character or `None` (end of buffer).
    fn read_alphanumeric(&mut self) -> &'input str {
        let start_position = self.get_position();

        // Loop while the next character in the buffer is alphanumeric or an underscore.
        while matches!(self.buffer.peek(), Some((_, c)) if (c.is_ascii_alphanumeric() || *c == '_')) {
            self.advance_buffer(); // Consume the alphanumeric character.
        }

        let end_position = self.get_position();
        &self.current_block.content[start_position..end_position]
    }

    /// Reads, consumes, and returns a string literal from the buffer.
    /// String literals are any characters contained within a pair of un-escaped double-quotes.
    /// The returned string doesn't include the opening and closing quotation marks, just the content between them.
    ///
    /// This function expects the lexer's cursor to be immediately before the opening '"' character.
    fn read_string_literal(&mut self) -> Result<&'input str, ErrorKind> {
        self.advance_buffer(); // Consume the opening quotation mark.

        let start_position = self.get_position();
        let mut is_next_char_escaped = false;
        while let Some((_, c)) = self.buffer.peek() {
            if *c == '\n' {
                // String literals cannot contain newlines.
                return Err(ErrorKind::UnterminatedStringLiteral);
            } else if is_next_char_escaped {
                // If this character is escaped, don't check it and just reset the flag.
                is_next_char_escaped = false;
            } else {
                match c {
                    '"' => {
                        // We've reached the end of the string literal.
                        let end_position = self.get_position();
                        self.advance_buffer(); // Consume the closing quotation mark.
                        return Ok(&self.current_block.content[start_position..end_position]);
                    }
                    '\\' => is_next_char_escaped = true,
                    _ => {}
                }
            }
            self.advance_buffer(); // Consume the character.
        }

        // Reaching this means we hit the end of a buffer before the end of the string literal.
        Err(ErrorKind::UnterminatedStringLiteral)
    }

    /// Reads, consumes. and returns a line comment from the buffer.
    /// This function expects the lexer's cursor to be immediately after the last '/' character.
    fn read_line_comment(&mut self) -> &'input str {
        let start_position = self.get_position();
        self.advance_to_end_of_line();
        let end_position = self.get_position();

        &self.current_block.content[start_position..end_position]
    }

    /// Reads and consumes a block comment from the buffer, ignoring it.
    /// This function expects the lexer's cursor to be immediately after the opening "/*".
    fn consume_block_comment(&mut self) -> Result<(), ErrorKind> {
        let mut last_character_was_an_asterisk = false;

        while let Some((_, c)) = self.buffer.peek().cloned() {
            self.advance_buffer(); // Consume the character.
            match c {
                '/' if last_character_was_an_asterisk => return Ok(()),
                '*' => last_character_was_an_asterisk = true,
                _ => last_character_was_an_asterisk = false,
            }
        }

        // Reaching this means we hit the end of a buffer before the end of the block comment.
        Err(ErrorKind::UnterminatedBlockComment)
    }

    /// Checks if an identifier corresponds to a Slice keyword. If it does,
    /// return the keyword's token. Otherwise, return an `[TokenKind::Identifier]` token.
    fn check_if_keyword(identifier: &str) -> TokenKind<'_> {
        debug_assert!(identifier.chars().all(|c| c.is_ascii_alphanumeric() || c == '_'));
        debug_assert!(!identifier.is_empty());

        match identifier {
            "module" => TokenKind::ModuleKeyword,
            "struct" => TokenKind::StructKeyword,
            "exception" => TokenKind::ExceptionKeyword,
            "class" => TokenKind::ClassKeyword,
            "interface" => TokenKind::InterfaceKeyword,
            "enum" => TokenKind::EnumKeyword,
            "custom" => TokenKind::CustomKeyword,
            "typealias" => TokenKind::TypeAliasKeyword,
            "Result" => TokenKind::ResultKeyword,
            "Sequence" => TokenKind::SequenceKeyword,
            "Dictionary" => TokenKind::DictionaryKeyword,
            "bool" => TokenKind::BoolKeyword,
            "int8" => TokenKind::Int8Keyword,
            "uint8" => TokenKind::UInt8Keyword,
            "int16" => TokenKind::Int16Keyword,
            "uint16" => TokenKind::UInt16Keyword,
            "int32" => TokenKind::Int32Keyword,
            "uint32" => TokenKind::UInt32Keyword,
            "varint32" => TokenKind::VarInt32Keyword,
            "varuint32" => TokenKind::VarUInt32Keyword,
            "int64" => TokenKind::Int64Keyword,
            "uint64" => TokenKind::UInt64Keyword,
            "varint62" => TokenKind::VarInt62Keyword,
            "varuint62" => TokenKind::VarUInt62Keyword,
            "float32" => TokenKind::Float32Keyword,
            "float64" => TokenKind::Float64Keyword,
            "string" => TokenKind::StringKeyword,
            "AnyClass" => TokenKind::AnyClassKeyword,
            "compact" => TokenKind::CompactKeyword,
            "idempotent" => TokenKind::IdempotentKeyword,
            "mode" => TokenKind::ModeKeyword,
            "stream" => TokenKind::StreamKeyword,
            "tag" => TokenKind::TagKeyword,
            "throws" => TokenKind::ThrowsKeyword,
            "unchecked" => TokenKind::UncheckedKeyword,
            ident => TokenKind::Identifier(ident),
        }
    }

    /// Consumes a single character from the lexer's buffer and returns a token of the specified kind.
    /// This is a convenience function for the common case where a token's lexeme is a single character.
    fn return_simple_token(&mut self, token: TokenKind<'input>, start: Location) -> Option<LexerResult<'input>> {
        self.advance_buffer(); // Consume the token from the buffer.
        Some(Ok((start, token, self.cursor))) // Return it.
    }

    /// Attempts to read and return a Slice token from the buffer.
    /// Returns `None` to indicate it read a token but ignored it (non-doc comments, whitespace, etc.),
    /// `Some(Ok(x))` to indicate success (where `x` is the next token),
    /// and `Some(Err(y))` to indicate an error occurred during lexing.
    fn lex_next_slice_token(&mut self, c: char) -> Option<LexerResult<'input>> {
        let start_location = self.cursor;
        match c {
            '(' => self.return_simple_token(TokenKind::LeftParenthesis, start_location),
            ')' => self.return_simple_token(TokenKind::RightParenthesis, start_location),
            '[' => {
                // Set the 'attribute_mode' flag since this must be the start of an attribute.
                self.attribute_mode = true;
                self.advance_buffer(); // Consume the '[' character.

                // Check if the next character is also '['.
                if matches!(self.buffer.peek(), Some((_, '['))) {
                    self.advance_buffer(); // Consume the second '[' character.
                    Some(Ok((start_location, TokenKind::DoubleLeftBracket, self.cursor)))
                } else {
                    Some(Ok((start_location, TokenKind::LeftBracket, self.cursor)))
                }
            }
            ']' => {
                // Clear the 'attribute_mode' flag since this must be the end of an attribute.
                self.attribute_mode = false;
                self.advance_buffer(); // Consume the ']' character.

                // Check if the next character is also ']'.
                if matches!(self.buffer.peek(), Some((_, ']'))) {
                    self.advance_buffer(); // Consume the second ']' character.
                    Some(Ok((start_location, TokenKind::DoubleRightBracket, self.cursor)))
                } else {
                    Some(Ok((start_location, TokenKind::RightBracket, self.cursor)))
                }
            }
            '{' => self.return_simple_token(TokenKind::LeftBrace, start_location),
            '}' => self.return_simple_token(TokenKind::RightBrace, start_location),
            '<' => self.return_simple_token(TokenKind::LeftChevron, start_location),
            '>' => self.return_simple_token(TokenKind::RightChevron, start_location),
            ',' => self.return_simple_token(TokenKind::Comma, start_location),
            ':' => {
                self.advance_buffer(); // Consume the ':' character.
                                       // Check if the next character is also ':'.
                if matches!(self.buffer.peek(), Some((_, ':'))) {
                    self.advance_buffer(); // Consume the second ':' character.
                    Some(Ok((start_location, TokenKind::DoubleColon, self.cursor)))
                } else {
                    Some(Ok((start_location, TokenKind::Colon, self.cursor)))
                }
            }
            '=' => self.return_simple_token(TokenKind::Equals, start_location),
            '?' => self.return_simple_token(TokenKind::QuestionMark, start_location),
            '-' => {
                self.advance_buffer(); // Consume the '-' character.
                                       // Check if the next character is '>'.
                if matches!(self.buffer.peek(), Some((_, '>'))) {
                    self.advance_buffer(); // Consume the second '>' character.
                    Some(Ok((start_location, TokenKind::Arrow, self.cursor)))
                } else {
                    Some(Ok((start_location, TokenKind::Minus, self.cursor)))
                }
            }
            '"' => {
                let result = self.read_string_literal();
                Some(match result {
                    Ok(s) => Ok((start_location, TokenKind::StringLiteral(s), self.cursor)),
                    Err(err) => Err((start_location, err, self.cursor)),
                })
            }
            '/' => {
                self.advance_buffer(); // Consume the '/' character.

                match self.buffer.peek() {
                    // The token is at least '//', indicating a line comment.
                    Some((_, '/')) => {
                        self.advance_buffer(); // Consume the 2nd '/' character.

                        // Check if there's a 3rd '/' character indicating this may be a doc comment.
                        let mut is_doc_comment = matches!(self.buffer.peek(), Some((_, '/')));
                        if is_doc_comment {
                            self.advance_buffer(); // Consume the 3rd '/' character.

                            // Check if there's a 4th '/' character, which would turn this back into a non-doc comment.
                            // Doc comments must start with _exactly_ 3 '/' characters.
                            is_doc_comment = !matches!(self.buffer.peek(), Some((_, '/')));
                        }

                        let content_start_loc = self.cursor;
                        let comment = self.read_line_comment();
                        match is_doc_comment {
                            true => Some(Ok((content_start_loc, TokenKind::DocComment(comment), self.cursor))),
                            false => None, // Non-doc comments are ignored.
                        }
                    }

                    // The token is "/*", indicating the start of a block comment.
                    Some((_, '*')) => {
                        self.advance_buffer(); // Consume the '*'.
                        match self.consume_block_comment() {
                            Ok(_) => None, // Block comments are always ignored.
                            Err(err) => Some(Err((start_location, err, self.cursor))),
                        }
                    }

                    // The token is just "/", indicating a syntax error. '/' on its own isn't a valid Slice token.
                    _ => {
                        let error = ErrorKind::UnknownSymbol {
                            symbol: "/".to_owned(),
                            suggestion: Some("//".to_owned()),
                        };
                        Some(Err((start_location, error, self.cursor)))
                    }
                }
            }
            '\\' => {
                self.advance_buffer(); // Consume the '\' character.
                                       // Check if the next character could be the start of an identifier.
                if matches!(self.buffer.peek(), Some((_, ch)) if ch.is_ascii_alphabetic()) {
                    let identifier = self.read_alphanumeric();
                    Some(Ok((start_location, TokenKind::Identifier(identifier), self.cursor)))
                } else {
                    // The token is just "\", indicating a syntax error. '\' on its own isn't a valid Slice token.
                    let error = ErrorKind::UnknownSymbol {
                        symbol: "\\".to_string(),
                        suggestion: Some("\\<identifier>".to_owned()),
                    };
                    Some(Err((start_location, error, self.cursor)))
                }
            }
            _ if c.is_ascii_alphabetic() => {
                let token = if self.attribute_mode {
                    // If we're lexing an attribute, return the identifier as-is, without checking if it's a keyword.
                    TokenKind::Identifier(self.read_alphanumeric())
                } else {
                    Self::check_if_keyword(self.read_alphanumeric())
                };
                Some(Ok((start_location, token, self.cursor)))
            }
            _ if c.is_ascii_digit() => {
                let integer = self.read_alphanumeric();
                Some(Ok((start_location, TokenKind::IntegerLiteral(integer), self.cursor)))
            }
            _ if c.is_whitespace() => {
                self.skip_whitespace();
                None
            }
            unknown => {
                self.advance_buffer(); // Consume the unknown symbol.
                let error = ErrorKind::UnknownSymbol {
                    symbol: unknown.to_string(),
                    suggestion: None,
                };
                Some(Err((start_location, error, self.cursor)))
            }
        }
    }
}

impl<'input, T> Iterator for Lexer<'input, T>
where
    T: Iterator<Item = SourceBlock<'input>>,
{
    type Item = LexerResult<'input>;

    /// Attempts to lex and return the next token in this lexer's token stream.
    /// Returns `None` to indicate end-of-stream, `Some(Ok(x))` to indicate success (where `x` is the next token),
    /// and `Some(Err(y))` to indicate an error occurred during lexing.
    fn next(&mut self) -> Option<Self::Item> {
        // Continue iterating until we return a token, or reach the end of our source blocks.
        loop {
            // Continue iterating until we return a token, or reach the end of the current source block.
            while let Some((_, c)) = self.buffer.peek().cloned() {
                // If the lexer has lexed a token or encountered an error, return it.
                if let Some(token) = self.lex_next_slice_token(c) {
                    return Some(token);
                }
            }

            // We've reached the end of the current source block.
            if let Some(next_source_block) = self.source_blocks.next() {
                // Drop the current source block and replace it with the next source block.
                self.current_block = next_source_block;
                self.buffer = self.current_block.content.char_indices().peekable();
                self.cursor = self.current_block.start;
            } else {
                // There are no more source blocks to parse, the lexer has hit end of input.
                return None;
            }
        }
    }
}

// Allows iterators of source blocks to be converted into `Lexer`s.
impl<'input, T> From<T> for Lexer<'input, T>
where
    T: Iterator<Item = SourceBlock<'input>>,
{
    fn from(source_blocks: T) -> Self {
        Lexer::new(source_blocks)
    }
}

// Allows string slices to be converted into `Lexer`s.
#[cfg(test)]
impl<'input> From<&'input str> for Lexer<'input, std::iter::Once<SourceBlock<'input>>> {
    fn from(s: &'input str) -> Self {
        let newlines = s.char_indices().filter(|&(_, c)| c == '\n').collect::<Vec<_>>();
        let chars_in_last_line = s[newlines.last().unwrap().0..].chars().count();

        let source_block = SourceBlock {
            content: s,
            start: Location::default(),
            end: Location {
                row: newlines.len() + 1,
                col: chars_in_last_line,
            },
        };
        Lexer::new(std::iter::once(source_block))
    }
}
