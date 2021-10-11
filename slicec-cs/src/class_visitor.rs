// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::builders::{ContainerBuilder, FunctionBuilder};
use crate::code_block::CodeBlock;
use crate::code_map::CodeMap;
use crate::cs_util::*;
use crate::decoding::decode_data_members;
use crate::encoding::encode_data_members;
use slice::ast::Ast;
use slice::grammar::{Class, Member};
use slice::util::{CaseStyle, TypeContext};
use slice::visitor::Visitor;

pub struct ClassVisitor<'a> {
    pub code_map: &'a mut CodeMap,
}

impl<'a> Visitor for ClassVisitor<'_> {
    fn visit_class_start(&mut self, class_def: &Class, _: usize, ast: &Ast) {
        let class_name = escape_identifier(class_def, CaseStyle::Pascal);
        let namespace = get_namespace(class_def);
        let has_base_class = class_def.base(ast).is_some();

        let members = class_def.members(ast);
        let base_members = if let Some(base) = class_def.base(ast) {
            base.all_data_members(ast)
        } else {
            vec![]
        };

        let non_default_members = members
            .iter()
            .cloned()
            .filter(|m| !is_member_default_initialized(m, ast))
            .collect::<Vec<_>>();

        let non_default_base_members = base_members
            .iter()
            .cloned()
            .filter(|m| !is_member_default_initialized(m, ast))
            .collect::<Vec<_>>();

        // TODO:
        // writeTypeDocComment(p, getDeprecateReason(p));

        // emitCommonAttributes();
        // emitTypeIdAttribute(p->scoped());
        // if (p->compactId() >= 0)
        // {
        //     emitCompactTypeIdAttribute(p->compactId());
        // }
        // emitCustomAttributes(p);

        let mut class_builder = ContainerBuilder::new("public partial class", &class_name);

        if let Some(base) = class_def.base(ast) {
            class_builder.add_base(escape_scoped_identifier(
                base,
                CaseStyle::Pascal,
                &namespace,
            ));
        } else {
            class_builder.add_base("IceRpc.AnyClass".to_owned());
        }

        // Add class fields
        class_builder.add_block(
            members
                .iter()
                .map(|m| data_member_declaration(m, false, FieldType::Class, ast))
                .collect::<Vec<_>>()
                .join("\n\n")
                .into(),
        );

        // Class static TypeId string
        class_builder.add_block(
            format!(
                "public static{} readonly string IceTypeId = typeof({}).GetIceTypedId()!",
                if has_base_class { " new" } else { "" },
                class_name,
            )
            .into(),
        );

        // TODO: get compact id
        let compact_id = -1;

        if compact_id >= 0 {
            class_builder.add_block(
                format!(
                "private static readonly int _compactTypeId = typeof({}).GetIceCompactTypeId()!.Value;",
                class_name
            ).into());
        }

        let constructor_summary = format!(
            r#"Constructs a new instance of <see cref="{}"/>."#,
            class_name
        );

        // One-shot ctor (may be parameterless)
        class_builder.add_block(constructor(
            &class_name,
            &constructor_summary,
            &namespace,
            &members,
            &base_members,
            ast,
        ));

        // Second public constructor for all data members minus those with a default initializer
        // This constructor is only generated if necessary
        if non_default_members.len() + non_default_base_members.len()
            < members.len() + base_members.len()
        {
            class_builder.add_block(constructor(
                &class_name,
                &constructor_summary,
                &namespace,
                &non_default_members,
                &non_default_base_members,
                ast,
            ));
        }

        // public constructor used for decoding
        // the decoder parameter is used to distinguish this ctor from the parameterless ctor that
        // users may want to add to the partial class. It's not used otherwise.
        let mut decode_constructor = FunctionBuilder::new("public", "", &class_name);

        if !has_base_class {
            let silence_warnings = r#"[global::System.Diagnostics.CodeAnalysis.SuppressMessage(
    "Microsoft.Performance",
    "CA1801: Review unused parameters",
    Justification="Special constructor used for Ice decoding")]"#;

            decode_constructor.add_attribute(silence_warnings);
        }

        // TODO: add this attribute
        // emitEditorBrowsableNeverAttribute();
        decode_constructor.add_parameter("Ice11Decoder", "decoder", None, "");
        if has_base_class {
            decode_constructor.add_base_argument("decoder");
        }
        decode_constructor.set_body(initialize_non_nullable_fields(
            &members,
            FieldType::Class,
            ast,
        ));

        class_builder.add_block(decode_constructor.build());

        class_builder.add_block(encode_and_decode(class_def, ast));

        self.code_map
            .insert(class_def, class_builder.build().into());
    }
}

fn constructor(
    escaped_name: &str,
    summary_comment: &str,
    namespace: &str,
    members: &[&Member],
    base_members: &[&Member],
    ast: &Ast,
) -> CodeBlock {
    let mut code = CodeBlock::new();

    let mut builder = FunctionBuilder::new("public", "", escaped_name);

    builder.add_comment("summary", summary_comment);

    builder.add_base_arguments(
        &base_members
            .iter()
            .filter(|m| !is_member_default_initialized(m, ast))
            .map(|m| escape_identifier(*m, CaseStyle::Camel))
            .collect::<Vec<String>>(),
    );

    for member in members.iter().chain(base_members.iter()) {
        let parameter_type =
            type_to_string(&member.data_type, namespace, ast, TypeContext::DataMember);
        let parameter_name = escape_identifier(*member, CaseStyle::Camel);

        // TODO get comment
        // CommentInfo comment = processComment(member, "");
        // writeDocCommentLines(_out, comment.summaryLines, "param", "name", paramName(member, "",
        // false));
        let comment = "";
        builder.add_parameter(&parameter_type, &parameter_name, None, comment);
    }

    builder.set_body({
        let mut code = CodeBlock::new();
        for member in members {
            writeln!(
                code,
                "this.{} = {};",
                field_name(member, FieldType::Class),
                escape_identifier(*member, CaseStyle::Camel)
            );
        }
        code
    });

    code.add_block(&builder.build());

    code
}

fn encode_and_decode(class_def: &Class, ast: &Ast) -> CodeBlock {
    let mut code = CodeBlock::new();

    let namespace = get_namespace(class_def);
    let members = class_def.members(ast);
    let has_base_class = class_def.base(ast).is_some();

    // const bool basePreserved = p->inheritsMetadata("preserve-slice");
    // const bool preserved = p->hasMetadata("preserve-slice");

    let is_base_preserved = false;
    let is_preserved = false;

    if is_preserved && !is_base_preserved {
        let ice_unknown_slices = "protected override global::System.Collections.Immutable.ImmutableList<IceRpc.Slice.SliceInfo> IceUnknownSlices { get; set; } = global::System.Collections.Immutable.ImmutableList<IceRpc.Slice.SliceInfo>.Empty;".to_owned();
        code.add_block(&ice_unknown_slices.into());
    }

    let encode_class = FunctionBuilder::new("protected override", "void", "IceEncode")
        .add_parameter("Ice11Encoder", "encoder", None, "")
        .set_body({
            let mut code = CodeBlock::new();

            let mut start_slice_args = vec!["IceTypeId"];
            // TODO no way to get class compact id yet
            let compact_id = -1;
            if compact_id >= 0 {
                start_slice_args.push("_compactTypeId");
            }

            writeln!(
                code,
                "encoder.IceStartSlice({});",
                start_slice_args.join(", ")
            );

            code.writeln(&encode_data_members(
                &members,
                &namespace,
                FieldType::Class,
                ast,
            ));

            if has_base_class {
                code.writeln("encoder.IceEndSlice(false);");
                code.writeln("base.IceEncode(encoder);");
            } else {
                code.writeln("encoder.IceEndSlice(true);"); // last slice
            }

            code
        })
        .build();

    let decode_class = FunctionBuilder::new("protected override", "void", "IceDecode")
        .add_parameter("Ice11Decoder", "decoder", None, "")
        .set_body({
            let mut code = CodeBlock::new();
            code.writeln("decoder.IceStartSlice();");
            code.writeln(&decode_data_members(
                &members,
                &namespace,
                FieldType::Class,
                ast,
            ));
            code.writeln("decoder.IceEndSlice();");
            if has_base_class {
                code.writeln("base.IceDecode(decoder);");
            }
            code
        })
        .build();

    code.add_block(&encode_class);
    code.add_block(&decode_class);

    code
}
