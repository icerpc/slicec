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
    let underlying = &ast.find_symbol_by_id::<TypeAlias>("Test::P").unwrap().underlying;
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

#[test_case(Primitive::Bool; "bool")]
#[test_case(Primitive::Int8; "int8")]
#[test_case(Primitive::UInt8; "uint8")]
#[test_case(Primitive::Int16; "int16")]
#[test_case(Primitive::UInt16; "uint16")]
#[test_case(Primitive::Int32; "int32")]
#[test_case(Primitive::UInt32; "uint32")]
#[test_case(Primitive::VarInt32; "varint32")]
#[test_case(Primitive::VarUInt32; "varuint32")]
#[test_case(Primitive::Int64; "int64")]
#[test_case(Primitive::UInt64; "uint64")]
#[test_case(Primitive::VarInt62; "varint62")]
#[test_case(Primitive::VarUInt62; "varuint62")]
#[test_case(Primitive::Float32; "float32")]
#[test_case(Primitive::Float64; "float64")]
#[test_case(Primitive::String; "string")]
fn find_primitive_node_returns_the_correct_node(primitive: Primitive) {
    // `find_primitive_node` indexes into the AST's elements by the primitive's discriminant,
    // which relies on the ordering of the `Primitive` enum. This test ensures this ordering is consistent.

    // Arrange
    let ast = slicec::ast::Ast::create();
    let expected_kind = primitive.kind();

    // Act
    let element: &dyn Element = ast.find_primitive_node(primitive).into();

    // Assert
    assert_eq!(element.kind(), expected_kind);
}
