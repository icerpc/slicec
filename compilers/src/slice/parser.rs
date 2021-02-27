
use crate::ast::{SliceAst, IntoNode};
use crate::error::ErrorHandler;
use crate::grammar::*;
use crate::options::SliceOptions;
use crate::util::{Location, SliceFile};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use pest_consume::match_nodes;
use pest_consume::Error as PestError;
use pest_consume::Parser as PestParser;

type PestResult<T> = Result<T, PestError<Rule>>;
type PestNode<'a, 'b> = pest_consume::Node<'a, Rule, &'b RefCell<ParserData>>;

//------------------------------------------------------------------------------
// Parser Utility Functions
//------------------------------------------------------------------------------
fn from_span(input: &PestNode) -> Location {
    let span = input.as_span();
    Location {
        start: span.start_pos().line_col(),
        end: span.end_pos().line_col(),
        file: input.user_data().borrow().current_file.clone(),
    }
}

fn construct_type<'a, T: From<&'a str> + IntoNode + 'static>(data: &mut ParserData, type_name: &'a str) -> usize {
    // Check if we've already constructed this type's definition.
    // If we did already construct and store it, just return a copy of the index stored in the type table for it.
    // Otherwise we construct the type on-the-spot, store it, and then return it's index.
    let result = match data.type_table.get(type_name) {
        Some(definition) => { definition.clone() },
        None => {
            // Construct the type with a into-conversion.
            let definition: T = type_name.into();

            let index = data.ast.add_element(definition);
            data.type_table.insert(type_name.to_owned(), index);
            index
        }
    };

    // Ensure the correct type was constructed (or retrieved from the type table).
    debug_assert!(data.ast.resolve_index(result).type_id() == std::any::TypeId::of::<T>());

    result
}

//------------------------------------------------------------------------------
// ParserData
//------------------------------------------------------------------------------
#[derive(Debug, Default)]
struct ParserData {
    ast: SliceAst,
    definition_table: HashMap<String, usize>,
    type_table: HashMap<String, usize>,
    error_handler: ErrorHandler,
    current_file: String,
}

impl ParserData {
    fn new() -> Self {
        Self::default()
    }
}

//------------------------------------------------------------------------------
// SliceParser
//------------------------------------------------------------------------------
#[derive(PestParser)]
#[grammar = "slice/slice.pest"]
pub(crate) struct SliceParser {
    slice_files: HashMap<String, SliceFile>,
    user_data: RefCell<ParserData>,
}

impl SliceParser {
    pub(crate) fn parse_files(options: &SliceOptions) -> (SliceAst,
                                                          HashMap<String, SliceFile>,
                                                          HashMap<String, usize>,
                                                          ErrorHandler) {
        let mut parser = SliceParser::new();

        for path in options.sources.iter() {
            parser.try_parse_file(path.clone(), true);
        }
        for path in options.references.iter() {
            parser.try_parse_file(path.clone(), false);
        }

        let data = parser.user_data.into_inner();
        (data.ast, parser.slice_files, data.type_table, data.error_handler)
    }

    fn new() -> Self {
        SliceParser {
            slice_files: HashMap::new(),
            user_data: RefCell::new(ParserData::new()),
        }
    }

    fn try_parse_file(&mut self, file: String, is_source: bool) {
        match self.parse_file(&file, is_source) {
            Ok(slice_file) => {
                self.slice_files.insert(file, slice_file);
            },
            Err(message) => {
                let data = &mut self.user_data.borrow_mut();
                data.error_handler.report_error(message.as_str().into());
            },
        }
    }

    fn parse_file(&mut self, file: &String, is_source: bool) -> Result<SliceFile, String> {
        // We use an explicit scope to ensure the mutable borrow is dropped before the parser starts running.
        {
            // Mutably borrow the ParserData struct, to set it's current file.
            let data = &mut self.user_data.borrow_mut();
            data.current_file = file.clone();
        }

        // Read the raw text from the file, and parse it into a raw ast.
        let raw_text = fs::read_to_string(&file).map_err(|e| e.to_string())?;
        let node = SliceParser::parse_with_userdata(Rule::main, &raw_text, &self.user_data).map_err(|e| e.to_string())?; // TODO maybe make this error print prettier?
        let raw_ast = match node.single() {
            Ok(value) => value,
            Err(error) => panic!("Failed to unwrap raw_ast!\n{:?}", error),
        };

        // Consume the raw ast into an unpatched ast, then store it in a `SliceFile`.
        let file_contents = SliceParser::main(raw_ast).map_err(|e| e.to_string())?;
        Ok(SliceFile::new(file.clone(), raw_text, file_contents, is_source))
    }
}

//------------------------------------------------------------------------------
// pest_consume Implementation
//------------------------------------------------------------------------------
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
            [module_start(module_start)] => {
                Interface::new(module_start.0, module_start.1)
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
        Ok(Identifier::new(input.as_str().to_owned(), from_span(&input)))
    }

    fn scoped_identifier(input: PestNode) -> PestResult<Identifier> {
        Ok(Identifier::new(input.as_str().to_owned(), from_span(&input)))
    }

    fn typename(input: PestNode) -> PestResult<TypeUse> {
        let location = from_span(&input);
        // Remove any whitespace from the type name, then create the TypeUse.
        let type_name: String = input.as_str().chars().filter(|c| !c.is_whitespace()).collect();
        let mut type_use = TypeUse::new(type_name, false, location);

        // Resolve and/or construct non user defined types.
        let user_data = &mut input.user_data().borrow_mut();
        match_nodes!(input.children();
            [scoped_identifier(identifier)] => {
                // Nothing to do, we wait until after we've generated a lookup table to patch user defined types.
            },
            [builtin_type(builtin)] => {
                type_use.definition = Some(construct_type::<Builtin>(user_data, &type_use.type_name));
            }
        );
        Ok(type_use)
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
