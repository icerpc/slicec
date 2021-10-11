use crate::builders::ContainerBuilder;
use crate::code_block::CodeBlock;
use crate::code_map::CodeMap;
use crate::cs_util::*;
use slice::ast::Ast;
use slice::grammar::*;
use slice::util::*;
use slice::visitor::Visitor;

#[derive(Debug)]
pub struct EnumVisitor<'a> {
    pub code_map: &'a mut CodeMap,
}

impl<'a> Visitor for EnumVisitor<'a> {
    fn visit_enum_start(&mut self, enum_def: &Enum, _: usize, ast: &Ast) {
        let code = format!(
            "\
{declaration}

{helper}",
            declaration = enum_declaration(enum_def, ast),
            helper = enum_helper(enum_def, ast),
        );

        self.code_map.insert(enum_def, code.into());
    }
}

fn enum_declaration(enum_def: &Enum, ast: &Ast) -> CodeBlock {
    // write_comment(&mut self.output, enum_def);
    // TODO: from slice2cs
    // writeTypeDocComment(p, getDeprecateReason(p));
    // emitCommonAttributes();
    // emitCustomAttributes(p);
    let escaped_identifier = escape_keyword(enum_def.identifier());
    let mut builder = ContainerBuilder::new("public enum", &escaped_identifier);
    // TODO add comment
    builder.add_block(enum_values(enum_def, ast));
    builder.build().into()
}

fn enum_values(enum_def: &Enum, ast: &Ast) -> CodeBlock {
    let mut code = CodeBlock::new();
    for enumerator in enum_def.enumerators(ast) {
        // TODO add comment
        code.add_block(&format!("{} = {};", enumerator.identifier(), enumerator.value).into());
    }
    code
}

fn enum_helper(enum_def: &Enum, ast: &Ast) -> CodeBlock {
    let escaped_identifier = escape_keyword(enum_def.identifier());
    let ns = get_namespace(enum_def);
    let mut builder = ContainerBuilder::new("public static class", &helper_name(enum_def, &ns));

    builder.add_comment(
        "summary",
        &format!(
            r#"Helper class for marshaling and unmarshaling <see cref="{}"/>."#,
            escaped_identifier
        ),
    );

    // When the number of enumerators is smaller than the distance between the min and max
    // values, the values are not consecutive and we need to use a set to validate the value
    // during unmarshaling.
    // Note that the values are not necessarily in order, e.g. we can use a simple range check
    // for enum E { A = 3, B = 2, C = 1 } during unmarshaling.
    let use_set = if let (Some(min_value), Some(max_value)) =
        (enum_def.min_value(ast), enum_def.max_value(ast))
    {
        !enum_def.is_unchecked && (enum_def.enumerators.len() as i64) < max_value - min_value + 1
    } else {
        // This means there are no enumerators.*
        true
    };

    let underlying_type = underlying_type(enum_def, ast);

    if use_set {
        builder.add_block(
            format!(
                "\
public static readonly global::System.Collections.Generic.HashSet<{underlying}> EnumeratorValues =
    new global::System.Collections.Generic.HashSet<{underlying}> {{ {enum_values} }};",
                underlying = underlying_type,
                enum_values = enum_def
                    .enumerators(ast)
                    .iter()
                    .map(|e| e.value.to_string())
                    .collect::<Vec<String>>()
                    .join(", ")
            )
            .into(),
        );
    }

    let mut as_enum: CodeBlock = if enum_def.is_unchecked {
        format!("({})value", escaped_identifier).into()
    } else {
        format!(
            r#"
{check_enum} ?
    ({escaped_identifier})value :
    throw new IceRpc.InvalidDataException($"invalid enumerator value '{{value}}' for {scoped}")"#,
            check_enum = match use_set {
                true => "EnumeratorValues.Contains(value)".to_owned(),
                false => format!(
                    "{min_value} <= value && value <= {max_value}",
                    min_value = enum_def.min_value(ast).unwrap(),
                    max_value = enum_def.max_value(ast).unwrap()
                ),
            },
            escaped_identifier = escaped_identifier,
            scoped = escape_scoped_identifier(enum_def, CaseStyle::Pascal, &ns),
        )
        .into()
    };

    builder.add_block(
        format!(
            r#"
public static {escaped_identifier} As{identifier}(this {underlying_type} value) =>
    {as_enum};"#,
            identifier = enum_def.identifier(),
            escaped_identifier = escaped_identifier,
            underlying_type = underlying_type,
            as_enum = as_enum.indent()
        )
        .into(),
    );

    // Enum decoding
    builder.add_block(
        format!(
            r#"
public static {escaped_identifier} Decode{identifier}(this IceRpc.IceDecoder decoder) =>
    As{identifier}({decode_enum});"#,
            identifier = enum_def.identifier(),
            escaped_identifier = escaped_identifier,
            decode_enum = match &enum_def.underlying {
                Some(underlying) =>
                    format!("Decode{}()", builtin_suffix(underlying.definition(ast))),
                _ => "DecodeSize()".to_owned(),
            }
        )
        .into(),
    );

    // Enum encoding
    builder.add_block(
        format!(
            r#"
public static void Encode{identifier}(this IceRpc.IceEncoder encoder, {escaped_identifier} value) =>
    {encode_enum}"#,
            identifier = enum_def.identifier(),
            escaped_identifier = escaped_identifier,
            encode_enum = match &enum_def.underlying {
                Some(underlying) => format!(
                    "encoder.Encode{}",
                    builtin_suffix(underlying.definition(ast))
                ),
                None => "encoder.EncodeSize((int)value)".to_owned(),
            }
        )
        .into(),
    );

    builder.build().into()
}

fn underlying_type(enum_def: &Enum, ast: &Ast) -> String {
    match &enum_def.underlying {
        Some(typeref) => type_to_string(
            typeref,
            enum_def.scope.as_ref().unwrap(),
            ast,
            TypeContext::Nested,
        ),
        _ => "int".to_owned(), // TODO we should make a builtin table to get names from.
    }
}
