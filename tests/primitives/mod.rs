// Copyright (c) ZeroC, Inc.

mod mode;

use crate::test_helpers::*;
use slicec::grammar::*;
use test_case::test_case;

#[test_case("bool", Primitive::Bool, "Slice2"; "bool")]
#[test_case("int8", Primitive::Int8, "Slice2"; "int8")]
#[test_case("uint8", Primitive::UInt8, "Slice2"; "uint8")]
#[test_case("int16", Primitive::Int16, "Slice2"; "int16")]
#[test_case("uint16", Primitive::UInt16, "Slice2"; "uint16")]
#[test_case("int32", Primitive::Int32, "Slice2"; "int32")]
#[test_case("uint32", Primitive::UInt32, "Slice2"; "uint32")]
#[test_case("varint32", Primitive::VarInt32, "Slice2"; "varint32")]
#[test_case("varuint32", Primitive::VarUInt32, "Slice2"; "varuint32")]
#[test_case("int64", Primitive::Int64, "Slice2"; "int64")]
#[test_case("uint64", Primitive::UInt64, "Slice2"; "uint64")]
#[test_case("varint62", Primitive::VarInt62, "Slice2"; "varint62")]
#[test_case("varuint62", Primitive::VarUInt62, "Slice2"; "varuint62")]
#[test_case("float32", Primitive::Float32, "Slice2"; "float32")]
#[test_case("float64", Primitive::Float64, "Slice2"; "float64")]
#[test_case("string", Primitive::String, "Slice2"; "string")]
#[test_case("AnyClass", Primitive::AnyClass, "Slice1"; "AnyClass")]
fn type_parses(slice_component: &str, expected: Primitive, mode: &str) {
    // Arrange
    let slice = format!(
        "
            mode = {mode}
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
