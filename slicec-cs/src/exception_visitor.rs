// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::builders::{ContainerBuilder, FunctionBuilder};
use crate::code_block::CodeBlock;
use crate::code_map::CodeMap;
use crate::cs_util::*;
use crate::decoding::decode_data_members;
use crate::encoding::encode_data_members;
use slice::ast::Ast;
use slice::grammar::Exception;
use slice::util::{CaseStyle, TypeContext};
use slice::visitor::Visitor;

pub struct ExceptionVisitor<'a> {
    pub code_map: &'a mut CodeMap,
}

impl<'a> Visitor for ExceptionVisitor<'_> {
    fn visit_exception_start(&mut self, exception_def: &Exception, _: usize, ast: &Ast) {
        let exception_name = escape_identifier(exception_def, CaseStyle::Pascal);
        let has_base = exception_def.base.is_some();

        let ns = get_namespace(exception_def);

        let members = exception_def.members(ast);

        let has_public_parameter_constructor = exception_def
            .all_data_members(ast)
            .iter()
            .all(|m| is_member_default_initialized(m, ast));

        // TODO: generate doc and attributes
        // writeTypeDocComment(p, getDeprecateReason(p));
        // emitDeprecate(p, false, _out);

        // emitCommonAttributes();
        // emitTypeIdAttribute(p->scoped());
        // emitCustomAttributes(p);

        let mut exception_class_builder =
            ContainerBuilder::new("public partial class", &exception_name);

        if let Some(base) = exception_def.base(ast) {
            exception_class_builder.add_base(escape_scoped_identifier(
                base,
                CaseStyle::Pascal,
                &ns,
            ));
        } else {
            exception_class_builder.add_base("IceRpc.RemoteException".to_owned());
        }

        exception_class_builder.add_block(
            members
                .iter()
                .map(|m| data_member_declaration(m, false, FieldType::Exception, ast))
                .collect::<Vec<_>>()
                .join("\n\n")
                .into(),
        );

        exception_class_builder.add_block(
            format!(
                "private static readonly string _iceTypeId = typeof({}).GetIceTypeId()!;",
                exception_name
            )
            .into(),
        );

        exception_class_builder
            .add_block(one_shot_constructor(exception_def, false, ast))
            .add_block(one_shot_constructor(exception_def, true, ast));

        // public parameter-less constructor
        if has_public_parameter_constructor {
            exception_class_builder.add_block(
                FunctionBuilder::new("public", "", &exception_name)
                    .add_parameter(
                        "IceRpc.RetryPolicy?",
                        "retryPolicy",
                        None,
                        "The retry policy for the exception",
                    )
                    .add_base_argument("retryPolicy")
                    .build(),
            );
        }

        exception_class_builder.add_block(
            FunctionBuilder::new("public", "", &exception_name)
                .add_parameter("Ice11Decoder", "decoder", None, "")
                .add_base_argument("decoder")
                .set_body(initialize_non_nullable_fields(
                    &members,
                    FieldType::Exception,
                    ast,
                ))
                .build(),
        );

        if !has_base && !exception_def.uses_classes(ast) {
            // public constructor used for Ice 2.0 decoding
            // TODO: emitEditorBrowsableNeverAttribute();
            exception_class_builder.add_block(
                FunctionBuilder::new("public", "", &exception_name)
                    .add_parameter("Ice20Decoder", "decoder", None, "")
                    .add_base_argument("decoder")
                    .set_body(decode_data_members(
                        &members,
                        &ns,
                        FieldType::Exception,
                        ast,
                    ))
                    .build(),
            );
        }

        // Remote exceptions are always "preserved".
        exception_class_builder.add_block(
            FunctionBuilder::new("protected override", "void", "IceDecode")
                .add_parameter("Ice11Decoder", "decoder", None, "")
                .set_body({
                    let mut code = CodeBlock::new();
                    code.writeln("decoder.IceStartSlice();");
                    code.writeln(&decode_data_members(
                        &members,
                        &ns,
                        FieldType::Exception,
                        ast,
                    ));
                    code.writeln("decoder.IceEndSlice();");

                    if has_base {
                        code.writeln("base.IceDecode(decoder);");
                    }
                    code
                })
                .build(),
        );

        exception_class_builder.add_block(
            FunctionBuilder::new("protected override", "void", "IceEncode")
                .add_parameter("Ice11Encoder", "encoder", None, "")
                .set_body({
                    let mut code = CodeBlock::new();
                    code.writeln("encoder.IceStartSlice(_iceTypeId);");
                    code.writeln(&encode_data_members(
                        &members,
                        &ns,
                        FieldType::Exception,
                        ast,
                    ));

                    if has_base {
                        code.writeln("encoder.IceEndSlice(lastSlice: false);");
                        code.writeln("base.IceEncode(encoder);");
                    } else {
                        code.writeln("encoder.IceEndSlice(lastSlice: true);")
                    }

                    code
                })
                .build(),
        );

        if !has_base && !exception_def.uses_classes(ast) {
            exception_class_builder.add_block(
                FunctionBuilder::new("protected override", "void", "IceEncode")
                    .add_parameter("Ice20Encoder", "encoder", None, "")
                    .set_body({
                        let mut code = CodeBlock::new();
                        code.writeln("encoder.EncodeString(_iceTypeId);");
                        code.writeln("encoder.EncodeString(Message);");
                        code.writeln("Origin.Encode(encoder);");
                        code.writeln(&encode_data_members(
                            &members,
                            &ns,
                            FieldType::Exception,
                            ast,
                        ));
                        code
                    })
                    .build(),
            );
        }

        self.code_map
            .insert(exception_def, exception_class_builder.build().into());
    }
}

fn one_shot_constructor(
    exception_def: &Exception,
    add_message_and_exception_parameters: bool,
    ast: &Ast,
) -> CodeBlock {
    let exception_name = escape_identifier(exception_def, CaseStyle::Pascal);

    let ns = get_namespace(exception_def);

    let all_data_members = exception_def.all_data_members(ast);

    if all_data_members.is_empty() && !add_message_and_exception_parameters {
        return CodeBlock::new();
    }

    let message_parameter_name = escape_parameter_name(&all_data_members, "message");
    let inner_exception_parameter_name = escape_parameter_name(&all_data_members, "innerException");
    let retry_policy_parameter_name = escape_parameter_name(&all_data_members, "retryPolicy");

    let all_parameters = all_data_members
        .iter()
        .map(|m| {
            let member_type = type_to_string(&m.data_type, &ns, ast, TypeContext::DataMember);
            let member_name = escape_identifier(*m, CaseStyle::Camel);
            format!("{} {}", member_type, member_name)
        })
        .collect::<Vec<_>>();

    let base_parameters = if let Some(base) = exception_def.base(ast) {
        base.all_data_members(ast)
            .iter()
            .map(|m| escape_identifier(*m, CaseStyle::Pascal))
            .collect::<Vec<_>>()
    } else {
        vec![]
    };

    let mut ctor_builder = FunctionBuilder::new("public", "", &exception_name);

    ctor_builder.add_comment(
        "summary",
        &format!(
            r#"Constructs a new instance of <see cref="{}"/>."#,
            &exception_name
        ),
    );

    if add_message_and_exception_parameters {
        ctor_builder.add_parameter(
            "string?",
            &message_parameter_name,
            None,
            "Message that describes the exception.",
        );
        ctor_builder.add_base_argument(&message_parameter_name);
    }

    ctor_builder.add_parameters(&all_parameters);
    ctor_builder.add_base_arguments(&base_parameters);

    if add_message_and_exception_parameters {
        ctor_builder.add_parameter(
            "global::System.Exception?",
            &inner_exception_parameter_name,
            Some("null"),
            "The exception that is the cause of the current exception.",
        );
        ctor_builder.add_base_argument(&inner_exception_parameter_name);
    }

    ctor_builder.add_parameter(
        "IceRpc.RetryPolicy?",
        &retry_policy_parameter_name,
        Some("null"),
        "The retry policy for the exception.",
    );
    ctor_builder.add_base_argument(&retry_policy_parameter_name);

    // ctor impl
    let mut ctor_body = CodeBlock::new();
    for member in exception_def.members(ast) {
        let member_name = field_name(member, FieldType::Exception);
        let parameter_name = escape_identifier(member, CaseStyle::Camel);

        writeln!(ctor_body, "this.{} = {};", member_name, parameter_name);
    }

    ctor_builder.set_body(ctor_body);

    ctor_builder.build()
}
