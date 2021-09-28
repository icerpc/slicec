#[derive(Debug)]
pub struct StructVisitor<'a> {
    pub code_map: &'a mut CodeMap,
}

impl<'a> Visitor for EnumVisitor<'a> {
    fn visit_enum_start(&mut self, enum_def: &Enum, _: usize, ast: &Ast) {
        let code = format(
            "\
{declaration}

{helper}",
            declaration = enum_declaration(enum_def, ast),
            helper = enum_helper(enum_def, ast),
        );

        self.code_map.insert(enum_def, code);
    }
}

fn enum_declaration(enum_def: &Enum, ast: &Ast) -> CodeBlock {
    // write_comment(&mut self.output, enum_def);
    // TODO: from slice2cs
    // writeTypeDocComment(p, getDeprecateReason(p));
    // emitCommonAttributes();
    // emitCustomAttributes(p);

    let mut code = CodeBlock::new();
    write!(
        code,
        r#"
public enum {name} : {underlying_type}
{{
    {{enum_values}}
}}
"#,
        ame = enum_def.identifier(),
        underlying_type = underlying_type,
        enum_values = enum_values(enum_def, ast).indent()
    );

    code
}

fn enum_values(enum_def: &Enum, ast: &Ast) -> CodeBlock {
    let code = CodeBlock::new();
    for enumerator in enum_def.enumerators(ast) {
        let comment = "//TODO: get comment\n";
        code.add_block(
            format!(
                "{}{} = {}",
                comment,
                enumerator.identifier(),
                enumerator.value
            )
            .into(),
        );
    }
    code
}

fn enum_helper(enum_def: &Enum, ast: &Ast) -> CodeBlock {
    let escaped_identifier = escape_identifier(enum_def, CaseStyle::Pascal);

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

    let hash_set = if use_set {
        format!(
            "\
\npublic static readonly global::System.Collections.Generic.HashSet<{underlying}> EnumeratorValues =
    new global::System.Collections.Generic.HashSet<{underlying}> {{ {enum_values} }};",
            underlying = underlying_type,
            enum_values = enum_def
                .enumerators(ast)
                .iter()
                .map(|e| e.value.to_string())
                .collect::<Vec<String>>()
                .join(",")
        )
    } else {
        "".to_owned()
    };

    let as_enum = if enum_def.is_unchecked {
        format!("({})value", escaped_identifier)
    } else {
        let check_enum = if use_set {
            "EnumeratorValues.Contains(value)".to_owned()
        } else {
            // TODO: get the actual min and max values
            format!(
                "{min_value} <= value && value <= {max_value}",
                min_value = "min",
                max_value = "max"
            )
        };

        format!(
                "{check_enum} ? ({escaped_identifier})value : throw new IceRpc.InvalidDataException($\"invalid enumerator value '{{value}}' for {scoped}\")",
                check_enum = check_enum,
                escaped_identifier = escaped_identifier,
                scoped = escape_scoped_identifier(enum_def, CaseStyle::Pascal, ""),
            )
    };

    // Enum decoding
    let decode_enum = format!(
        "As{name}(decoder.{decode_method})",
        name = enum_def.identifier(),
        decode_method = if let Some(underlying) = &enum_def.underlying {
            format!("Decode{}()", builtin_suffix(underlying.definition(ast)))
        } else {
            "DecodeSize()".to_owned()
        }
    );

    // Enum encoding
    let encode_enum = if let Some(underlying) = &enum_def.underlying {
        format!(
            "encoder.Encode{}",
            builtin_suffix(underlying.definition(ast))
        )
    } else {
        "encoder.EncodeSize((int)value)".to_owned()
    };

    // Enum helper class
    format!(
        r#"
/// <summary>Helper class for marshaling and unmarshaling <see cref="{escaped_identifier}"/>.</summary>
public static class {identifier}Helper
{{{hash_set}

    public static {escaped_identifier} As{identifier}(this {underlying_type} value) =>
        {as_enum};

    public static {escaped_identifier} Decode{identifier} (this IceRpc.IceDecoder decoder) =>
        {decode_enum};

    public static void Encode{identifier} (this IceRpc.IceEncoder encoder, {escaped_identifier} value) =>
        {encode_enum};
}}"#,
        escaped_identifier = escaped_identifier,
        identifier = enum_def.identifier(),
        underlying_type = underlying_type,
        hash_set = hash_set.replace("\n", "\n    "),
        as_enum = as_enum.replace("\n", "\n    "),
        decode_enum = decode_enum,
        encode_enum = encode_enum
    ).into()
}

fn underlying_type(enum_def: &Enum, ast: &Ast) -> String {
    if let Some(typeref) = &enum_def.underlying {
        type_to_string(
            typeref,
            enum_def.scope.as_ref().unwrap(),
            ast,
            TypeContext::Nested,
        )
    } else {
        "int".to_owned() // TODO we should make a builtin table to get names from.
    }
}
