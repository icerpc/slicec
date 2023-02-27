// Copyright (c) ZeroC, Inc.

mod encoding;

use crate::helpers::parsing_helpers::parse_for_ast;
use slice::grammar::*;
use test_case::test_case;

#[test_case("bool", Primitive::Bool, None; "bool")]
#[test_case("int8", Primitive::Int8, None; "int8")]
#[test_case("uint8", Primitive::UInt8, None; "uint8")]
#[test_case("int16", Primitive::Int16, None; "int16")]
#[test_case("uint16", Primitive::UInt16, None; "uint16")]
#[test_case("int32", Primitive::Int32, None; "int32")]
#[test_case("uint32", Primitive::UInt32, None; "uint32")]
#[test_case("varint32", Primitive::VarInt32, None; "varint32")]
#[test_case("varuint32", Primitive::VarUInt32, None; "varuint32")]
#[test_case("int64", Primitive::Int64, None; "int64")]
#[test_case("uint64", Primitive::UInt64, None; "uint64")]
#[test_case("varint62", Primitive::VarInt62, None; "varint62")]
#[test_case("varuint62", Primitive::VarUInt62, None; "varuint62")]
#[test_case("float32", Primitive::Float32, None; "float32")]
#[test_case("float64", Primitive::Float64, None; "float64")]
#[test_case("string", Primitive::String, None; "string")]
#[test_case("AnyClass", Primitive::AnyClass, Some("encoding = 1"); "AnyClass")]
fn type_parses(slice_component: &str, expected: Primitive, encoding: Option<&str>) {
    // Arrange
    let encoding = encoding.unwrap_or("");
    let slice = format!(
        "
            {encoding}
            module Test

            typealias P = {slice_component}
        "
    );

    // Act
    let ast = parse_for_ast(slice);

    // Assert
    let underlying = &ast.find_element::<TypeAlias>("Test::P").unwrap().underlying;
    if let TypeRefDefinition::Patched(ptr) = &underlying.definition {
        let primitive = ptr.clone().downcast::<Primitive>().unwrap();
        assert_eq!(
            std::mem::discriminant(primitive.borrow()),
            std::mem::discriminant(&expected)
        );
    } else {
        panic!("type alias was unpatched");
    }
}
