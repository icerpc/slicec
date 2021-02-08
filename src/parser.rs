
use crate::definitions::*;

extern crate pest_consume;
use pest_consume::Error;
use pest_consume::Parser;

type ParseResult<T> = std::result::Result<T, Error<Rule>>;
type ParseNode<'a> = pest_consume::Node<'a, Rule, ()>;

#[derive(Parser)]
#[grammar = "slice.pest"]
pub struct SliceParser;

#[pest_consume::parser]
impl SliceParser {
    fn main(input: ParseNode) -> ParseResult<Vec<Module>> {
        Ok(())
    }

    fn definitions(input: ParseNode) -> ParseResult<()> {
        Ok(())
    }

    fn module_def(input: ParseNode) -> ParseResult<()> {
        Ok(())
    }

    fn struct_def(input: ParseNode) -> ParseResult<()> {
        Ok(())
    }

    fn interface_def(input: ParseNode) -> ParseResult<()> {
        Ok(())
    }

    fn data_member(input: ParseNode) -> ParseResult<()> {
        Ok(())
    }

    fn identifier(input: ParseNode) -> ParseResult<()> {
        Ok(())
    }

    fn scoped_identifier(input: ParseNode) -> ParseResult<()> {
        Ok(())
    }

    fn typename(input: ParseNode) -> ParseResult<()> {
        Ok(())
    }

    fn builtin_type(input: ParseNode) -> ParseResult<()> {
        Ok(())
    }

    fn module_kw(input: ParseNode) -> ParseResult<()> {
        Ok(())
    }

    fn struct_kw(input: ParseNode) -> ParseResult<()> {
        Ok(())
    }

    fn interface_kw(input: ParseNode) -> ParseResult<()> {
        Ok(())
    }

    fn int_kw(input: ParseNode) -> ParseResult<()> {
        Ok(())
    }

    fn string_kw(input: ParseNode) -> ParseResult<()> {
        Ok(())
    }
}

pub fn parse(input: &String) -> Result<Vec<Module>, Error<Rule>>
{
    let nodes = SliceParser::parse(Rule::main, &input)?;
    let root_node = nodes.single()?;

    SliceParser::main(root_node)
}
