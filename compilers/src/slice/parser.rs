
use crate::ast::{SliceAst, SliceFile};
use crate::grammar::*;
use crate::options::SliceOptions;
use crate::util::{Location, SliceError};

use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;

extern crate pest_consume;
use pest_consume::match_nodes;
use pest_consume::Error as PestError;
use pest_consume::Parser as PestParser;

#[allow(unused)] //TODO ONLY BECAUSE OF PEST_CONSUME
type PestResult<T> = Result<T, PestError<Rule>>;
#[allow(unused)] //TODO ONLY BECAUSE OF PEST_CONSUME
type PestNode<'a, 'b> = pest_consume::Node<'a, Rule, &'b RefCell<ParserData>>;

//------------------------------------------------------------------------------
// Parser Utility Functions
//------------------------------------------------------------------------------
#[allow(unused)] //TODO ONLY BECAUSE OF PEST_CONSUME
fn from_span(input: &PestNode) -> Location {
    let span = input.as_span();
    Location {
        start: span.start_pos().line_col(),
        end: span.end_pos().line_col(),
        file: input.user_data().borrow().current_file.clone(),
    }
}

//------------------------------------------------------------------------------
// ParserData
//------------------------------------------------------------------------------
pub struct ParserData {
    pub ast: SliceAst,
    pub current_file: String,
    pub errors: Vec<SliceError>,
}

impl ParserData {
    pub fn new() -> Self {
        ParserData {
            ast: SliceAst::new(),
            current_file: String::new(),
            errors: Vec::new(),
        }
    }

    //TODO add an error function here!
}

//------------------------------------------------------------------------------
// SliceParser
//------------------------------------------------------------------------------
#[derive(PestParser)]
#[grammar = "slice/slice.pest"]
pub struct SliceParser {
    slice_files: HashMap<String, SliceFile>,
    user_data: RefCell<ParserData>,
}

impl SliceParser {
    #[allow(unused_variables)] // TODO this is because we don't have any meaningful options yet.
    pub fn new(options: &SliceOptions) -> Self {
        SliceParser {
            slice_files: HashMap::new(),
            user_data: RefCell::new(ParserData::new()),
        }
    }

    pub fn parse_file(&mut self, file: String, is_source: bool) {
        // We use an explicit scope here so the mutable borrow is dropped before the rest of the method runs.
        {
            let data = &mut self.user_data.borrow_mut();
            data.current_file = file.clone();
        }

        // TODO we should convert all the \r\n and \r to \n BEFORE parsing!!! Otherwise text positions will change! Seriously do this!
        let raw_text = match fs::read_to_string(&file) {
            Ok(value) => value,
            Err(error) => return, //TODO report the error!
        };

        let nodes = match SliceParser::parse_with_userdata(Rule::main, &raw_text, &self.user_data) {
            Ok(value) => value,
            Err(error) => return, //TODO report the error!
        };

        let raw_ast = match nodes.single() {
            Ok(value) => value,
            Err(error) => panic!("Failed to unwrap raw_ast!\n{:?}", error),
        };

        let file_contents = match SliceParser::main(raw_ast) {
            Ok(value) => value,
            Err(error) => return, //TODO report the error!
        };

        let slice_file = SliceFile::new(file.clone(), raw_text, file_contents, is_source);
        self.slice_files.insert(file, slice_file);
    }

    pub fn into_data(self) -> (SliceAst, HashMap<String, SliceFile>, Vec<SliceError>) {
        let data = self.user_data.into_inner();
        (data.ast, self.slice_files, data.errors)
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
        Ok(ast.add_node(Box::new(module_def)))
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
        Ok(ast.add_node(Box::new(struct_def)))
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
            [module_start(module_start)] => {
                Interface::new(module_start.0, module_start.1)
            }
        );
        let ast = &mut input.user_data().borrow_mut().ast;
        Ok(ast.add_node(Box::new(interface_def)))
    }

    fn data_member(input: PestNode) -> PestResult<usize> {
        let location = from_span(&input);
        let data_member = match_nodes!(input.children();
            [typename(data_type), identifier(identifier)] => {
                DataMember::new(data_type, identifier, location)
            }
        );
        let ast = &mut input.user_data().borrow_mut().ast;
        Ok(ast.add_node(Box::new(data_member)))
    }

    fn identifier(input: PestNode) -> PestResult<Identifier> {
        Ok(Identifier::new(input.as_str().to_owned(), from_span(&input)))
    }

    fn scoped_identifier(input: PestNode) -> PestResult<Identifier> {
        Ok(Identifier::new(input.as_str().to_owned(), from_span(&input)))
    }

    fn typename(input: PestNode) -> PestResult<TypeUse> {
        Ok(TypeUse::new(input.as_str().to_owned(), false, from_span(&input)))
    }

    fn builtin_type(input: PestNode) -> PestResult<()> {
        Ok(())
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

    fn int_kw(input: PestNode) -> PestResult<()> {
        Ok(())
    }

    fn string_kw(input: PestNode) -> PestResult<()> {
        Ok(())
    }

    fn EOI(input: PestNode) -> PestResult<()> {
        Ok(())
    }
}
