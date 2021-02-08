
extern crate pest_consume;

use pest_consume::Error;
use pest_consume::Parser;
use std::fs;

type ParseResult<T> = std::result::Result<T, Error<Rule>>;
type ParseNode<'a> = pest_consume::Node<'a, Rule, ()>;

#[derive(Parser)]
#[grammar = "slice.pest"]
struct SliceParser;

#[pest_consume::parser]
impl SliceParser {
    fn main(input: ParseNode) -> ParseResult<f64> {
        Ok(8.6)
    }
}

fn main() {
    let slice_file = fs::read_to_string("test.ice").expect("Wheres the file?!");

    let nodes = SliceParser::parse(Rule::main, &slice_file).unwrap();
    let root_node = nodes.single().unwrap();

    let tree = SliceParser::main(root_node);

    print!("{:?}", tree);
}
