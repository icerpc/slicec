use crate::builders::{ContainerBuilder, FunctionBuilder};
use crate::code_block::CodeBlock;
use crate::code_map::CodeMap;
use crate::cs_util::*;
use crate::decoding::*;
use crate::encoding::*;
use slice::ast::Ast;
use slice::grammar::*;
use slice::util::*;
use slice::visitor::Visitor;

#[derive(Debug)]
pub struct StructVisitor<'a> {
    pub code_map: &'a mut CodeMap,
}

impl<'a> Visitor for StructVisitor<'a> {
    fn visit_struct_start(&mut self, struct_def: &Struct, _: usize, ast: &Ast) {
        let readonly = struct_def.has_attribute("cs:readonly");
        let escaped_identifier = escape_keyword(struct_def.identifier());
        let members = struct_def.members(ast);
        let namespace = get_namespace(struct_def);

        let mut builder = ContainerBuilder::new(
            &format!(
                "{access} partial record struct",
                access = if readonly { "public readonly" } else { "public" },
            ),
            &escaped_identifier,
        );

        // TODO: add deprecate
        // emitDeprecate(p, false, _out);

        builder.add_custom_attributes(struct_def);

        builder.add_block(
            members
                .iter()
                .map(|m| data_member_declaration(m, readonly, FieldType::NonMangled, ast))
                .collect::<Vec<_>>()
                .join("\n\n")
                .into(),
        );

        let mut main_constructor = FunctionBuilder::new("public", "", &escaped_identifier);
        main_constructor.add_comment(
            "summary",
            &format!(
                r#"Constructs a new instance of <see cref="{}"/>."#,
                &escaped_identifier
            ),
        );

        for member in &members {
            main_constructor.add_parameter(
                &type_to_string(&member.data_type, &namespace, ast, TypeContext::DataMember),
                member.identifier(),
                None,
                "", // TODO add parameter comment
            );
        }
        main_constructor.set_body({
            let mut code = CodeBlock::new();
            for member in &members {
                writeln!(
                    code,
                    "this.{} = {};",
                    field_name(*member, FieldType::NonMangled),
                    escape_identifier(*member, CaseStyle::Camel),
                );
            }
            code
        });
        builder.add_block(main_constructor.build());

        // Decode constructor
        builder.add_block(
            FunctionBuilder::new("public", "", &escaped_identifier)
                .add_comment(
                    "summary",
                    &format!(
                        r#"Constructs a new instance of <see cref="{}"/> from a decoder."#,
                        &escaped_identifier
                    ),
                )
                .add_parameter("IceRpc.IceDecoder", "decoder", None, "The decoder.")
                .set_body(decode_data_members(
                    &members,
                    &namespace,
                    FieldType::NonMangled,
                    ast,
                ))
                .build(),
        );

        // Encode method
        builder.add_block(
            FunctionBuilder::new("public readonly", "void", "Encode")
                .add_comment("summary", "Encodes the fields of this struct.")
                .add_parameter("IceRpc.Encoder", "encoder", None, "The encoder.")
                .set_body(encode_data_members(
                    &members,
                    &namespace,
                    FieldType::NonMangled,
                    ast,
                ))
                .build(),
        );

        self.code_map.insert(struct_def, builder.build().into());
    }
}
