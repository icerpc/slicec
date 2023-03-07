// Copyright (c) ZeroC, Inc.

use super::parser::Parser;
use crate::ast::node::Node;
use crate::diagnostics::{Error, ErrorKind};
use crate::grammar::*;
use crate::parsers::CommentParser;
use crate::slice_file::Span;
use crate::utils::ptr_util::{OwnedPtr, WeakPtr};
use crate::{downgrade_as, upcast_weak_as};

use std::num::IntErrorKind;
use std::ops::RangeInclusive;

use lalrpop_util::lalrpop_mod;

// Place the code generated by LALRPOP into a submodule named 'lalrpop'.
lalrpop_mod!(
    #[allow(unused, clippy::all)] // LALRPOP generates stuff we don't use, and isn't worth linting.
    pub lalrpop,
    "/parsers/slice/grammar.rs"
);

macro_rules! set_children_for {
    ($parent_ptr:expr, $children:ident, $parser:expr) => {{
        // 1. Set the parent on each of the children.
        // 2. Move the children into the AST.
        // 3. Store pointers to the children in the parent.
        for mut child in $children {
            unsafe {
                child.borrow_mut().parent = $parent_ptr.downgrade();
                let weak_ptr = $parser.ast.add_named_element(child);
                $parent_ptr.borrow_mut().$children.push(weak_ptr);
            }
        }
    }};
}

macro_rules! set_fields_for {
    ($parent_ptr:expr, $children:ident, $parser:expr) => {{
        // 1. Set the parent on each of the children.
        // 2. Move the children into the AST.
        // 3. Store pointers to the children in the parent.
        for mut child in $children {
            unsafe {
                child.borrow_mut().parent = downgrade_as!($parent_ptr, dyn Container<WeakPtr<Field>>);
                let weak_ptr = $parser.ast.add_named_element(child);
                $parent_ptr.borrow_mut().$children.push(weak_ptr);
            }
        }
    }};
}

// This macro does the following:
// 1. Set the module as the definition's parent.
// 2. Move the definition into the AST and keep a pointer to it.
// 3. Convert the pointer to a Definition and store it in the module.
macro_rules! add_definition_to_module {
    ($child:expr,Module, $module_ptr:expr, $parser:expr) => {{
        $child.borrow_mut().parent = Some($module_ptr.downgrade());
        let weak_ptr = $parser.ast.add_named_element($child);
        $module_ptr.borrow_mut().contents.push(Definition::Module(weak_ptr));
    }};
    ($child:expr, $node_type:ident, $module_ptr:expr, $parser:expr) => {{
        $child.borrow_mut().parent = $module_ptr.downgrade();
        let weak_ptr = $parser.ast.add_named_element($child);
        $module_ptr.borrow_mut().contents.push(Definition::$node_type(weak_ptr));
    }};
}

// Convenience type for storing an unparsed doc comment. Each element of the vector is one line of the comment.
type RawDocComment<'a> = Vec<(&'a str, Span)>;

// Grammar Rule Functions

fn handle_file_encoding(
    parser: &mut Parser,
    (old_encoding, attributes): (Option<FileEncoding>, Vec<Attribute>),
    encoding: FileEncoding,
) -> (Option<FileEncoding>, Vec<Attribute>) {
    // The file encoding can only be set once.
    if let Some(old_file_encoding) = old_encoding {
        let old_span = old_file_encoding.span();
        Error::new(ErrorKind::MultipleEncodingVersions)
            .set_span(old_span)
            .add_note("file encoding was previously specified here", Some(old_span))
            .report(parser.diagnostic_reporter);
    }
    parser.file_encoding = encoding.version;
    (Some(encoding), attributes)
}

fn construct_file_encoding(parser: &mut Parser, i: Integer<i128>, span: Span) -> FileEncoding {
    let version = match i.value {
        1 => Encoding::Slice1,
        2 => Encoding::Slice2,
        v => {
            Error::new(ErrorKind::InvalidEncodingVersion { encoding: v })
                .set_span(&i.span)
                .add_note("must be '1' or '2'", None)
                .report(parser.diagnostic_reporter);
            Encoding::default() // Dummy
        }
    };
    FileEncoding { version, span }
}

fn construct_module(
    parser: &mut Parser,
    (raw_comment, attributes): (RawDocComment, Vec<Attribute>),
    identifier: Identifier,
    definitions: Vec<Node>,
    is_file_scoped: bool,
    span: Span,
) -> OwnedPtr<Module> {
    // If nested module syntax is being used, get the last module's identifier, otherwise use the whole identifier.
    let last_identifier = identifier.value.rsplit("::").next().unwrap_or(&identifier.value);
    let comment = parse_doc_comment(parser, last_identifier, raw_comment);

    // In case nested module syntax was used, we split the identifier on '::' and construct a module for each segment.
    // We use `rsplit` to iterate in reverse order (right to left) to construct them in child-to-parent order.
    // Ex: `Foo::Bar::Baz`: first create `Baz` to add the definitions in, then `Bar` to add `Baz` to it, etc...
    let mut modules = identifier.value.rsplit("::").map(|i| {
        // Pop the module's scope off the scope stack and construct it (otherwise it would be in its own scope).
        parser.current_scope.pop_scope();
        OwnedPtr::new(Module {
            identifier: Identifier {
                value: i.to_owned(),
                span: span.clone(),
            },
            contents: Vec::new(),
            is_file_scoped: false,
            parent: None,
            scope: parser.current_scope.clone(),
            attributes: Vec::new(),
            comment: None,
            span: span.clone(),
        })
    });

    // It's safe to unwrap because if the parser called this function, at least one module must have been constructed.
    // Since we're iterating in reverse order, this will return the inner-most module.
    // If nested module syntax wasn't used, this is just the singular module.
    let mut current_module = modules.next().unwrap();

    unsafe {
        // Any attributes, comments, or definitions belong to the innermost module, stored as `current_module`.
        // We re-borrow it every time we set a field to make ensure that the borrows are dropped immediately.
        current_module.borrow_mut().is_file_scoped = is_file_scoped;
        current_module.borrow_mut().attributes = attributes;
        current_module.borrow_mut().comment = comment;
        for definition in definitions {
            match definition {
                Node::Module(mut x) => add_definition_to_module!(x, Module, current_module, parser),
                Node::Struct(mut x) => add_definition_to_module!(x, Struct, current_module, parser),
                Node::Exception(mut x) => add_definition_to_module!(x, Exception, current_module, parser),
                Node::Class(mut x) => add_definition_to_module!(x, Class, current_module, parser),
                Node::Interface(mut x) => add_definition_to_module!(x, Interface, current_module, parser),
                Node::Enum(mut x) => add_definition_to_module!(x, Enum, current_module, parser),
                Node::CustomType(mut x) => add_definition_to_module!(x, CustomType, current_module, parser),
                Node::TypeAlias(mut x) => add_definition_to_module!(x, TypeAlias, current_module, parser),
                _ => panic!("impossible definition type encountered: {definition:?}"),
            }
        }

        // Work up the nested module syntax, storing each module in its parent until we reach the outer-most module.
        for mut parent_module in modules {
            add_definition_to_module!(current_module, Module, parent_module, parser);
            current_module = parent_module;
        }
    }

    // Return the outer-most module.
    current_module
}

fn construct_struct(
    parser: &mut Parser,
    (raw_comment, attributes): (RawDocComment, Vec<Attribute>),
    is_compact: bool,
    identifier: Identifier,
    fields: Vec<OwnedPtr<Field>>,
    span: Span,
) -> OwnedPtr<Struct> {
    let comment = parse_doc_comment(parser, &identifier.value, raw_comment);
    let mut struct_ptr = OwnedPtr::new(Struct {
        identifier,
        fields: Vec::new(),
        is_compact,
        parent: WeakPtr::create_uninitialized(), // Patched by its container.
        scope: parser.current_scope.clone(),
        attributes,
        comment,
        span,
        supported_encodings: None, // Patched by the encoding patcher.
    });

    // Add all the fields to the struct.
    set_fields_for!(struct_ptr, fields, parser);

    struct_ptr
}

fn construct_exception(
    parser: &mut Parser,
    (raw_comment, attributes): (RawDocComment, Vec<Attribute>),
    identifier: Identifier,
    base_type: Option<TypeRef>,
    fields: Vec<OwnedPtr<Field>>,
    span: Span,
) -> OwnedPtr<Exception> {
    let base = base_type.map(|type_ref| type_ref.downcast::<Exception>().unwrap());
    let comment = parse_doc_comment(parser, &identifier.value, raw_comment);

    let mut exception_ptr = OwnedPtr::new(Exception {
        identifier,
        fields: Vec::new(),
        base,
        parent: WeakPtr::create_uninitialized(), // Patched by its container.
        scope: parser.current_scope.clone(),
        attributes,
        comment,
        span,
        supported_encodings: None, // Patched by the encoding patcher.
    });

    // Add all the fields to the exception.
    set_fields_for!(exception_ptr, fields, parser);

    exception_ptr
}

fn construct_class(
    parser: &mut Parser,
    (raw_comment, attributes): (RawDocComment, Vec<Attribute>),
    identifier: Identifier,
    compact_id: Option<Integer<u32>>,
    base_type: Option<TypeRef>,
    fields: Vec<OwnedPtr<Field>>,
    span: Span,
) -> OwnedPtr<Class> {
    let base = base_type.map(|type_ref| type_ref.downcast::<Class>().unwrap());
    let comment = parse_doc_comment(parser, &identifier.value, raw_comment);

    let mut class_ptr = OwnedPtr::new(Class {
        identifier,
        fields: Vec::new(),
        compact_id,
        base,
        parent: WeakPtr::create_uninitialized(), // Patched by its container.
        scope: parser.current_scope.clone(),
        attributes,
        comment,
        span,
        supported_encodings: None, // Patched by the encoding patcher.
    });

    // Add all the fields to the class.
    set_fields_for!(class_ptr, fields, parser);

    class_ptr
}

pub fn construct_field(
    parser: &mut Parser,
    (raw_comment, attributes): (RawDocComment, Vec<Attribute>),
    identifier: Identifier,
    tag: Option<Integer<u32>>,
    data_type: TypeRef,
    span: Span,
) -> OwnedPtr<Field> {
    let comment = parse_doc_comment(parser, &identifier.value, raw_comment);
    OwnedPtr::new(Field {
        identifier,
        data_type,
        tag,
        parent: WeakPtr::create_uninitialized(), // Patched by its container.
        scope: parser.current_scope.clone(),
        attributes,
        comment,
        span,
    })
}

fn construct_interface(
    parser: &mut Parser,
    (raw_comment, attributes): (RawDocComment, Vec<Attribute>),
    identifier: Identifier,
    bases: Option<Vec<TypeRef>>,
    operations: Vec<OwnedPtr<Operation>>,
    span: Span,
) -> OwnedPtr<Interface> {
    let bases = bases
        .unwrap_or_default() // Create an empty vector if no bases were specified.
        .into_iter()
        .map(|base| base.downcast::<Interface>().unwrap())
        .collect::<Vec<_>>();
    let comment = parse_doc_comment(parser, &identifier.value, raw_comment);

    let mut interface_ptr = OwnedPtr::new(Interface {
        identifier,
        operations: Vec::new(),
        bases,
        parent: WeakPtr::create_uninitialized(), // Patched by its container.
        scope: parser.current_scope.clone(),
        attributes,
        comment,
        span,
        supported_encodings: None, // Patched by the encoding patcher.
    });

    // Add all the operations to the interface.
    set_children_for!(interface_ptr, operations, parser);

    interface_ptr
}

#[allow(clippy::too_many_arguments)]
fn construct_operation(
    parser: &mut Parser,
    (raw_comment, attributes): (RawDocComment, Vec<Attribute>),
    is_idempotent: bool,
    identifier: Identifier,
    parameters: Vec<OwnedPtr<Parameter>>,
    return_type: Option<Vec<OwnedPtr<Parameter>>>,
    exception_specification: Option<Throws>,
    span: Span,
) -> OwnedPtr<Operation> {
    // If no return type was provided set the return type to an empty Vec.
    let mut return_type = return_type.unwrap_or_default();

    // If no throws clause was present, set the exception specification to `None`.
    let throws = exception_specification.unwrap_or(Throws::None);

    let comment = parse_doc_comment(parser, &identifier.value, raw_comment);

    let mut operation_ptr = OwnedPtr::new(Operation {
        identifier,
        parameters: Vec::new(),
        return_type: Vec::new(),
        throws,
        is_idempotent,
        encoding: parser.file_encoding,
        parent: WeakPtr::create_uninitialized(), // Patched by its container.
        scope: parser.current_scope.clone(),
        attributes,
        comment,
        span,
    });

    // Fix the return members to have `is_returned` set to true.
    for parameter in &mut return_type {
        unsafe {
            parameter.borrow_mut().is_returned = true;
        }
    }

    // Add all the parameters and return members to the operation.
    set_children_for!(operation_ptr, parameters, parser);
    set_children_for!(operation_ptr, return_type, parser);

    operation_ptr
}

fn construct_parameter(
    parser: &mut Parser,
    (raw_comment, attributes): (RawDocComment, Vec<Attribute>),
    identifier: Identifier,
    (is_streamed, tag): (bool, Option<Integer<u32>>),
    data_type: TypeRef,
    span: Span,
) -> OwnedPtr<Parameter> {
    let comment = parse_doc_comment(parser, &identifier.value, raw_comment);
    OwnedPtr::new(Parameter {
        identifier,
        data_type,
        tag,
        is_streamed,
        is_returned: false,                      // Patched by its operation.
        parent: WeakPtr::create_uninitialized(), // Patched by its container.
        scope: parser.current_scope.clone(),
        attributes,
        comment,
        span,
    })
}

fn construct_single_return_type(
    parser: &Parser,
    (is_streamed, tag): (bool, Option<Integer<u32>>),
    data_type: TypeRef,
    span: Span,
) -> Vec<OwnedPtr<Parameter>> {
    // Create a dummy identifier for the return type, since it's nameless.
    let dummy_identifier = Identifier {
        value: "returnValue".to_owned(),
        span: span.clone(),
    };

    vec![OwnedPtr::new(Parameter {
        identifier: dummy_identifier,
        data_type,
        tag,
        is_streamed,
        is_returned: false,                      // Patched by its operation.
        parent: WeakPtr::create_uninitialized(), // Patched by its container.
        scope: parser.current_scope.clone(),
        attributes: Vec::new(),
        comment: None,
        span,
    })]
}

fn check_return_tuple(parser: &mut Parser, return_tuple: &Vec<OwnedPtr<Parameter>>, span: Span) {
    if return_tuple.len() < 2 {
        Error::new(ErrorKind::ReturnTuplesMustContainAtLeastTwoElements)
            .set_span(&span)
            .report(parser.diagnostic_reporter)
    }
}

fn construct_enum(
    parser: &mut Parser,
    (raw_comment, attributes): (RawDocComment, Vec<Attribute>),
    is_unchecked: bool,
    identifier: Identifier,
    underlying_type: Option<TypeRef>,
    enumerators: Vec<OwnedPtr<Enumerator>>,
    span: Span,
) -> OwnedPtr<Enum> {
    let underlying = underlying_type.map(|type_ref| type_ref.downcast::<Primitive>().unwrap());
    let comment = parse_doc_comment(parser, &identifier.value, raw_comment);

    let mut enum_ptr = OwnedPtr::new(Enum {
        identifier,
        enumerators: Vec::new(),
        underlying,
        is_unchecked,
        parent: WeakPtr::create_uninitialized(), // Patched by its container.
        scope: parser.current_scope.clone(),
        attributes,
        comment,
        span,
        supported_encodings: None, // Patched by the encoding patcher.
    });

    // Add all the enumerators to the enum.
    set_children_for!(enum_ptr, enumerators, parser);

    // Clear the `last_enumerator_value` field since this is the end of the enum.
    parser.last_enumerator_value = None;

    enum_ptr
}

fn construct_enumerator(
    parser: &mut Parser,
    (raw_comment, attributes): (RawDocComment, Vec<Attribute>),
    identifier: Identifier,
    enumerator_value: Option<Integer<i128>>,
    span: Span,
) -> OwnedPtr<Enumerator> {
    let comment = parse_doc_comment(parser, &identifier.value, raw_comment);

    // If the enumerator was given an explicit value, use it. Otherwise it's implicit value is calculated as follows:
    // If this is the first enumerator in the enum its value is 0 (since `last_enumerator_value` is `None`).
    // For any other enumerator, its value is equal to the previous enumerator's value plus 1.
    let value = match enumerator_value {
        Some(integer) => EnumeratorValue::Explicit(integer),
        None => EnumeratorValue::Implicit(parser.last_enumerator_value.map_or(0, |x| x.wrapping_add(1))),
    };

    let enumerator = OwnedPtr::new(Enumerator {
        identifier,
        value,
        parent: WeakPtr::create_uninitialized(), // Patched by its container.
        scope: parser.current_scope.clone(),
        attributes,
        comment,
        span,
    });

    parser.last_enumerator_value = Some(enumerator.borrow().value());
    enumerator
}

fn construct_custom_type(
    parser: &mut Parser,
    (raw_comment, attributes): (RawDocComment, Vec<Attribute>),
    identifier: Identifier,
    span: Span,
) -> OwnedPtr<CustomType> {
    let comment = parse_doc_comment(parser, &identifier.value, raw_comment);
    OwnedPtr::new(CustomType {
        identifier,
        parent: WeakPtr::create_uninitialized(), // Patched by its container.
        scope: parser.current_scope.clone(),
        attributes,
        comment,
        span,
        supported_encodings: None, // Patched by the encoding patcher.
    })
}

fn construct_type_alias(
    parser: &mut Parser,
    (raw_comment, attributes): (RawDocComment, Vec<Attribute>),
    identifier: Identifier,
    underlying: TypeRef,
    span: Span,
) -> OwnedPtr<TypeAlias> {
    let comment = parse_doc_comment(parser, &identifier.value, raw_comment);
    OwnedPtr::new(TypeAlias {
        identifier,
        underlying,
        parent: WeakPtr::create_uninitialized(), // Patched by its container.
        scope: parser.current_scope.clone(),
        attributes,
        comment,
        span,
    })
}

fn construct_type_ref(
    parser: &Parser,
    attributes: Vec<Attribute>,
    definition: TypeRefDefinition,
    is_optional: bool,
    span: Span,
) -> TypeRef {
    TypeRef {
        definition,
        is_optional,
        scope: parser.current_scope.clone(),
        attributes,
        span,
    }
}

fn primitive_to_type_ref_definition(parser: &Parser, primitive: Primitive) -> TypeRefDefinition {
    // These unwraps are safe because the primitive types are always defined in the AST.
    let node = parser.ast.find_node(primitive.kind()).unwrap();
    let weak_ptr: WeakPtr<Primitive> = node.try_into().unwrap();
    TypeRefDefinition::Patched(upcast_weak_as!(weak_ptr, dyn Type))
}

fn anonymous_type_to_type_ref_definition<T>(parser: &mut Parser, ptr: OwnedPtr<T>) -> TypeRefDefinition
where
    T: Type + 'static,
    OwnedPtr<T>: Into<Node>,
{
    let weak_ptr = parser.ast.add_element(ptr);
    TypeRefDefinition::Patched(upcast_weak_as!(weak_ptr, dyn Type))
}

fn construct_unpatched_type_ref_definition(mut identifier: Identifier) -> TypeRefDefinition {
    // Remove any whitespace from the identifier so it can be looked up in the AST.
    identifier.value.retain(|c| !c.is_whitespace());
    TypeRefDefinition::Unpatched(identifier.value)
}

fn try_construct_attribute(
    parser: &mut Parser,
    directive: Identifier,
    arguments: Option<Vec<String>>,
    span: Span,
) -> Attribute {
    Attribute::new(
        parser.diagnostic_reporter,
        &directive.value,
        arguments.unwrap_or_default(),
        span,
    )
}

fn try_parse_integer(parser: &mut Parser, s: &str, span: Span) -> Integer<i128> {
    // Check the literal for a base prefix. If present, remove it and set the base.
    // "0b" = binary, "0x" = hexadecimal, otherwise we assume it's decimal.
    let (literal, base) = match s {
        _ if s.starts_with("0b") => (&s[2..], 2),
        _ if s.starts_with("0x") => (&s[2..], 16),
        _ => (s, 10),
    };

    let value = match i128::from_str_radix(literal, base) {
        Ok(x) => x,
        Err(err) => {
            let error = match err.kind() {
                IntErrorKind::InvalidDigit => ErrorKind::InvalidIntegerLiteral { base },
                _ => ErrorKind::IntegerLiteralOverflows,
            };
            Error::new(error).set_span(&span).report(parser.diagnostic_reporter);
            0 // Dummy value
        }
    };

    Integer { value, span }
}

fn parse_tag_value(parser: &mut Parser, i: Integer<i128>, span: Span) -> Integer<u32> {
    // Verify that the provided integer is a valid tag id.
    if !RangeInclusive::new(0, i32::MAX as i128).contains(&i.value) {
        Error::new(ErrorKind::TagValueOutOfBounds)
            .set_span(&span)
            .report(parser.diagnostic_reporter)
    }

    // Cast the integer to a `u32` since it most closely matches the allowed range of tags.
    // It's fine if the value doesn't fit, the cast will just give us a dummy value.
    let value = i.value as u32;
    Integer { value, span: i.span }
}

fn parse_compact_id_value(parser: &mut Parser, i: Integer<i128>, span: Span) -> Integer<u32> {
    // Verify that the provided integer is a valid compact id.
    if !RangeInclusive::new(0, i32::MAX as i128).contains(&i.value) {
        Error::new(ErrorKind::CompactIdOutOfBounds)
            .set_span(&span)
            .report(parser.diagnostic_reporter)
    }

    // Cast the integer to a `u32` since it most closely matches the allowed range of compact ids.
    // It's fine if the value doesn't fit, the cast will just give us a dummy value.
    let value = i.value as u32;
    Integer { value, span: i.span }
}

fn parse_doc_comment(parser: &mut Parser, identifier: &str, raw_comment: RawDocComment) -> Option<DocComment> {
    if raw_comment.is_empty() {
        // If the doc comment had 0 lines, that just means there is no doc comment.
        None
    } else {
        let scoped_identifier = parser.current_scope.raw_parser_scope.to_owned() + "::" + identifier;
        let mut comment_parser = CommentParser::new(parser.file_name, &scoped_identifier, parser.diagnostic_reporter);
        // Any errors are reported to the `diagnostic_reporter`, so it's okay to `ok` the result here.
        comment_parser.parse_doc_comment(raw_comment).ok()
    }
}
