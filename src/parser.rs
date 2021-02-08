
use crate::definitions::*;

extern crate pest_consume;
use pest_consume::{match_nodes, Error, Parser};

type ParseResult<T> = std::result::Result<T, Error<Rule>>;
type ParseNode<'a> = pest_consume::Node<'a, Rule, ()>;



#[derive(Parser)]
#[grammar = "slice.pest"]
pub struct SliceParser;



#[pest_consume::parser]
impl SliceParser {
    fn main(input: ParseNode) -> ParseResult<Vec<Module>> {
        Ok(match_nodes!(input.into_children();
            [module_def(modules).., EOI(_)] => { modules.collect() }
        ))
    }

    fn definition(input: ParseNode) -> ParseResult<Box<dyn Definition>> {
        Ok(match_nodes!(input.into_children();
            [module_def(module_def)]       => { Box::new(module_def) },
            [struct_def(struct_def)]       => { Box::new(struct_def) },
            [interface_def(interface_def)] => { Box::new(interface_def) }
        ))
    }

    fn module_def(input: ParseNode) -> ParseResult<Module> {
        Ok(match_nodes!(input.into_children();
            [_, identifier(identifier), definition(definitions)..] => {
                Module {
                    identifier: identifier,
                    content: definitions.collect(),
                }
            }
        ))
    }

    fn struct_def(input: ParseNode) -> ParseResult<Struct> {
        Ok(match_nodes!(input.into_children();
            [_, identifier(identifier), data_member(members)..] => {
                Struct {
                    identifier: identifier,
                    content: members.collect(),
                }
            }
        ))
    }

    fn interface_def(input: ParseNode) -> ParseResult<Interface> {
        Ok(match_nodes!(input.into_children();
            [_, identifier(identifier)] => {
                Interface {
                    identifier: identifier,
                }
            },
        ))
    }

    fn data_member(input: ParseNode) -> ParseResult<DataMember> {
        Ok(match_nodes!(input.into_children();
            [typename(typename), identifier(identifier)] => {
                DataMember {
                    typename: typename,
                    identifier: identifier,
                }
            },
        ))
    }

    fn typename(input: ParseNode) -> ParseResult<Type> {
        Ok(input.as_str().parse::<Type>().unwrap())
    }

    fn identifier(input: ParseNode) -> ParseResult<String> {
        Ok(input.as_str().to_owned())
    }

    fn scoped_identifier(input: ParseNode) -> ParseResult<String> {
        Ok(input.as_str().to_owned())
    }

    fn module_kw(input:ParseNode) -> ParseResult<()> { Ok(()) }
    fn struct_kw(input:ParseNode) -> ParseResult<()> { Ok(()) }
    fn interface_kw(input:ParseNode) -> ParseResult<()> { Ok(()) }
    fn int_kw(input:ParseNode) -> ParseResult<()> { Ok(()) }
    fn string_kw(input:ParseNode) -> ParseResult<()> { Ok(()) }

    fn EOI(input: ParseNode) -> ParseResult<()> {
        Ok(())
    }
}



pub fn parse(input: &String) -> Result<Vec<Module>, Error<Rule>>
{
    let nodes = SliceParser::parse(Rule::main, &input)?;
    let root = nodes.single()?;

    SliceParser::main(root)
}
