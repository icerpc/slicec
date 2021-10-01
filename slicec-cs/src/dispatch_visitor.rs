use crate::builders::{ContainerBuilder, FunctionBuilder};
use crate::code_block::CodeBlock;
use crate::code_map::{self, CodeMap};
use crate::comments::*;
use crate::cs_util::*;
use crate::decoding::*;
use crate::encoding::*;
use crate::proxy_visitor::{get_invocation_params, operation_return_task, to_tuple_type};
use slice::ast::{Ast, Node};
use slice::grammar::*;
use slice::util::*;
use slice::visitor::Visitor;

pub struct DispatchVisitor<'a> {
    pub code_map: &'a mut CodeMap,
}

impl<'a> Visitor for DispatchVisitor<'_> {
    fn visit_interface_start(&mut self, interface_def: &Interface, _: usize, ast: &Ast) {
        let bases = interface_def.bases(ast);
        let interface_name = interface_name(interface_def);

        let mut interface_builder =
            ContainerBuilder::new("public partial interface", &interface_name);

        // TODO: add doc comments and attributes
        // writeServantDocComment(p, getDeprecateReason(p));
        // emitCommonAttributes();
        // emitTypeIdAttribute(p->scoped());
        // emitCustomAttributes(p);

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
            // TODO:
            // writeReturnValueStruct(op);
            // writeMethodDeclaration(op);
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

        if non_streamed_parameters.len() == 0 {
            continue;
        }

        let operation_name = &escape_identifier(operation, CaseStyle::Pascal);

        let decoder_factory = if operation.sends_classes(ast) {
            "request.GetIceDecoderFactory(_defaultIceDecoderFactories.Ice11DecoderFactory)"
        } else {
            "request.GetIceDecoderFactory(_defaultIceDecoderFactories)"
        };

        let code = format!(
            "\
///<summary>Decodes the argument{s} of operation {operation_name}.</summary>
public static {return_type} {operation_name}(IceRpc.IncomingRequest request) =>
    request.ToArgs(
        {decoder_factory},
        {decode_func});",
            s = if non_streamed_parameters.len() == 1 { "" } else { "s" },
            return_type = to_tuple_type(&non_streamed_parameters, true, ast),
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

        if non_streamed_returns.len() == 0 {
            continue;
        }

        let operation_name = &escape_identifier(operation, CaseStyle::Pascal);
        let returns_classes = operation.returns_classes(ast);
        let return_type = &to_tuple_type(&non_streamed_returns, false, ast);

        let mut builder = FunctionBuilder::new(
            "public static",
            "global::System.ReadOnlyMemory<global::System.ReadOnlyMemory<byte>>",
            &operation_name,
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
            builder.add_parameter("IceEncoding", "encoding", "The encoding of the payload");
        }

        if non_streamed_returns.len() == 1 {
            builder.add_parameter(
                return_type,
                "returnValue",
                "The return value to write into the new response payload.",
            );
        } else {
            builder.add_parameter(
                return_type,
                "returnValueTuple",
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

        class_builder.add_block(builder.build().into());
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

fn response_encode_action(operation: &Operation, ast: &Ast) -> CodeBlock {
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
            tuple_type = to_tuple_type(&returns, false, ast),
            encode_action = encode_operation(operation, true, ast),
        )
        .into()
    }
}

fn operation_declaration(operation: &Operation, ast: &Ast) -> CodeBlock {
    format!(
        "\
{comment}
public {return_task} {name}Async({parameters});",
        comment = "///TODO:",
        return_task = operation_return_task(operation, true, ast),
        name = escape_identifier(operation, CaseStyle::Pascal),
        parameters = get_invocation_params(operation, ast).join(", ")
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
    let stream_return = operation.stream_return(ast);

    let mut code = CodeBlock::new();

    if stream_parameter.is_some() {
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
                var_name = parameter_name(&parameters.first().unwrap(), "iceP_", true),
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

    let returns_classes = operation.returns_classes(ast);

    code
}
