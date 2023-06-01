// Copyright (c) ZeroC, Inc.

use super::tokens::*;
use crate::slice_file::{Location, Span};

use std::iter::Peekable;
use std::str::Chars;
use std::vec::IntoIter;

type LexerResult<'a> = Result<Token<'a>, Error<'a>>;

/// Converts the lines of a doc comment into a stream of semantic tokens.
///
/// This token stream is in turn consumed by the [comment parser](super::parser::CommentParser) which parses the tokens
/// into a [`DocComment`](crate::grammar::DocComment).
#[derive(Debug)]
pub struct Lexer<'input> {
    /// Iterator over the lines of the doc comment this lexer is operating on.
    lines: IntoIter<(&'input str, Span)>,

    /// The line that is currently being lexed (the lexer works one line at a time).
    current_line: &'input str,

    /// Iterator over the characters in the current line.
    /// This is what the lexer actually operates on, by peeking at and consuming codepoints from this buffer.
    buffer: Peekable<Chars<'input>>,

    /// The lexer's position in the current line's buffer.
    position: usize,

    /// The lexer's current [location](crate::slice_file::Location) in the input.
    /// Used to tag tokens with their starting and ending locations.
    cursor: Location,

    /// The current mode of the lexer; controls how the input is tokenized in a context-dependent manner.
    mode: LexerMode,
}

impl<'input> Lexer<'input> {
    /// Creates a new lexer over the provided lines.
    pub fn new(lines: Vec<(&'input str, Span)>) -> Self {
        let mut lines = lines.into_iter();
        let (first_line, first_span) = lines.next().expect("created lexer over an empty comment");

        // Create a lexer. The values don't matter, because they're all set by `switch_to_next_line`.
        let mut lexer = Lexer {
            lines,
            current_line: "",
            buffer: "".chars().peekable(),
            position: 0,
            cursor: Location::default(),
            mode: LexerMode::Message,
        };
        lexer.switch_to_next_line(first_line, first_span); // Actually initialize the lexer.
        lexer
    }

    /// The lexer operates on doc comments one line at a time; this function tells the lexer to discard the line it's
    /// currently lexing and switch to the provided line (and span). It updates all the lexer's fields accordingly.
    fn switch_to_next_line(&mut self, line: &'input str, span: Span) {
        self.current_line = line;
        self.buffer = self.current_line.chars().peekable();
        self.position = 0;
        self.cursor = span.start;

        // If the first non-whitespace character on this line is '@', then this line starts a new tag, and we put the
        // lexer in `BlockTag` mode accordingly. Otherwise, we put the lexer in its 'default' `Message` mode instead.
        if self.current_line.trim_start().starts_with('@') {
            self.mode = LexerMode::BlockTag;
        } else {
            self.mode = LexerMode::Message;
        }
    }

    /// Consumes the next character in the buffer and moves the lexer's cursor forward accordingly.
    fn advance_buffer(&mut self) {
        if self.buffer.next().is_some() {
            self.position += 1;
            self.cursor.col += 1;
        }
    }

    /// Skips over whitespace characters in the buffer until a non-whitespace character is reached.
    /// After calling this function, the next character will be non-whitespace or `None` (end of buffer).
    fn skip_whitespace(&mut self) {
        // Loop while the next character in the buffer is whitespace (except '\n').
        while matches!(self.buffer.peek(), Some(c) if c.is_whitespace()) {
            self.advance_buffer(); // Consume the character.
        }
    }

    /// Reads, consumes, and returns a string of alphanumeric characters from the buffer.
    /// After calling this function, the next char will be a non-alphanumeric character or `None` (end-of-buffer).
    fn read_identifier(&mut self) -> &'input str {
        let start_position = self.position;

        // Loop while the next character in the buffer is an alphanumeric or underscore.
        while matches!(self.buffer.peek(), Some(c) if (c.is_ascii_alphanumeric() || *c == '_')) {
            self.advance_buffer(); // Consume the character.
        }

        &self.current_line[start_position..self.position]
    }

    /// Attempts to read and validate a tag keyword from the buffer.
    /// Tag keywords always start with a '@' character that is followed by an identifier.
    /// If a valid tag keyword is found, this function returns `Some(Ok(<keyword_token>)))`, otherwise it returns
    /// `Some(Err(...))`.
    ///
    /// This function also ensures the tag is used in the correct context. For instance, `@link` is only valid as an
    /// inline tag. If found while the lexer is in `BlockTag` mode, this returns a `IncorrectContextForTag` error.
    fn read_tag_keyword(&mut self) -> LexerResult<'input> {
        let start_location = self.cursor;

        // Consume the '@' character then read the following keyword.
        debug_assert!(matches!(self.buffer.peek(), Some('@')));
        self.advance_buffer();
        let ident = self.read_identifier();

        // Return the token (or error) corresponding to the keyword.
        let token = match ident {
            "param" => Ok((start_location, TokenKind::ParamKeyword, self.cursor)),
            "returns" => Ok((start_location, TokenKind::ReturnsKeyword, self.cursor)),
            "throws" => Ok((start_location, TokenKind::ThrowsKeyword, self.cursor)),
            "see" => Ok((start_location, TokenKind::SeeKeyword, self.cursor)),
            "link" => Ok((start_location, TokenKind::LinkKeyword, self.cursor)),
            "" => Err((start_location, ErrorKind::MissingTag, self.cursor)),
            tag => Err((start_location, ErrorKind::UnknownTag { tag }, self.cursor)),
        };

        // Check if the keyword was valid within the current context (inline vs block).
        let is_inline = self.mode == LexerMode::InlineTag;
        if let Ok((start, token_kind, end)) = &token {
            let is_valid = match token_kind {
                // These tags are never valid inline.
                TokenKind::ParamKeyword
                | TokenKind::ReturnsKeyword
                | TokenKind::ThrowsKeyword
                | TokenKind::SeeKeyword => !is_inline,

                // These tags are only valid inline.
                TokenKind::LinkKeyword => is_inline,

                _ => unreachable!("Encountered non-keyword token in 'lex_tag_keyword'!"),
            };
            if !is_valid {
                let error = ErrorKind::IncorrectContextForTag { tag: ident, is_inline };
                return Err((*start, error, *end));
            }
        }

        // If all the checks were fine, we return the token here.
        token
    }

    /// Reads and returns a token from the buffer while the lexer is in `Message` mode.
    /// If the first character in the buffer is a '{', this function checks if it's the start of an inline tag.
    /// If it is, this returns a '{' token and switches the lexer to `InlineTag` mode.
    /// Otherwise, this reads raw text from the buffer and returns a `Text` token. No errors are possible in this
    /// function, and since it's only called when the buffer is non-empty, it always returns something.
    fn lex_message(&mut self) -> Token<'input> {
        let start_location = self.cursor;
        let start_position = self.position;

        // Check for the start of an inline tag. This is a '{' token followed by a '@' token (possibly separated by
        // whitespace). If both are present, we switch to `InlineTag` mode and return the '{' we consumed.
        // Otherwise, we fall through into the rest of the function which returns a normal `Text` token.
        if matches!(self.buffer.peek(), Some('{')) {
            self.advance_buffer(); // Consume the '{' character.
            self.skip_whitespace(); // Skip any whitespace.

            if matches!(self.buffer.peek(), Some('@')) {
                self.mode = LexerMode::InlineTag;
                return (start_location, TokenKind::LeftBrace, self.cursor);
            }
        }

        // Loop while the next character in the buffer is not '{'.
        while matches!(self.buffer.peek(), Some(c) if *c != '{') {
            self.advance_buffer(); // Consume the character.
        }

        // Return the text.
        let text = &self.current_line[start_position..self.position];
        (start_location, TokenKind::Text(text), self.cursor)
    }

    /// Attempts to read and return a token from the buffer while the lexer is in `BlockTag` or `InlineTag` mode.
    /// Returns `None` if there's only whitespace left in the buffer (which is ignored while in these modes).
    /// Returns `Some(Ok(x))` to indicate success (where `x` is the next token),
    /// and `Some(Err(y))` to indicate an error occurred during lexing.
    fn lex_tag_component(&mut self) -> Option<LexerResult<'input>> {
        self.skip_whitespace();

        // Check the next character in the buffer if it isn't empty. If it is empty, the `map` will return `None`.
        self.buffer.peek().cloned().map(|c| match c {
            // If the next character is a '@' it must be the start of a tag keyword.
            '@' => self.read_tag_keyword(),

            // If the next character is a ':' it can either be a scope separator "::" or the end of block tag ":".
            ':' => {
                let start_location = self.cursor;
                self.advance_buffer(); // Consume the ':' character.

                // Check if the next character is also ':'. If so, this is a scope separator, otherwise it's just ':'.
                if matches!(self.buffer.peek(), Some(':')) {
                    self.advance_buffer(); // Consume the 2nd ':' character.
                    Ok((start_location, TokenKind::DoubleColon, self.cursor))
                } else {
                    // If we were lexing a block tag, this marks the end of the tag; switch back to `Message` mode.
                    if self.mode == LexerMode::BlockTag {
                        self.mode = LexerMode::Message;
                    }
                    Ok((start_location, TokenKind::Colon, self.cursor))
                }
            }

            // If the next character is a '}' it should be the end of an inline tag.
            '}' => {
                // If we were lexing an inline tag, this marks the end of the tag; switch back to `Message` mode.
                if self.mode == LexerMode::InlineTag {
                    self.mode = LexerMode::Message;
                }
                let start_location = self.cursor;
                self.advance_buffer(); // Consume the '}' character.
                Ok((start_location, TokenKind::RightBrace, self.cursor))
            }

            // If the next character is an alphanumeric or underscore, it's the start of an identifier.
            c if c.is_ascii_alphanumeric() || c == '_' => {
                let start_location = self.cursor;
                let identifier = self.read_identifier();
                Ok((start_location, TokenKind::Identifier(identifier), self.cursor))
            }

            // If none of the above cases matched, the next character is an unknown symbol and we return an error.
            c => {
                let start_location = self.cursor;
                self.advance_buffer(); // Consume the unknown symbol.
                Err((start_location, ErrorKind::UnknownSymbol { symbol: c }, self.cursor))
            }
        })
    }
}

impl<'input> Iterator for Lexer<'input> {
    type Item = LexerResult<'input>;

    /// Attempts to lex and return the next token in this lexer's token stream.
    /// Returns `None` to indicate end-of-stream, `Some(Ok(x))` to indicate success (where `x` is the next token),
    /// and `Some(Err(y))` to indicate that an error occurred during lexing.
    fn next(&mut self) -> Option<Self::Item> {
        // While the buffer isn't empty, attempt to lex a token from it.
        // This loop exits when we return a token, error, or reach the end of the comment.
        while self.buffer.peek().is_some() {
            let item = match self.mode {
                LexerMode::BlockTag | LexerMode::InlineTag => self.lex_tag_component(),
                LexerMode::Message => Some(Ok(self.lex_message())),
                _ => unreachable!("comment lexer finished with a non-empty buffer!"),
            };
            // If the lexer lexed a token or encountered an error, return it.
            if let Some(result) = item {
                return Some(result);
            }
        }

        // If we get to this match, we've hit the end of the current line.
        match self.mode {
            // If the lexer is in `InlineTag` mode when it hit EOL, this means there was no closing '}'.
            // So, we return an `UnterminatedInlineTag` error since inline tags can't span multiple lines.
            LexerMode::InlineTag => {
                self.mode = LexerMode::Message; // Change the mode so the error is only reported once.
                Some(Err((self.cursor, ErrorKind::UnterminatedInlineTag, self.cursor)))
            }

            // If the lexer is in `Message` or `BlockTag` mode when it hit EOL, this is normal and expected.
            // We check if there's another line to the comment. If so, we start lexing that line; otherwise we switch
            // the lexer to `Finished` mode, since there's no more input left. Either way we return a `Newline` token.
            LexerMode::BlockTag | LexerMode::Message => {
                let newline_token = (self.cursor, TokenKind::Newline, self.cursor);
                if let Some((next_line, next_span)) = self.lines.next() {
                    self.switch_to_next_line(next_line, next_span);
                } else {
                    self.mode = LexerMode::Finished;
                }
                Some(Ok(newline_token))
            }

            // If the lexer has hit the end of the comment, return `None` to signal this.
            LexerMode::Finished => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum LexerMode {
    /// Indicates that the lexer is currently lexing a block tag.
    /// While in this mode the lexer only looks for tag keywords and identifiers.
    ///
    /// The lexer enters this mode when it starts a new line, and the first non-whitespace character on the line
    /// is '@'. When the lexer hits a ':' or the end of a line, it switches into [`Message`](LexerMode::Message) mode.
    BlockTag,

    /// Indicates that the lexer is currently lexing an inline tag. Similar to [`BlockTag`](LexerMode::BlockTag) mode,
    /// while in this mode the lexer only looks for tag keywords and identifier.
    ///
    /// This mode starts when the lexer sees an opening brace, and ends when it hits a closing brace or newline;
    /// in both cases it switches to [`Message`](LexerMode::Message) mode.
    InlineTag,

    /// Indicates that the lexer is currently lexing raw text.
    /// While in this mode the lexer performs no additional analysis of the text and simply forwards it along.
    Message,

    /// Indicates that the lexer has reached the end of the doc comment.
    /// While in this mode, calling `next` is no-op and the lexer just returns `None` for everything.
    Finished,
}
