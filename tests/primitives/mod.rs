// Copyright (c) ZeroC, Inc.

mod encoding;

use slice::grammar::*;
use slice::test_helpers::parse_for_ast;
use test_case::test_case;

#[test_case("bool", Primitive::Bool, 2; "bool")]
#[test_case("int8", Primitive::Int8, 2; "int8")]
#[test_case("uint8", Primitive::UInt8, 2; "uint8")]
#[test_case("int16", Primitive::Int16, 2; "int16")]
#[test_case("uint16", Primitive::UInt16, 2; "uint16")]
#[test_case("int32", Primitive::Int32, 2; "int32")]
#[test_case("uint32", Primitive::UInt32, 2; "uint32")]
#[test_case("varint32", Primitive::VarInt32, 2; "varint32")]
#[test_case("varuint32", Primitive::VarUInt32, 2; "varuint32")]
#[test_case("int64", Primitive::Int64, 2; "int64")]
#[test_case("uint64", Primitive::UInt64, 2; "uint64")]
#[test_case("varint62", Primitive::VarInt62, 2; "varint62")]
#[test_case("varuint62", Primitive::VarUInt62, 2; "varuint62")]
#[test_case("float32", Primitive::Float32, 2; "float32")]
#[test_case("float64", Primitive::Float64, 2; "float64")]
#[test_case("string", Primitive::String, 2; "string")]
#[test_case("AnyClass", Primitive::AnyClass, 1; "AnyClass")]
fn type_parses(slice_component: &str, expected: Primitive, encoding: u8) {
    // Arrange
    let slice = format!(
        "
            encoding = Slice{encoding}
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
