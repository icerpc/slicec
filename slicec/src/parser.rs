// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::mut_ref_from_node;
use crate::ast::{Ast, Node};
use crate::comment_parser::CommentParser;
use crate::error::ErrorHandler;
use crate::grammar::*;
use crate::options::SliceOptions;
use crate::util::{Location, SliceFile};
use pest::error::ErrorVariant as PestErrorVariant;
use pest_consume::match_nodes;
use pest_consume::Error as PestError;
use pest_consume::Parser as PestParser;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;

type PestResult<T> = Result<T, PestError<Rule>>;
type PestNode<'a, 'b> = pest_consume::Node<'a, Rule, &'b RefCell<ParserData>>;

fn from_span(input: &PestNode) -> Location {
    let span = input.as_span();
    Location {
        start: span.start_pos().line_col(),
        end: span.end_pos().line_col(),
        file: input.user_data().borrow().current_file.clone(),
    }
}

#[derive(Debug, Default)]
struct ParserData {
    ast: Ast,
    error_handler: ErrorHandler,
    current_file: String,
    current_enum_value: i64,
}

impl ParserData {
    fn new() -> Self {
        Self::default()
    }
}

#[derive(PestParser)]
#[grammar = "slice.pest"]
pub(crate) struct SliceParser {
    slice_files: HashMap<String, SliceFile>,
    user_data: RefCell<ParserData>,
}

impl SliceParser {
    pub(crate) fn parse_files(options: &SliceOptions) -> (Ast,
                                                          HashMap<String, SliceFile>,
                                                          ErrorHandler) {
        let mut parser = SliceParser::new();

        for path in options.sources.iter() {
            parser.try_parse_file(path, true);
        }
        for path in options.references.iter() {
            parser.try_parse_file(path, false);
        }

        let data = parser.user_data.into_inner();
        (data.ast, parser.slice_files, data.error_handler)
    }

    fn new() -> Self {
        SliceParser {
            slice_files: HashMap::new(),
            user_data: RefCell::new(ParserData::new()),
        }
    }

    fn try_parse_file(&mut self, file: &str, is_source: bool) {
        match self.parse_file(file, is_source) {
            Ok(slice_file) => {
                self.slice_files.insert(file.to_owned(), slice_file);
            }
            Err(message) => {
                let data = &mut self.user_data.borrow_mut();
                data.error_handler.report_error(message.into());
            }
        }
    }

    fn parse_file(&mut self, file: &str, is_source: bool) -> Result<SliceFile, String> {
        // We use an explicit scope to ensure the mutable borrow is dropped before the parser starts running.
        {
            // Mutably borrow the ParserData struct, to set its current file.
            let data = &mut self.user_data.borrow_mut();
            data.current_file = file.to_owned();
        }

        // Read the raw text from the file, and parse it into a raw ast.
        let raw_text = fs::read_to_string(&file).map_err(|e| e.to_string())?;
        let node = SliceParser::parse_with_userdata(Rule::main, &raw_text, &self.user_data).map_err(|e| e.to_string())?; // TODO maybe make this error print prettier?
        let raw_ast = node.single().expect("Failed to unwrap raw_ast!");

        // Consume the raw ast into an unpatched ast, then store it in a `SliceFile`.
        let file_contents = SliceParser::main(raw_ast).map_err(|e| e.to_string())?;
        Ok(SliceFile::new(file.to_owned(), raw_text, file_contents, is_source))
    }
}

#[pest_consume::parser]
impl SliceParser {
    fn main(input: PestNode) -> PestResult<Vec<usize>> {
        let module_ids = match_nodes!(input.into_children();
            [module_def(ids).., EOI(_)] => { ids.collect() },
        );
        Ok(module_ids)
    }

    fn definition(input: PestNode) -> PestResult<usize> {
        let definition_id = match_nodes!(input.into_children();
            [module_def(id)]    => { id },
            [struct_def(id)]    => { id },
            [interface_def(id)] => { id },
            [enum_def(id)]      => { id },
        );
        Ok(definition_id)
    }

    fn module_start(input: PestNode) -> PestResult<(Identifier, Location)> {
        let location = from_span(&input);
        let identifier = match_nodes!(input.into_children();
            [_, identifier(ident)] => { ident },
        );
        Ok((identifier, location))
    }

    fn module_def(input: PestNode) -> PestResult<usize> {
        let module_def = match_nodes!(input.children();
            [doc_comment(comment), module_start(module_start), definition(contents)..] => {
                Module::new(module_start.0, contents.collect(), module_start.1, comment)
            },
        );
        let ast = &mut input.user_data().borrow_mut().ast;
        Ok(ast.add_element(module_def))
    }

    fn struct_start(input: PestNode) -> PestResult<(Identifier, Location)> {
        let location = from_span(&input);
        let identifier = match_nodes!(input.into_children();
            [_, identifier(ident)] => { ident },
        );
        Ok((identifier, location))
    }

    fn struct_def(input: PestNode) -> PestResult<usize> {
        let struct_def = match_nodes!(input.children();
            [doc_comment(comment), struct_start(struct_start), data_member(members)..] => {
                Struct::new(struct_start.0, members.collect(), struct_start.1, comment)
            },
        );
        let ast = &mut input.user_data().borrow_mut().ast;
        Ok(ast.add_element(struct_def))
    }

    fn interface_start(input: PestNode) -> PestResult<(Identifier, Location)> {
        let location = from_span(&input);
        let identifier = match_nodes!(input.into_children();
            [_, identifier(ident)] => { ident },
        );
        Ok((identifier, location))
    }

    fn interface_def(input: PestNode) -> PestResult<usize> {
        let interface_def = match_nodes!(input.children();
            [doc_comment(comment), interface_start(interface_start), operation(operations)..] => {
                Interface::new(interface_start.0, operations.collect(), interface_start.1, comment)
            },
        );
        let ast = &mut input.user_data().borrow_mut().ast;
        Ok(ast.add_element(interface_def))
    }

    fn enum_start(input: PestNode) -> PestResult<(bool, Identifier, Location, Option<TypeRef>)> {
        // Reset the current enumerator value back to 0.
        input.user_data().borrow_mut().current_enum_value = 0;

        let location = from_span(&input);
        Ok(match_nodes!(input.into_children();
            [unchecked_modifier(unchecked), _, identifier(ident)] => {
                (unchecked, ident, location, None)
            },
            [unchecked_modifier(unchecked), _, identifier(ident), typename(underlying)] => {
                (unchecked, ident, location, Some(underlying))
            },
        ))
    }

    fn enum_def(input: PestNode) -> PestResult<usize> {
        let enum_def = match_nodes!(input.children();
            [doc_comment(comment), enum_start(enum_start), enumerator_list(enumerators)] => {
                Enum::new(
                    enum_start.1,
                    enumerators,
                    enum_start.0,
                    enum_start.3,
                    enum_start.2,
                    comment,
                )
            },
            [doc_comment(comment), enum_start(enum_start)] => {
                Enum::new(
                    enum_start.1,
                    Vec::new(),
                    enum_start.0,
                    enum_start.3,
                    enum_start.2,
                    comment,
                )
            },
        );
        let ast = &mut input.user_data().borrow_mut().ast;
        Ok(ast.add_element(enum_def))
    }

    // Parses an operation's return type. There are 3 possible syntaxes for a return type:
    //   A void return type, specified by the `void` keyword.
    //   A single unnamed return type, specified by a typename.
    //   A return tuple, specified as a list of named elements enclosed in parenthesis.
    fn return_type(input: PestNode) -> PestResult<ReturnType> {
        let location = from_span(&input);
        Ok(match_nodes!(input.into_children();
            [void_kw(_)] => {
                ReturnType::Void(location)
            },
            [typename(data_type)] => {
                ReturnType::Single(data_type, location)
            },
            [return_tuple(tuple)] => {
                ReturnType::Tuple(tuple, location)
            },
        ))
    }

    // Parses a return type that is written in return tuple syntax.
    fn return_tuple(input: PestNode) -> PestResult<Vec<usize>> {
        Ok(match_nodes!(input.children();
            // Return tuple elements and parameters have the same syntax, so we re-use the parsing
            // for parameter lists, then change their member type here, after the fact.
            [parameter_list(return_elements)] => {
                let ast = &mut input.user_data().borrow_mut().ast;
                for id in return_elements.iter() {
                    let return_element = mut_ref_from_node!(Node::Member, ast, *id);
                    return_element.member_type = MemberType::ReturnElement;
                }
                return_elements
            },
        ))
    }

    fn operation(input: PestNode) -> PestResult<usize> {
        let location = from_span(&input);
        let operation = match_nodes!(input.children();
            [doc_comment(comment), return_type(return_type), identifier(identifier)] => {
                Operation::new(return_type, identifier, Vec::new(), location, comment)
            },
            [doc_comment(comment), return_type(return_type), identifier(identifier), parameter_list(parameters)] => {
                Operation::new(return_type, identifier, parameters, location, comment)
            },
        );
        let ast = &mut input.user_data().borrow_mut().ast;
        Ok(ast.add_element(operation))
    }

    fn data_member(input: PestNode) -> PestResult<usize> {
        let location = from_span(&input);
        let data_member = match_nodes!(input.children();
            [doc_comment(comment), typename(data_type), identifier(identifier)] => {
                Member::new(data_type, identifier, MemberType::DataMember, location, comment)
            },
        );
        let ast = &mut input.user_data().borrow_mut().ast;
        Ok(ast.add_element(data_member))
    }

    fn parameter_list(input: PestNode) -> PestResult<Vec<usize>> {
        Ok(match_nodes!(input.into_children();
            [parameter(parameter_id)] => {
                vec![parameter_id]
            },
            [parameter(parameter_id), parameter_list(mut list)] => {
                // The parameter comes before the parameter_list when parsing, so we have to
                // insert the new parameter at the front of the list.
                list.insert(0, parameter_id);
                list
            },
        ))
    }

    fn parameter(input: PestNode) -> PestResult<usize> {
        let location = from_span(&input);
        let parameter = match_nodes!(input.children();
            [doc_comment(comment), typename(data_type), identifier(identifier)] => {
                Member::new(data_type, identifier, MemberType::Parameter, location, comment)
            },
        );
        let ast = &mut input.user_data().borrow_mut().ast;
        Ok(ast.add_element(parameter))
    }

    fn enumerator_list(input: PestNode) -> PestResult<Vec<usize>> {
        Ok(match_nodes!(input.into_children();
            [enumerator(enumerator_id)] => {
                vec![enumerator_id]
            },
            [enumerator(enumerator_id), enumerator_list(mut list)] => {
                // The enumerator comes before the enumerator_list when parsing, so we have to
                // insert the new enumerator at the front of the list.
                list.insert(0, enumerator_id);
                list
            },
        ))
    }

    fn enumerator(input: PestNode) -> PestResult<usize> {
        let location = from_span(&input);
        let mut next_enum_value = input.user_data().borrow().current_enum_value;

        let enumerator_def = match_nodes!(input.children();
            [doc_comment(comment), identifier(ident)] => {
                Enumerator::new(ident, next_enum_value, location, comment)
            },
            [doc_comment(comment), identifier(ident), integer(value)] => {
                next_enum_value = value;
                Enumerator::new(ident, value, location, comment)
            },
        );

        let parser_data = &mut input.user_data().borrow_mut();
        parser_data.current_enum_value = next_enum_value + 1;
        Ok(parser_data.ast.add_element(enumerator_def))
    }

    fn identifier(input: PestNode) -> PestResult<Identifier> {
        Ok(Identifier::new(
            input.as_str().to_owned(),
            from_span(&input),
        ))
    }

    fn scoped_identifier(input: PestNode) -> PestResult<Identifier> {
        Ok(Identifier::new(
            input.as_str().to_owned(),
            from_span(&input),
        ))
    }

    fn global_identifier(input: PestNode) -> PestResult<Identifier> {
        Ok(Identifier::new(
            input.as_str().to_owned(),
            from_span(&input),
        ))
    }

    fn typename(input: PestNode) -> PestResult<TypeRef> {
        let location = from_span(&input);
        // Remove any whitespace from the type name, then create the TypeRef.
        let type_name: String = input.as_str().chars().filter(|c| !c.is_whitespace()).collect();
        let mut type_use = TypeRef::new(type_name, false, location);

        // Resolve and/or construct non user defined types.
        match_nodes!(input.children();
            [primitive(primitive)] => {
                let ast = &mut input.user_data().borrow_mut().ast;
                type_use.definition = Some(ast.add_primitive(primitive));
            },
            [sequence(sequence)] => {
                let ast = &mut input.user_data().borrow_mut().ast;
                type_use.definition = Some(ast.add_element(sequence));
            },
            [dictionary(dictionary)] => {
                let ast = &mut input.user_data().borrow_mut().ast;
                type_use.definition = Some(ast.add_element(dictionary));
            },
            [global_identifier(identifier)] => {
                // Nothing to do, we wait until after we've generated a lookup table to patch user defined types.
            },
            [scoped_identifier(identifier)] => {
                // Nothing to do, we wait until after we've generated a lookup table to patch user defined types.
            },
        );
        Ok(type_use)
    }

    fn sequence(input: PestNode) -> PestResult<Sequence> {
        Ok(match_nodes!(input.into_children();
            [_, typename(element_type)] => {
                Sequence::new(element_type)
            },
        ))
    }

    fn dictionary(input: PestNode) -> PestResult<Dictionary> {
        Ok(match_nodes!(input.into_children();
            [_, typename(key_type), typename(value_type)] => {
                Dictionary::new(key_type, value_type)
            },
        ))
    }

    fn primitive(input: PestNode) -> PestResult<Primitive> {
        Ok(match_nodes!(input.into_children();
            [bool_kw(bool_kw)]         => Primitive::Bool,
            [byte_kw(byte_kw)]         => Primitive::Byte,
            [short_kw(short_kw)]       => Primitive::Short,
            [ushort_kw(ushort_kw)]     => Primitive::UShort,
            [int_kw(int_kw)]           => Primitive::Int,
            [uint_kw(uint_kw)]         => Primitive::UInt,
            [varint_kw(varint_kw)]     => Primitive::VarInt,
            [varuint_kw(varuint_kw)]   => Primitive::VarUInt,
            [long_kw(long_kw)]         => Primitive::Long,
            [ulong_kw(ulong_kw)]       => Primitive::ULong,
            [varlong_kw(varlong_kw)]   => Primitive::VarLong,
            [varulong_kw(varulong_kw)] => Primitive::VarULong,
            [float_kw(float_kw)]       => Primitive::Float,
            [double_kw(double_kw)]     => Primitive::Double,
            [string_kw(string_kw)]     => Primitive::String
        ))
    }

    fn unchecked_modifier(input: PestNode) -> PestResult<bool> {
        Ok(match_nodes!(input.into_children();
            []                => false,
            [unchecked_kw(_)] => true
        ))
    }

    fn integer(input: PestNode) -> PestResult<i64> {
        let int = input.as_str().parse::<i64>();
        match int {
            Ok(int) => Ok(int),
            Err(err) => {
                Err(PestError::new_from_span(
                    PestErrorVariant::CustomError { message: format!("Malformed integer: {}", err)},
                    input.as_span(),
                ))
            }
        }
    }

    fn doc_comment(input: PestNode) -> PestResult<Option<DocComment>> {
        let location = from_span(&input);
        Ok(match_nodes!(input.into_children();
            [] => {
                None
            },
            [line_doc_comment(comments)..] => {
                // Merge all the line comments together.
                let combined = comments.collect::<Vec<String>>().join("\n");
                Some(CommentParser::parse_doc_comment(&combined, location))
            },
            [block_doc_comment(comment)] => {
                Some(CommentParser::parse_doc_comment(&comment, location))
            }
        ))
    }

    fn line_doc_comment(input: PestNode) -> PestResult<String> {
        Ok(input.as_str().to_owned())
    }

    fn block_doc_comment(input: PestNode) -> PestResult<String> {
        Ok(input.as_str().to_owned())
    }

    fn module_kw(input: PestNode) -> PestResult<()> {
        Ok(())
    }

    fn struct_kw(input: PestNode) -> PestResult<()> {
        Ok(())
    }

    fn interface_kw(input: PestNode) -> PestResult<()> {
        Ok(())
    }

    fn enum_kw(input: PestNode) -> PestResult<()> {
        Ok(())
    }

    fn sequence_kw(input: PestNode) -> PestResult<()> {
        Ok(())
    }

    fn dictionary_kw(input: PestNode) -> PestResult<()> {
        Ok(())
    }

    fn void_kw(input: PestNode) -> PestResult<()> {
        Ok(())
    }

    fn bool_kw(input: PestNode) -> PestResult<()> {
        Ok(())
    }

    fn byte_kw(input: PestNode) -> PestResult<()> {
        Ok(())
    }

    fn short_kw(input: PestNode) -> PestResult<()> {
        Ok(())
    }

    fn ushort_kw(input: PestNode) -> PestResult<()> {
        Ok(())
    }

    fn int_kw(input: PestNode) -> PestResult<()> {
        Ok(())
    }

    fn uint_kw(input: PestNode) -> PestResult<()> {
        Ok(())
    }

    fn varint_kw(input: PestNode) -> PestResult<()> {
        Ok(())
    }

    fn varuint_kw(input: PestNode) -> PestResult<()> {
        Ok(())
    }

    fn long_kw(input: PestNode) -> PestResult<()> {
        Ok(())
    }

    fn ulong_kw(input: PestNode) -> PestResult<()> {
        Ok(())
    }

    fn varlong_kw(input: PestNode) -> PestResult<()> {
        Ok(())
    }

    fn varulong_kw(input: PestNode) -> PestResult<()> {
        Ok(())
    }

    fn float_kw(input: PestNode) -> PestResult<()> {
        Ok(())
    }

    fn double_kw(input: PestNode) -> PestResult<()> {
        Ok(())
    }

    fn string_kw(input: PestNode) -> PestResult<()> {
        Ok(())
    }

    fn unchecked_kw(input: PestNode) -> PestResult<()> {
        Ok(())
    }

    fn EOI(input: PestNode) -> PestResult<()> {
        Ok(())
    }
}
