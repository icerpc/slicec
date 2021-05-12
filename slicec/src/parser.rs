// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::ast::Ast;
use crate::error::ErrorHandler;
use crate::grammar::*;
use crate::options::SliceOptions;
use crate::util::{Location, SliceFile};
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
            // Mutably borrow the ParserData struct, to set it's current file.
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
            [module_def(ids).., EOI(_)] => { ids.collect() }
        );
        Ok(module_ids)
    }

    fn definition(input: PestNode) -> PestResult<usize> {
        let definition_id = match_nodes!(input.into_children();
            [module_def(id)]    => { id },
            [struct_def(id)]    => { id },
            [interface_def(id)] => { id },
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
            [module_start(module_start), definition(contents)..] => {
                Module::new(module_start.0, contents.collect(), module_start.1)
            }
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
            [struct_start(struct_start), data_member(members)..] => {
                Struct::new(struct_start.0, members.collect(), struct_start.1)
            }
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
            [interface_start(interface_start)] => {
                Interface::new(interface_start.0, interface_start.1)
            }
        );
        let ast = &mut input.user_data().borrow_mut().ast;
        Ok(ast.add_element(interface_def))
    }

    fn data_member(input: PestNode) -> PestResult<usize> {
        let location = from_span(&input);
        let data_member = match_nodes!(input.children();
            [typename(data_type), identifier(identifier)] => {
                DataMember::new(data_type, identifier, location)
            }
        );
        let ast = &mut input.user_data().borrow_mut().ast;
        Ok(ast.add_element(data_member))
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
            [global_identifier(identifier)] => {
                // Nothing to do, we wait until after we've generated a lookup table to patch user defined types.
            },
            [scoped_identifier(identifier)] => {
                // Nothing to do, we wait until after we've generated a lookup table to patch user defined types.
            },
            [primitive(primitive)] => {
                let ast = &mut input.user_data().borrow_mut().ast;
                type_use.definition = Some(ast.add_primitive(primitive));
            }
        );
        Ok(type_use)
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

    fn module_kw(input: PestNode) -> PestResult<()> {
        Ok(())
    }

    fn struct_kw(input: PestNode) -> PestResult<()> {
        Ok(())
    }

    fn interface_kw(input: PestNode) -> PestResult<()> {
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

    fn EOI(input: PestNode) -> PestResult<()> {
        Ok(())
    }
}
