// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::grammar::*;
use crate::slice_file::Location;
use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser as PestParser;

type PestPair<'i> = Pair<'i, Rule>;

#[derive(Debug, PestParser)]
#[grammar = "parser/comments.pest"]
pub struct CommentParser;

impl CommentParser {
    pub fn parse_doc_comment(raw_comment: &str, location: Location) -> DocComment {
        // Create an empty comment that the parser will populate as it traverses the parse tree.
        let mut comment = DocComment {
            overview: String::new(),
            see_also: Vec::new(),
            params: Vec::new(),
            returns: None,
            throws: Vec::new(),
            deprecate_reason: None,
            location,
        };

        // Attempt to parse the raw string as a comment (via the `main` rule in `comment.pest`).
        match CommentParser::parse(Rule::main, raw_comment) {
            Ok(mut result) => {
                // Unwrap the parse tree from the `main` rule that the comment was matched against.
                let parse_tree = result.next().unwrap();
                Self::traverse_parse_tree(&mut comment, parse_tree);
            }
            Err(err) => {
                println!("{:?}", err);
                // TODO handle the error better.
            }
        }

        // Sanitize the comment before returning it.
        comment.sanitize();

        // Return the comment. If parsing succeeded, it will be populated with content. If there was
        // a syntax error, the comment will be empty.
        comment
    }

    fn traverse_parse_tree(comment: &mut DocComment, parse_tree: PestPair) {
        // Comments are parsed line by line, but field descriptions can span multiple lines.
        // This string references the most recent field description, so that following lines can
        // append text to it. At the start it references the comment's message before any fields.
        let mut current_string = &mut comment.overview;

        for token in parse_tree.into_inner() {
            match token.as_rule() {
                Rule::message => {
                    // Remove any trailing padding from the message, and append it to the current
                    // field being written into the comment.
                    current_string.push_str(
                        token
                            .as_str()
                            .trim_end_matches(Self::is_padding)
                            .trim_start_matches(char::is_whitespace),
                    );
                    current_string.push('\n');
                }
                Rule::deprecated_field => {
                    // Iterate through the subtokens. Any of them can be missing, but they will
                    // always be in the following order when present: space,
                    // message.
                    let mut has_space = false;
                    let mut has_message = false;
                    for subtoken in token.into_inner().collect::<Vec<PestPair>>() {
                        match subtoken.as_rule() {
                            Rule::space => {
                                has_space = true;
                            }
                            Rule::message => {
                                has_message = true;
                                // Issue an error if there's no whitespace between the identifier
                                // and tag.
                                if !has_space {
                                    // TODO issue an error about missing a space.
                                }

                                // Issue an error if a return field was already specified in the
                                // comment.
                                if comment.deprecate_reason.is_some() {
                                    // TODO issue an error.
                                }

                                let deprecated_message = subtoken
                                    .as_str()
                                    .trim_end_matches(Self::is_padding)
                                    .trim_start_matches(char::is_whitespace);
                                comment.deprecate_reason = Some(deprecated_message.to_owned());
                                // Re-point the current string reference to point to the return
                                // message string so that any
                                // following text (even following lines) is appended to it.
                                current_string = comment.deprecate_reason.as_mut().unwrap();
                                current_string.push('\n');
                            }
                            _ => panic!("matched impossible token: {:?}", subtoken),
                        }
                    }

                    // Issue an error if the parameter field didn't have an identifier.
                    if !has_message {
                        // TODO issue an error about not having an identifier.
                    }
                }
                Rule::param_field => {
                    // Iterate through the subtokens. Any of them can be missing, but they will
                    // always be in the following order when present: space,
                    // identifier, message.
                    let mut has_space = false;
                    let mut has_identifier = false;
                    for subtoken in token.into_inner().collect::<Vec<PestPair>>() {
                        match subtoken.as_rule() {
                            Rule::space => {
                                has_space = true;
                            }
                            Rule::identifier => {
                                has_identifier = true;
                                // Issue an error if there's no whitespace between the identifier
                                // and tag.
                                if !has_space {
                                    // TODO issue an error about missing a space.
                                }

                                // Add a new parameter field to the comment, with an empty
                                // description.
                                let identifier = subtoken.as_str().trim_end_matches(Self::is_padding);
                                comment.params.push((identifier.to_owned(), String::new()));
                                // Re-point the current string reference to point to the parameter's
                                // description string, so that any
                                // following text (even following lines) is appended to it.
                                current_string = &mut comment.params.last_mut().unwrap().1;
                            }
                            Rule::message => {
                                // The grammar rules make it impossible for a message to not follow
                                // identifiers.
                                debug_assert!(has_identifier);
                                // Add the message onto the parameter's description.
                                *current_string += subtoken
                                    .as_str()
                                    .trim_end_matches(Self::is_padding)
                                    .trim_start_matches(char::is_whitespace);
                                current_string.push('\n');
                            }
                            _ => panic!("matched impossible token: {:?}", subtoken),
                        }
                    }

                    // Issue an error if the parameter field didn't have an identifier.
                    if !has_identifier {
                        // TODO issue an error about not having an identifier.
                    }
                }
                Rule::return_field => {
                    // Iterate through the subtokens. Any of them can be missing, but they will
                    // always be in the following order when present: space,
                    // message.
                    let mut has_space = false;
                    let mut has_message = false;
                    for subtoken in token.into_inner().collect::<Vec<PestPair>>() {
                        match subtoken.as_rule() {
                            Rule::space => {
                                has_space = true;
                            }
                            Rule::message => {
                                has_message = true;
                                // Issue an error if there's no whitespace between the identifier
                                // and tag.
                                if !has_space {
                                    // TODO issue an error about missing a space.
                                }

                                // Issue an error if a return field was already specified in the
                                // comment.
                                if comment.returns.is_some() {
                                    // TODO issue an error.
                                }

                                let return_message = subtoken
                                    .as_str()
                                    .trim_end_matches(Self::is_padding)
                                    .trim_start_matches(char::is_whitespace);
                                comment.returns = Some(return_message.to_owned());
                                // Re-point the current string reference to point to the return
                                // message string so that any
                                // following text (even following lines) is appended to it.
                                current_string = comment.returns.as_mut().unwrap();
                                current_string.push('\n');
                            }
                            _ => panic!("matched impossible token: {:?}", subtoken),
                        }
                    }

                    // Issue an error if the parameter field didn't have an identifier.
                    if !has_message {
                        // TODO issue an error about not having an identifier.
                    }
                }
                Rule::see_field => {
                    // Iterate through the subtokens. Any of them can be missing, but they will
                    // always be in the following order when present: space,
                    // identifier, message.
                    let mut has_space = false;
                    let mut has_identifier = false;
                    for subtoken in token.into_inner().collect::<Vec<PestPair>>() {
                        match subtoken.as_rule() {
                            Rule::space => {
                                has_space = true;
                            }
                            Rule::identifier => {
                                has_identifier = true;
                                // Issue an error if there's no whitespace between the identifier
                                // and tag.
                                if !has_space {
                                    // TODO issue an error about missing a space.
                                }

                                // Add a new see field to the comment, with an empty description.
                                let identifier = subtoken.as_str().trim_end_matches(Self::is_padding);
                                comment.see_also.push(identifier.to_owned());
                                // Re-point the current string reference to point to the identifier
                                // string. References shouldn't have
                                // additional text, but there's no where else logical
                                // to append the text to.
                                current_string = comment.see_also.last_mut().unwrap();
                            }
                            Rule::message => {
                                // The grammar rules make it impossible for a message to not follow
                                // identifiers.
                                debug_assert!(has_identifier);
                                // Issue an error; references shouldn't have additional
                                // descriptions. TODO issue the
                                // error.
                            }
                            _ => panic!("matched impossible token: {:?}", subtoken),
                        }
                    }

                    // Issue an error if the parameter didn't have an identifier field.
                    if !has_identifier {
                        // TODO issue an error about not having an identifier.
                    }
                }
                Rule::throws_field => {
                    // Iterate through the subtokens. Any of them can be missing, but they will
                    // always be in the following order when present: space,
                    // identifier, message.
                    let mut has_space = false;
                    let mut has_identifier = false;
                    for subtoken in token.into_inner().collect::<Vec<PestPair>>() {
                        match subtoken.as_rule() {
                            Rule::space => {
                                has_space = true;
                            }
                            Rule::identifier => {
                                has_identifier = true;
                                // Issue an error if there's no whitespace between the identifier
                                // and tag.
                                if !has_space {
                                    // TODO issue an error about missing a space.
                                }

                                // Add a new throws field to the comment, with an empty description.
                                let identifier = subtoken.as_str().trim_end_matches(Self::is_padding);
                                comment.throws.push((identifier.to_owned(), String::new()));
                                // Re-point the current string reference to point to the throws'
                                // description string, so that any
                                // following text (even following lines) is appended to it.
                                current_string = &mut comment.throws.last_mut().unwrap().1;
                            }
                            Rule::message => {
                                // The grammar rules make it impossible for a message to not follow
                                // identifiers.
                                debug_assert!(has_identifier);
                                // Add the message onto the throws' description.
                                *current_string += subtoken
                                    .as_str()
                                    .trim_end_matches(Self::is_padding)
                                    .trim_start_matches(char::is_whitespace);
                                current_string.push('\n');
                            }
                            _ => panic!("matched impossible token: {:?}", subtoken),
                        }
                    }

                    // Issue an error if the throws field didn't have an identifier.
                    if !has_identifier {
                        // TODO issue an error about not having an identifier.
                    }
                }
                Rule::invalid_field => {
                    // Iterate through the subtokens.
                    let mut has_tag = false;
                    for subtoken in token.into_inner().collect::<Vec<PestPair>>() {
                        if let Rule::identifier = subtoken.as_rule() {
                            has_tag = true;
                            // Issue an error for the unknown field tag.
                            // TODO issue the error.
                        }
                    }

                    // Issue an error if the field didn't have a tag name.
                    if !has_tag {
                        // TODO issue an error.
                    }
                }
                Rule::EOI => {}
                _ => panic!("matched impossible token: {:?}", token),
            }
        }
    }

    // Utility function used for trimming leading and trailing padding from comments.
    fn is_padding(c: char) -> bool {
        c == '*' || c.is_whitespace()
    }
}
