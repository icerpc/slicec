use slice::ast::{Ast, Node};
use slice::grammar::*;
use slice::util::*;
use slice::visitor::Visitor;

use crate::builders::{ContainerBuilder, FunctionBuilder};
use crate::code_block::CodeBlock;
use crate::code_map::CodeMap;
use crate::cs_util::*;
use crate::decoding::*;
use crate::encoded_result::{encoded_result_struct, has_encoded_result};
use crate::encoding::*;
use crate::member_util::*;
use crate::operation_util::*;

pub struct DispatchVisitor<'a> {
    pub code_map: &'a mut CodeMap,
}

impl<'a> Visitor for DispatchVisitor<'_> {
    fn visit_interface_start(&mut self, interface_def: &Interface, _: usize, ast: &Ast) {
        let bases = interface_def.bases(ast);
        let interface_name = interface_name(interface_def);

        let mut interface_builder =
            ContainerBuilder::new("public partial interface", &interface_name);

        // TODO: add doc comments and deprecate attribute
        // writeServantDocComment(p, getDeprecateReason(p));

        interface_builder
            .add_type_id_attribute(interface_def)
            .add_custom_attributes(interface_def);

        interface_builder.add_bases(
            &bases
                .iter()
                .map(|base| escape_scoped_identifier(*base, CaseStyle::Pascal, base.scope()))
                .collect::<Vec<_>>(),
        );

        interface_builder
            .add_block(request_class(interface_def, ast))
            .add_block(response_class(interface_def, ast));

        interface_builder.add_block(format!("\
private static readonly DefaultIceDecoderFactories _defaultIceDecoderFactories = new(typeof({}).Assembly);
", interface_name).into());

        for operation in interface_def.operations(ast) {
            interface_builder.add_block(encoded_result_struct(operation, ast));
            interface_builder.add_block(operation_declaration(operation, ast));
        }

        for operation in interface_def.operations(ast) {
            interface_builder.add_block(operation_dispatch(interface_def, operation, ast));
        }

        self.code_map
            .insert(interface_def, interface_builder.build().into());
    }
}

fn request_class(interface_def: &Interface, ast: &Ast) -> CodeBlock {
    let bases = interface_def.bases(ast);
    let operations = interface_def.operations(ast);

    if !operations.iter().any(|o| o.has_non_streamed_params(ast)) {
        return "".into();
    }

    let mut class_builder = ContainerBuilder::new(
        if bases.is_empty() {
            "public static class"
        } else {
            "public static new class"
        },
        "Request",
    );

    class_builder.add_comment(
        "summary",
        "Provides static methods that read the arguments of requests.",
    );

    for operation in operations {
        let non_streamed_parameters = operation.non_streamed_params(ast);

        if non_streamed_parameters.is_empty() {
            continue;
        }

        let operation_name = &escape_identifier(operation, CaseStyle::Pascal);

        let decoder_factory = if operation.sends_classes(ast) {
            "request.GetIceDecoderFactory(_defaultIceDecoderFactories.Ice11DecoderFactory)"
        } else {
            "request.GetIceDecoderFactory(_defaultIceDecoderFactories)"
        };

        let ns = get_namespace(operation);

        let code = format!(
            "\
///<summary>Decodes the argument{s} of operation {operation_name}.</summary>
public static {return_type} {operation_name}(IceRpc.IncomingRequest request) =>
    request.ToArgs(
        {decoder_factory},
        {decode_func});",
            s = if non_streamed_parameters.len() == 1 { "" } else { "s" },
            return_type = to_tuple_type(&non_streamed_parameters, &ns, ast, TypeContext::Outgoing),
            operation_name = operation_name,
            decoder_factory = decoder_factory,
            decode_func = request_decode_func(operation, ast).indent().indent(),
        );

        class_builder.add_block(code.into());
    }

    class_builder.build().into()
}

fn response_class(interface_def: &Interface, ast: &Ast) -> CodeBlock {
    let bases = interface_def.bases(ast);

    let operations = interface_def.operations(ast);

    if !operations.iter().any(|o| o.has_non_streamed_return(ast)) {
        return "".into();
    }

    let mut class_builder = ContainerBuilder::new(
        if bases.is_empty() {
            "public static class"
        } else {
            "public static new class"
        },
        "Response",
    );

    class_builder.add_comment(
        "summary",
        "Provides static methods that read the arguments of requests.",
    );

    for operation in operations {
        let non_streamed_returns = operation.non_streamed_returns(ast);

        if non_streamed_returns.is_empty() {
            continue;
        }

        let ns = get_namespace(operation);
        let operation_name = &escape_identifier(operation, CaseStyle::Pascal);
        let returns_classes = operation.returns_classes(ast);
        let return_type = &to_tuple_type(&non_streamed_returns, &ns, ast, TypeContext::Outgoing);

        let mut builder = FunctionBuilder::new(
            "public static",
            "global::System.ReadOnlyMemory<global::System.ReadOnlyMemory<byte>>",
            operation_name,
        );

        builder
            .add_comment(
                "summary",
                &format!(
                    "Creates a respons payload for operation {}.",
                    &operation_name
                ),
            )
            .add_comment("returns", "A new response payload.");

        if !returns_classes {
            builder.add_parameter(
                "IceEncoding",
                "encoding",
                None,
                "The encoding of the payload",
            );
        }

        if non_streamed_returns.len() == 1 {
            builder.add_parameter(
                return_type,
                "returnValue",
                None,
                "The return value to write into the new response payload.",
            );
        } else {
            builder.add_parameter(
                return_type,
                "returnValueTuple",
                None,
                "The return values to write into the new response payload.",
            );
        };

        builder.use_expression_body(true);

        let body = format!(
            "\
{encoding}.{encoding_operation}(
    {return_arg},
    {encode_action},
    {class_format})",
            encoding = if returns_classes {
                "IceRpc.Encoding.Ice11"
            } else {
                "encoding"
            },
            encoding_operation = if non_streamed_returns.len() == 1 {
                "CreatePayloadFromSingleReturnValue"
            } else {
                "CreatePayloadFromReturnValueTuple"
            },
            return_arg = if non_streamed_returns.len() == 1 {
                "returnValue"
            } else {
                "returnValueTuple"
            },
            encode_action = response_encode_action(operation, ast),
            class_format = operation_format_type_to_string(operation)
        );

        builder.set_body(body.into());

        class_builder.add_block(builder.build());
    }

    class_builder.build().into()
}

fn request_decode_func(operation: &Operation, ast: &Ast) -> CodeBlock {
    let ns = get_namespace(operation);

    let parameters = operation.parameters(ast);

    let use_default_decode_func = parameters.len() == 1
        && get_bit_sequence_size(&parameters, ast) == 0
        && parameters.first().unwrap().tag.is_none();

    if use_default_decode_func {
        decode_func(&parameters.first().unwrap().data_type, &ns, ast)
    } else {
        format!(
            "decoder =>
{{
    {};
}}",
            decode_operation(operation, false, ast).indent()
        )
        .into()
    }
}

pub fn response_encode_action(operation: &Operation, ast: &Ast) -> CodeBlock {
    let ns = get_namespace(operation);

    // We only want the non-streamed returns
    let returns = operation.non_streamed_returns(ast);

    // When the operation returns a T? where T is an interface or a class, there is a built-in
    // encoder, so defaultEncodeAction is true.
    let use_default_encode_action = returns.len() == 1
        && get_bit_sequence_size(&returns, ast) == 0
        && returns.first().unwrap().tag.is_none();

    if use_default_encode_action {
        encode_action(&returns.first().unwrap().data_type, &ns, true, true, ast)
    } else {
        let encoder_class = if operation.returns_classes(ast) {
            "Ice11Encoder"
        } else {
            "IceEncoder"
        };

        format!(
            "({encoder} encoder, {_in}{tuple_type} value) => {{ {encode_action} }}",
            encoder = encoder_class,
            _in = if returns.len() == 1 { "" } else { "in " },
            tuple_type = to_tuple_type(&returns, &ns, ast, TypeContext::Outgoing),
            encode_action = encode_operation(operation, true, ast),
        )
        .into()
    }
}

fn operation_declaration(operation: &Operation, ast: &Ast) -> CodeBlock {
    let ns = get_namespace(operation);
    format!(
        "\
{comment}
public {return_task} {name}Async({parameters});",
        comment = "///TODO:",
        return_task = operation_return_task(operation, &ns, true, ast),
        name = escape_identifier(operation, CaseStyle::Pascal),
        parameters = operation_params(operation, true, ast).join(", ")
    )
    .into()
}

fn operation_dispatch(interface_def: &Interface, operation: &Operation, ast: &Ast) -> CodeBlock {
    let operation_name = &escape_identifier(operation, CaseStyle::Pascal);
    let internal_name = format!("IceD{}Async", &operation_name);

    format!(
        r#"
[IceRpc.Slice.Operation("{name}")]
protected static async global::System.Threading.Tasks.ValueTask<(IceEncoding, global::System.ReadOnlyMemory<global::System.ReadOnlyMemory<byte>>, IceRpc.IStreamParamSender?)> {internal_name}(
    {interface_name} target,
    IceRpc.IncomingRequest request,
    IceRpc.Dispatch dispatch,
    global::System.Threading.CancellationToken cancel)
{{
    {dispatch_body}
}}
"#,
        name = operation.identifier(),
        internal_name = internal_name,
        interface_name = interface_name(interface_def),
        dispatch_body = operation_dispatch_body(operation, ast).indent()
    )
    .into()
}

fn operation_dispatch_body(operation: &Operation, ast: &Ast) -> CodeBlock {
    let ns = get_namespace(operation);
    let operation_name = &escape_identifier(operation, CaseStyle::Pascal);
    let parameters = operation.parameters(ast);
    let stream_parameter = operation.stream_parameter(ast);
    let return_parameters = operation.return_members(ast);

    let mut code = CodeBlock::new();

    if stream_parameter.is_none() {
        code.writeln("request.StreamReadingComplete();");
    }

    if !operation.is_idempotent() {
        code.writeln("request.CheckNonIdempotent();");
    }

    if operation.compress_return() {
        // At this point, Dispatch is just created and the application had no opportunity to set any
        // response feature.
        code.writeln("dispatch.ResponseFeatures = IceRpc.FeatureCollectionExtensions.CompressPayload(dispatch.ResponseFeatures);");
    }

    // Even when the parameters are empty, we verify the payload is indeed empty (can contain
    // tagged params
    // that we skip).
    if parameters.is_empty() {
        code.writeln(
            "request.CheckEmptyArgs(request.GetIceDecoderFactory(_defaultIceDecoderFactories));",
        );
    }

    // TODO: cleanup stream logic
    match parameters.len() {
        0 => {}
        1 if stream_parameter.is_some() => {
            let stream_parameter = stream_parameter.unwrap();
            let name = parameter_name(stream_parameter, "iceP_", true);
            let stream_assignment = match stream_parameter.data_type.definition(ast) {
                Node::Primitive(_, b) if matches!(b, Primitive::Byte) => {
                    "IceRpc.Slice.StreamParamReceiver.ToByteStream(request)".to_owned()
                }
                _ => {
                    format!(
                        "\
IceRpc.Slice.StreamParamReceiver.ToAsyncEnumerable<{stream_type}>(
    request,
    request.GetIceDecoderFactory(_defaultIceDecoderFactories),
    {decode_func})
    ",
                        stream_type = type_to_string(
                            &stream_parameter.data_type,
                            &ns,
                            ast,
                            TypeContext::Outgoing
                        ),
                        decode_func = decode_func(&stream_parameter.data_type, &ns, ast)
                    )
                }
            };
            writeln!(code, "{} = {}", name, stream_assignment);
        }
        1 => {
            writeln!(
                code,
                "var {var_name} = Request.{operation_name}(request);",
                var_name = parameter_name(parameters.first().unwrap(), "iceP_", true),
                operation_name = operation_name,
            )
        }
        _ => {
            // > 1 parameter
            writeln!(
                code,
                "var args = Request.{operation_name}(request);",
                operation_name = operation_name,
            )
        }
    };

    if has_encoded_result(operation) {
        // TODO: support for stream param with encoded result?

        let mut args = vec![];

        match parameters.as_slice() {
            [p] => {
                args.push(parameter_name(p, "iceP_", true));
            }
            _ => {
                for p in parameters {
                    args.push("args.".to_owned() + &field_name(p, FieldType::NonMangled));
                }
            }
        }
        // TODO: should these be escaped?
        args.push("dispatch".to_owned());
        args.push("cancel".to_owned());

        writeln!(
            code,
            "var returnValue = await target.{name}({args}).ConfigureAwait(false);",
            name = operation_name,
            args = args.join(", ")
        );

        let encoding = if operation.returns_classes(ast) {
            "IceRpc.Encoding.Ice11"
        } else {
            "request.GetIceEncoding()"
        };

        writeln!(
            code,
            "return ({encoding}, returnValue,Payload, null)",
            encoding = encoding
        );
    } else {
        let mut args = match parameters.as_slice() {
            [parameter] => vec![parameter_name(parameter, "iceP_", true)],
            _ => parameters
                .iter()
                .map(|parameter| format!("args.{}", &field_name(parameter, FieldType::NonMangled)))
                .collect(),
        };
        args.push("dispatch".to_owned());
        args.push("cancel".to_owned());

        writeln!(
            code,
            "{return_value}await target.{operation_name}({args}).ConfigureAwait(false);",
            return_value = if !return_parameters.is_empty() {
                "var returnValue = "
            } else {
                ""
            },
            operation_name = operation_name,
            args = args.join(", ")
        );

        let encoding = if operation.returns_classes(ast) {
            "IceRpc.Encoding.Ice11"
        } else {
            code.writeln("var payloadEncoding = request.GetIceEncoding();");
            "payloadEncoding"
        };

        writeln!(
            code,
            "return ({encoding}, {payload}, {stream});",
            encoding = encoding,
            payload = dispatch_return_payload(operation, encoding, ast),
            stream = stream_param_sender(operation, encoding, ast)
        );
    }

    code
}

fn dispatch_return_payload(operation: &Operation, encoding: &str, ast: &Ast) -> CodeBlock {
    let return_values = operation.return_members(ast);
    let return_stream = operation.stream_return(ast);
    let non_streamed_return_values = operation.non_streamed_returns(ast);

    let mut returns = vec![];

    if !operation.returns_classes(ast) {
        returns.push(encoding.to_owned());
    }

    if return_stream.is_some() {
        returns.extend(non_streamed_return_values.iter().map(|return_value| {
            "returnValue.".to_owned() + &field_name(return_value, FieldType::NonMangled)
        }));
    } else {
        returns.push("returnValue".to_owned());
    };

    match return_values.len() {
        0 => format!("{}.CreateEmptyPayload()", encoding),
        1 if return_stream.is_some() => format!("{}.CreateEmptyPayload()", encoding),
        _ => format!(
            "Response.{operation_name}({args})",
            operation_name = escape_identifier(operation, CaseStyle::Pascal),
            args = returns.join(", ")
        ),
    }
    .into()
}

fn stream_param_sender(operation: &Operation, encoding: &str, ast: &Ast) -> CodeBlock {
    let ns = get_namespace(operation);
    let return_values = operation.return_members(ast);
    if let Some(stream_parameter) = operation.stream_parameter(ast) {
        let node = stream_parameter.data_type.definition(ast);

        let stream_arg = if return_values.len() == 1 {
            "returnValue".to_owned()
        } else {
            "returnValue.".to_owned() + &field_name(stream_parameter, FieldType::NonMangled)
        };

        let stream_type =
            type_to_string(&stream_parameter.data_type, &ns, ast, TypeContext::Outgoing);

        match node {
        Node::Primitive(_, b) if matches!(b, Primitive::Byte) => {
            format!("new IceRpc.Slice.ByteStreamParamSender({})", stream_arg).into()
        }
        _ => format!("\
    new IceRpc.Slice.AsyncEnumerableStreamParamSender<{stream_type}>({stream_arg}, {encoding} {decode_func})",
        stream_type = stream_type,
        stream_arg = stream_arg,
        encoding = encoding,
        decode_func = decode_func(&stream_parameter.data_type, &ns, ast)).into()
    }
    } else {
        "null".into()
    }
}
