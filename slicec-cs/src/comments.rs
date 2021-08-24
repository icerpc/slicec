use slice::grammar::NamedSymbol;
use slice::writer::Writer;

/// Helper method that checks if a named symbol has a comment written on it, and if so, formats
/// it as a C# style doc comment and writes it to the underlying output.
pub fn write_comment(writer: &mut Writer, named_symbol: &dyn NamedSymbol) {
    // If the symbol has a doc comment attached to it, write it's fields to the output.
    if let Some(comment) = &named_symbol.comment() {
        // Write the comment's summary message if it has one.
        if !comment.message.is_empty() {
            write_comment_field(writer, "summary", &comment.message, "");
        }

        // Write each of the comment's parameter fields.
        for param in &comment.params {
            let (identifier, description) = param;
            let attribute = format!(r#" name="{}""#, identifier);
            write_comment_field(writer, "param", description, &attribute);
        }

        // Write the comment's returns message if it has one.
        if let Some(returns) = &comment.returns {
            write_comment_field(writer, "returns", returns, "");
        }

        // Write each of the comment's exception fields.
        for exception in &comment.throws {
            let (exception, description) = exception;
            let attribute = format!(r#" cref="{}""#, exception);
            write_comment_field(writer, "exceptions", description, &attribute);
        }
    }
}

pub fn write_comment_field(output: &mut Writer, field_name: &str, content: &str, attribute: &str) {
    let mut field_string = format!("/// <{}{}>", field_name, attribute);
    if !content.is_empty() {
        // Iterate through each line of the field's content, and at the end of each line, append
        // a newline followed by 3 forward slashes to continue the comment.
        for line in content.lines() {
            field_string += line;
            field_string += "\n/// ";
        }
        // Remove the trailing newline and slashes by truncating off the last 5 characters.
        field_string.truncate(field_string.len() - 5);
    } // Append a closing tag, and write the field.
    field_string = field_string + "</" + field_name + ">\n";
    output.write(&field_string);
}
