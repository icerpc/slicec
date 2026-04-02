// Copyright (c) ZeroC, Inc.

use crate::diagnostics::Lint;
use clap::ArgAction::Append;
use clap::{Parser, ValueEnum};

// Note: clap uses the doc-comments of fields to populate the '--help' output of slicec.
//       boolean flags automatically default to false, and strings automatically default to empty.

/// This struct is responsible for parsing the command line options of the 'slicec' compiler.
/// The option parsing capabilities are generated on the struct by the `clap::Parser` derive macro.
#[derive(Debug, Default, Parser)]
#[command(author, version, about, long_about = DESCRIPTION, rename_all = "kebab-case")]
pub struct SliceOptions {
    /// List of Slice files to compile.
    pub sources: Vec<String>,

    /// Add a directory or Slice file to the list of references.
    #[arg(short = 'R', num_args = 1, action = Append, value_name = "REFERENCE")]
    pub references: Vec<String>,

    /// Specify a code-generator plugin that should be run after parsing and validation complete (if successful).
    ///   Ex: '--generator /path/to/my/generator'
    ///
    /// Each code-generator can be provided with arbitrary string arguments using the following syntax:
    ///   '/path/to/my/generator;arg1=value1;arg2 = value2;arg3;...'
    /// Leading and trailing whitespace is stripped from arguments and their values. Argument values are optional.
    #[arg(short = 'G', long = "generator", num_args = 1, action = Append, value_name = "GENERATOR", value_parser = plugin_parser, verbatim_doc_comment)]
    pub generators: Vec<Plugin>,

    /// Set the output directory for the generated code. Defaults to the current working directory.
    #[arg(short = 'O', long, value_name = "DIRECTORY")]
    pub output_dir: Option<String>,

    /// Define a preprocessor symbol.
    #[arg(short = 'D', num_args = 1, action = Append, value_name = "SYMBOL")]
    pub defined_symbols: Vec<String>,

    /// Instruct the compiler to allow the specified lint.
    // TODO add a link to the lint reference in this doc comment!
    #[arg(short = 'A', long = "allow", num_args = 1, action = Append, value_name = "LINT_NAME", value_parser = Lint::ALLOWABLE_LINT_IDENTIFIERS, hide_possible_values = true, ignore_case = true)]
    pub allowed_lints: Vec<String>,

    /// Validate input files without generating code for them.
    #[arg(long)]
    pub dry_run: bool,

    /// Set which format to emit errors and warnings with.
    #[arg(long, value_name = "FORMAT", value_enum, default_value_t = DiagnosticFormat::Human, ignore_case = true)]
    pub diagnostic_format: DiagnosticFormat,

    /// Disable ANSI color codes in diagnostic output.
    #[arg(long)]
    pub disable_color: bool,
}

/// Short description of slicec that is displayed in its help dialogue.
const DESCRIPTION: &str = "\
The Slice compiler.
Parses Slice files into a typed Abstract Syntax Tree (AST) describing the provided Slice definitions.
This AST is encoded with Slice, and then output, to be consumed by other tools.";

fn plugin_parser<'a>(s: &str) -> Result<Plugin, &'a str> {
    // Helper enum to track what element the parser is currently parsing.
    #[derive(PartialEq, Eq)]
    enum State { Path, Key, Value }

    assert!(!s.is_empty());

    let mut plugin_path = String::new();
    let mut plugin_args = Vec::<(String, String)>::new();

    // State for our little value parser.
    let mut string_buffer = &mut plugin_path;
    let mut state = State::Path;

    // Iterate through the provided string and parse it into a 'plugin path + arguments'.
    let mut char_iter = s.chars().peekable();
    while let Some(c) = char_iter.next() {
        match c {
            // The next character after this is being escaped, add it directly to the buffer without parsing.
            '\\' => match char_iter.next() {
                Some(escaped_char) => string_buffer.push(escaped_char),
                None => return Err("unterminated escape sequence (for a literal '\\' character, use '\\\\')"),
            },

            ';' => {
                if string_buffer.is_empty() {
                    match state {
                        State::Path => return Err("missing plugin path (ex: 'PATH;KEY=VALUE')"),
                        State::Key => return Err("missing argument key (ex: 'PATH;KEY=VALUE')"),
                        State::Value => return Err("missing argument value (ex: 'PATH;KEY=VALUE' or 'PATH;KEY')"),
                    }
                }

                // We only handle ';' if there's more characters after it. If it's a trailing ';' we ignore it.
                if char_iter.peek().is_some() {
                    // Add a '(key=value)' argument pair, and re-target `string_buffer` to point at the key's buffer.
                    plugin_args.push(Default::default());
                    string_buffer = &mut plugin_args.last_mut().unwrap().0;
                    state = State::Key;
                }
            }

            '=' => match state {
                State::Path => string_buffer.push('='), // '=' has no special meaning in the plugin path.
                State::Key => {
                    if string_buffer.is_empty() {
                        return Err("missing argument key (ex: 'PATH;KEY=VALUE')");
                    }
                    if matches!(char_iter.peek(), None | Some(';')) {
                        return Err("missing argument value (ex: 'PATH;KEY=VALUE' or 'PATH;KEY')");
                    }

                    // Re-target `string_buffer` to point at the argument value's buffer (instead of the key).
                    string_buffer = &mut plugin_args.last_mut().unwrap().1;
                    state = State::Value;
                }
                State::Value => {
                    return Err("'=' can only appear once per argument (for a literal '=' character, use '\\=')")
                }
            },

            _ => string_buffer.push(c),
        }
    }

    // Trim any leading/trailing whitespace from the path and arguments.
    let path = plugin_path.trim().to_owned();
    let args = plugin_args
        .into_iter()
        .map(|(key, value)| (key.trim().to_owned(), value.trim().to_owned()))
        .collect();

    Ok(Plugin { path, args })
}

#[derive(Clone, Debug)]
pub struct Plugin {
    pub path: String,
    pub args: Vec<(String, String)>,
}

/// This enum is used to specify the format for emitted diagnostics.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, ValueEnum)]
pub enum DiagnosticFormat {
    /// Diagnostics are printed to the console in an easily readable format with source code snippets when possible.
    #[default]
    Human,

    /// Diagnostics will be serialized as JSON objects and printed to the console, one diagnostic per line.
    Json,
}
