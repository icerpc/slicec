
use crate::grammar::*;
use crate::util::Location;

extern crate pest;
use pest::Span;

extern crate pest_consume;
use pest_consume::match_nodes;
use pest_consume::Error as PestError;
use pest_consume::Parser as PestParser;

type PestResult<T> = Result<T, PestError<Rule>>;
type PestNode<'a, 'b> = pest_consume::Node<'a, Rule, &'b std::cell::RefCell<ParserData>>;

//------------------------------------------------------------------------------
// Parser Utility Functions
//------------------------------------------------------------------------------
fn from_span(span: &Span) -> Location {
    Location { start: span.start(), end: span.end() }
}

fn store_definition(data_holder: &std::cell::RefCell<ParserData>, definition: Box<dyn Definition>) -> usize {
    let mut parser_data = data_holder.borrow_mut();
    let mut ast = &mut parser_data.ast;

    ast.push(definition);
    ast.len() - 1
}

//------------------------------------------------------------------------------
// Definition
//------------------------------------------------------------------------------
pub trait Definition : Node {}

impl Definition for Module {}
impl Definition for Struct {}
impl Definition for Interface {}

//------------------------------------------------------------------------------
// ParserData
//------------------------------------------------------------------------------
pub struct ParserData {
    pub ast: Vec<Box<dyn Definition>>,
}

impl ParserData {
    pub fn new() -> Self {
        ParserData { ast: Vec::new() }
    }
}

//------------------------------------------------------------------------------
// SliceParser
//------------------------------------------------------------------------------
#[derive(PestParser)]
#[grammar = "slice/slice.pest"]
pub struct SliceParser;

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
        let location = from_span(&input.as_span());
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
        Ok(store_definition(input.user_data(), Box::new(module_def)))
    }

    fn struct_start(input: PestNode) -> PestResult<(Identifier, Location)> {
        let location = from_span(&input.as_span());
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
        Ok(store_definition(input.user_data(), Box::new(struct_def)))
    }

    fn interface_start(input: PestNode) -> PestResult<(Identifier, Location)> {
        let location = from_span(&input.as_span());
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
        Ok(store_definition(input.user_data(), Box::new(interface_def)))
    }

    fn data_member(input: PestNode) -> PestResult<DataMember> {
        let location = from_span(&input.as_span());

        let data_member = match_nodes!(input.into_children();
            [typename(data_type), identifier(identifier)] => {
                DataMember::new(data_type, identifier, location)
            }
        );
        Ok(data_member)
    }

    fn identifier(input: PestNode) -> PestResult<Identifier> {
        Ok(Identifier::new(input.as_str().to_owned(), from_span(&input.as_span())))
    }

    fn scoped_identifier(input: PestNode) -> PestResult<Identifier> {
        Ok(Identifier::new(input.as_str().to_owned(), from_span(&input.as_span())))
    }

    fn typename(input: PestNode) -> PestResult<TypeUse> {
        Ok(TypeUse::new(input.as_str().to_owned(), false, from_span(&input.as_span())))
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
