// Copyright (c) ZeroC, Inc. All rights reserved.

mod container;
mod encoding;
mod inheritance;
mod tags;

use crate::helpers::parsing_helpers::parse_for_ast;
use slice::grammar::{Exception, NamedSymbol, Operation, Throws};

#[test]
fn throws_specific_exception() {
    let slice = "
        module Test;

        exception E
        {
        }

        interface I
        {
            op() throws E;
        }
    ";

    let ast = parse_for_ast(slice);
    let op = ast.find_element::<Operation>("Test::I::op").unwrap();

    match &op.throws {
        Throws::Specific(exception) => assert_eq!(
            exception.parser_scoped_identifier(),
            ast.find_element::<Exception>("Test::E")
                .unwrap()
                .parser_scoped_identifier()
        ),
        _ => panic!("Expected throws to be specific"),
    }
}

#[test]
fn throws_nothing() {
    let slice = "
        module Test;

        interface I
        {
            op();
        }
    ";

    let ast = parse_for_ast(slice);
    let op = ast.find_element::<Operation>("Test::I::op").unwrap();

    match &op.throws {
        Throws::None => (),
        _ => panic!("Expected throws to be nothing"),
    }
}

#[test]
fn throws_any_exception() {
    let slice = "
        encoding=1;
        module Test;

        interface I
        {
            op() throws AnyException;
        }
    ";

    let ast = parse_for_ast(slice);
    let op = ast.find_element::<Operation>("Test::I::op").unwrap();

    match &op.throws {
        Throws::AnyException => (),
        _ => panic!("Expected throws to be any"),
    }
}
