use crate::code_block::CodeBlock;
use crate::code_map::CodeMap;
use crate::cs_util::*;
use crate::decoding::*;
use crate::encoding::*;
use slice::ast::Ast;
use slice::grammar::*;
use slice::util::*;
use slice::visitor::Visitor;
use std::iter::FromIterator;

#[derive(Debug)]
pub struct StructVisitor<'a> {
    pub code_map: &'a mut CodeMap,
}

impl<'a> Visitor for StructVisitor<'a> {
    fn visit_struct_start(&mut self, struct_def: &Struct, _: usize, ast: &Ast) {
        let readonly = struct_def.has_attribute("cs:readonly");

        let members = struct_def.members(ast);

        let constructor_parameters = members
            .iter()
            .map(|m| {
                format!(
                    "{} {}",
                    type_to_string(
                        &m.data_type,
                        struct_def.scope(),
                        ast,
                        TypeContext::DataMember
                    ),
                    m.identifier()
                )
            })
            .collect::<Vec<String>>();

        let mut constructor_body = CodeBlock::from_iter(members.iter().map(|m| {
            format!(
                "this.{} = {};",
                fix_case(m.identifier(), CaseStyle::Pascal),
                m.identifier()
            )
        }));

        let mut data_members: CodeBlock = members
            .iter()
            .map(|m| struct_data_member(m, ast))
            .collect::<Vec<_>>()
            .join("\n\n")
            .into();

        // TODO: this stuff from slice2cs
        // emitDeprecate(p, false, _out);
        // emitCommonAttributes();
        // emitCustomAttributes(p);

        let struct_code = format!(
            r#"
{access} partial struct {name} : global::System.IEquatable<{name}>
{{
    {data_members}

    /// <summary>Constructs a new instance of <see cref="{name}"/>.</summary>{doc_comment}
    public {name}({constructor_parameters})
    {{
        {constructor_body}
    }}

    /// <summary>Constructs a new instance of <see cref="{name}"/> from a decoder.</summary>
    public {name}(IceRpc.IceDecoder decoder)
    {{
        {decoder_body}
    }}

    ///<summary>Encodes the fields of this struct</summary>
    public readonly void Encode(IceRpc.IceEncoder encoder)
    {{
        {encoder_body}
    }}
}}"#,
            name = struct_def.identifier(),
            doc_comment = "", // TODO: get doc comment
            access = if readonly { "public readonly" } else { "public" },
            constructor_parameters = constructor_parameters.join(", "),
            constructor_body = constructor_body.indent().indent(),
            decoder_body = decode_data_members(&struct_def.members(ast), ast)
                .indent()
                .indent(),
            encoder_body = encode_data_members(&struct_def.members(ast), ast)
                .indent()
                .indent(),
            data_members = data_members.indent()
        );

        self.code_map.insert(struct_def, struct_code.into());
    }
}

fn struct_data_member(data_member: &Member, ast: &Ast) -> String {
    let type_string = type_to_string(
        &data_member.data_type,
        data_member.scope(),
        ast,
        TypeContext::DataMember,
    );

    format!(
        "\
{comment}
public {type_string} {name};",
        comment = "///TODO: comment",
        type_string = type_string,
        name = data_member.identifier()
    )
}
