// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::code_block::CodeBlock;
use crate::cs_util::*;
use slice::ast::Ast;
use slice::grammar::{DocComment, NamedSymbol, Operation};
use slice::writer::Writer;
use std::fmt;

use regex::Regex;

/// Helper method that checks if a named symbol has a comment written on it, and if so, formats
/// it as a C# style doc comment and writes it to the underlying output.
pub fn write_comment(writer: &mut Writer, named_symbol: &dyn NamedSymbol) {
    if let Some(comment) = named_symbol.comment() {
        writer.write(&CsharpComment::new(comment).to_string());
    }
}

struct CommentTag<'a> {
    tag: &'a str,
    content: &'a str,
    attribute_name: &'a str,
    attribute_value: &'a str,
}

impl fmt::Display for CommentTag<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.content.is_empty() {
            // If the comment has no content don't write anything.
            return Ok(());
        }

        let attribute = if self.attribute_name.is_empty() {
            "".to_owned()
        } else {
            format!(r#" {}="{}""#, self.attribute_name, self.attribute_value)
        };

        write!(
            f,
            "/// <{tag}{attribute}>{content}</{tag}>",
            tag = self.tag,
            attribute = attribute,
            content = self
                .content
                .trim_matches(char::is_whitespace)
                .replace("\n", "\n/// ")
        )
    }
}

pub struct CsharpComment(pub DocComment);

impl CsharpComment {
    pub fn new(comment: &DocComment) -> Self {
        // process comment here
        // replace @link @see, etc.
        let mut comment = comment.clone();

        // Replace comments like '<code>my code</code>' by 'my code'
        let re: regex::Regex = Regex::new(r"(?ms)<.+>\s?(?P<content>.+)\s?</.+>").unwrap();
        comment.message = re
            .replace_all(&mut comment.message, "${content}")
            .to_string();

        // Replace comments like '{@link FooBar}' by 'FooBar'
        let re: regex::Regex = Regex::new(r"\{@link\s+(?P<link>\w+)\s?\}").unwrap();
        comment.message = re.replace_all(&mut comment.message, "${link}").to_string();

        // TODO: ${see} should actually be replaced by the real Csharp identifier (see
        // csharpIdentifier in C++)
        let re: regex::Regex = Regex::new(r"\{@see\s+(?P<see>\w+)\s?\}").unwrap();
        comment.message = re
            .replace_all(&mut comment.message, r#"<see cref="${see}"/>"#)
            .to_string();

        CsharpComment(comment)
    }
}

impl fmt::Display for CsharpComment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let comment = &self.0;

        // Write the comment's summary message.
        writeln!(f, "{}", CommentTag {
            tag: "summary",
            content: &comment.message,
            attribute_name: "",
            attribute_value: ""
        })?;

        // Write deprecate reason if present
        if let Some(reason) = &comment.deprecate_reason {
            writeln!(f, "{}", CommentTag {
                tag: "para",
                content: reason,
                attribute_name: "",
                attribute_value: ""
            })?;
        }

        // Write each of the comment's parameter fields.
        for param in &comment.params {
            let (identifier, description) = param;
            writeln!(f, "{}", CommentTag {
                tag: "param",
                content: description,
                attribute_name: "name",
                attribute_value: &identifier
            })?;
        }

        // Write the comment's returns message if it has one.
        if let Some(returns) = &comment.returns {
            writeln!(f, "{}", CommentTag {
                tag: "returns",
                content: returns,
                attribute_name: "",
                attribute_value: ""
            })?;
        }

        // Write each of the comment's exception fields.
        for exception in &comment.throws {
            let (exception, description) = exception;
            writeln!(f, "{}", CommentTag {
                tag: "exceptions",
                content: description,
                attribute_name: "cref",
                attribute_value: exception
            })?;
        }

        Ok(())
    }
}

pub fn operation_doc_comment(operation: &Operation, dispatch: bool, ast: &Ast) -> CodeBlock {
    // let summary =
    //     CommentTag { tag: "summary", attribute_name: "", attribute_value: "", content: "" };
    // "".to_owned()

    let mut code = CodeBlock::new();

    if let Some(comment) = &operation.comment {
        let parsed_comment = CsharpComment::new(comment);
        code.writeln(&CommentTag {
            tag: "summary",
            attribute_name: "",
            attribute_value: "",
            content: &parsed_comment.0.message,
        });

        // TODO: write params (see writeParamDocComment in C++)
    }

    if dispatch {
        code.writeln(&CommentTag {
            tag: "param",
            attribute_name: "name",
            attribute_value: &escape_member_name(&operation.parameters(ast), "dispatch"),
            content: "The dispatch properties",
        });
    } else {
        code.writeln(&CommentTag {
            tag: "param",
            attribute_name: "name",
            attribute_value: &escape_member_name(&operation.parameters(ast), "invocation"),
            content: "The invocation properties.",
        });
    }

    code.writeln(&CommentTag {
        tag: "param",
        attribute_name: "name",
        attribute_value: &escape_member_name(&operation.parameters(ast), "cancel"),
        content: "A cancellation token that receives the cancellation requests.",
    });

    // TODO: return types (see C++)

    code
}
