// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::helpers::parsing_helpers::parse_for_ast;
use slice::grammar::*;
use test_case::test_case;

mod encoding;

#[test_case("bool", Primitive::Bool, None)]
#[test_case("int8", Primitive::Int8, None)]
#[test_case("uint8", Primitive::UInt8, None)]
#[test_case("int16", Primitive::Int16, None)]
#[test_case("uint16", Primitive::UInt16, None)]
#[test_case("int32", Primitive::Int32, None)]
#[test_case("uint32", Primitive::UInt32, None)]
#[test_case("varint32", Primitive::VarInt32, None)]
#[test_case("varuint32", Primitive::VarUInt32, None)]
#[test_case("int64", Primitive::Int64, None)]
#[test_case("uint64", Primitive::UInt64, None)]
#[test_case("varint62", Primitive::VarInt62, None)]
#[test_case("varuint62", Primitive::VarUInt62, None)]
#[test_case("float32", Primitive::Float32, None)]
#[test_case("float64", Primitive::Float64, None)]
#[test_case("string", Primitive::String, None)]
#[test_case("AnyClass", Primitive::AnyClass, Some("encoding = 1;"))]
fn primitive_types_parse(slice_component: &str, expected: Primitive, encoding: Option<&str>) {
    let slice = &format!(
        "
        {encoding}
        module Test;
        compact struct S {{
            i: {slice_component},
        }}
        ",
        encoding = encoding.unwrap_or(""),
        slice_component = slice_component
    );

    let ast = parse_for_ast(slice);

    let type_ptr = ast.find_typed_entity::<DataMember>("Test::S::i").unwrap();
    let primitive_ptr = type_ptr
        .borrow()
        .data_type()
        .definition
        .clone()
        .downcast::<Primitive>()
        .unwrap();
    let primitive = primitive_ptr.borrow();

    assert_eq!(
        std::mem::discriminant(primitive),
        std::mem::discriminant(&expected)
    );
}
