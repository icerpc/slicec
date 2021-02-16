
mod cs_util;
mod cs_generator;
mod cs_options;

use structopt::StructOpt;
use cs_options::*;

pub fn main() {
    let mine: Vec<String> = Vec::new();
    mine.push("hello".to_owned());
    let thing1 = mine[0];
    let thing2 = mine[0];


    let opts = CsOptions::from_args();
    //let result = slice::parse(&opts.slice_options);

    //let visitor = CsVisitor::new(&opts);
    //visitor.visit(&result);
}