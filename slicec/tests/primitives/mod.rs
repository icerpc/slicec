// Copyright (c) ZeroC, Inc.

use crate::test_helpers::*;
use slicec::grammar::*;
use test_case::test_case;

#[test_case("bool", Primitive::Bool; "bool")]
#[test_case("int8", Primitive::Int8; "int8")]
#[test_case("uint8", Primitive::UInt8; "uint8")]
#[test_case("int16", Primitive::Int16; "int16")]
#[test_case("uint16", Primitive::UInt16; "uint16")]
#[test_case("int32", Primitive::Int32; "int32")]
#[test_case("uint32", Primitive::UInt32; "uint32")]
#[test_case("varint32", Primitive::VarInt32; "varint32")]
#[test_case("varuint32", Primitive::VarUInt32; "varuint32")]
#[test_case("int64", Primitive::Int64; "int64")]
#[test_case("uint64", Primitive::UInt64; "uint64")]
#[test_case("varint62", Primitive::VarInt62; "varint62")]
#[test_case("varuint62", Primitive::VarUInt62; "varuint62")]
#[test_case("float32", Primitive::Float32; "float32")]
#[test_case("float64", Primitive::Float64; "float64")]
#[test_case("string", Primitive::String; "string")]
fn type_parses(slice_component: &str, expected: Primitive) {
    // Arrange
    let slice = format!(
        "
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
