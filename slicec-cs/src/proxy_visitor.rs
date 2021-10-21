// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::builders::{
    AttributeBuilder, CommentBuilder, ContainerBuilder, FunctionBuilder, FunctionType,
};
use crate::code_block::CodeBlock;
use crate::code_map::CodeMap;
use crate::comments::*;
use crate::decoding::*;
use crate::encoding::*;
use crate::member_util::*;
use crate::traits::*;
use slice::ast::{Ast, Node};
use slice::grammar::*;
use slice::util::*;
use slice::visitor::Visitor;

pub struct ProxyVisitor<'a> {
    pub code_map: &'a mut CodeMap,
}

impl<'a> Visitor for ProxyVisitor<'_> {
    fn visit_interface_start(&mut self, interface_def: &Interface, _: usize, ast: &Ast) {
        let prx_interface = interface_def.proxy_name(); // IFooPrx
        let prx_impl: String = interface_def.proxy_implementation_name(); // FooPrx

        let all_bases: Vec<&Interface> = interface_def.all_bases(ast);
        let bases: Vec<&Interface> = interface_def.bases(ast);

        let mut prx_impl_bases: Vec<String> = vec![prx_interface.clone(), "IPrx".to_owned()];

        let mut all_base_impl: Vec<String> = all_bases
            .iter()
            .map(|b| b.proxy_implementation_name())
            .collect();

        let mut add_service_prx = false;
        if !all_bases.iter().any(|b| b.scope() == "::IceRpc::Service")
            && interface_def.scoped_identifier() != "::IceRpc::Service"
        {
            prx_impl_bases.push("IceRpc.IServicePrx".to_owned());
            all_base_impl.push("IceRpc.ServicePrx".to_owned());
            add_service_prx = true;
        }

        // prx bases
        let prx_bases: Vec<String> = bases.into_iter().map(|b| b.proxy_name()).collect();

        let summary_message = format!(
            r#"The client-side interface for Slice interface {}. <seealso cref="{}"/>.
{}"#,
            interface_def.identifier(),
            interface_def.interface_name(),
            doc_comment_message(interface_def)
        );

        let proxy_interface = ContainerBuilder::new("public partial interface", &prx_interface)
            .add_comment("summary", &summary_message)
            .add_type_id_attribute(interface_def)
            .add_container_attributes(interface_def)
            .add_bases(&prx_bases)
            .add_block(proxy_interface_operations(interface_def, ast))
            .build();

        let mut proxy_impl_builder =
            ContainerBuilder::new("public readonly partial record struct", &prx_impl);

        proxy_impl_builder.add_bases(&prx_impl_bases)
            .add_comment("summary", &format!(r#"Typed proxy record struct. It implements <see cref="{}"/> by sending requests to a remote IceRPC service."#, prx_interface))
            .add_type_id_attribute(interface_def)
            .add_container_attributes(interface_def)
            .add_block(request_class(interface_def, ast))
            .add_block(response_class(interface_def, ast))
            .add_block(format!(r#"
/// <summary>The default path for services that implement Slice interface <c>{interface_name}</c>.</summary>
public static readonly string DefaultPath = typeof({prx_impl}).GetDefaultPath();

private static readonly DefaultIceDecoderFactories _defaultIceDecoderFactories = new (typeof({prx_impl}).Assembly);

/// <summary>The proxy to the remote service.</summary>
public IceRpc.Proxy Proxy {{ get; init; }}"#,
            interface_name = interface_def.identifier(),
            prx_impl = prx_impl
        ).into());

        for base_impl in all_base_impl {
            proxy_impl_builder.add_block(
                format!(
                    r#"
/// <summary>Implicit conversion to <see cref="{base_impl}"/>.</summary>
public static implicit operator {base_impl}({prx_impl} prx) => new(prx.Proxy);"#,
                    base_impl = base_impl,
                    prx_impl = prx_impl
                )
                .into(),
            );
        }

        proxy_impl_builder.add_block(proxy_impl_static_methods(interface_def));

        if add_service_prx {
            proxy_impl_builder.add_block(
                "\
/// <inheritdoc/>
public global::System.Threading.Tasks.Task<string[]> IceIdsAsync(
    IceRpc.Invocation? invocation = null,
    global::System.Threading.CancellationToken cancel = default) =>
    new IceRpc.ServicePrx(Proxy).IceIdsAsync(invocation, cancel);

/// <inheritdoc/>
public global::System.Threading.Tasks.Task<bool> IceIsAAsync(
    string id,
    IceRpc.Invocation? invocation = null,
    global::System.Threading.CancellationToken cancel = default) =>
    new IceRpc.ServicePrx(Proxy).IceIsAAsync(id, invocation, cancel);

/// <inheritdoc/>
public global::System.Threading.Tasks.Task IcePingAsync(
    IceRpc.Invocation? invocation = null,
    global::System.Threading.CancellationToken cancel = default) =>
    new IceRpc.ServicePrx(Proxy).IcePingAsync(invocation, cancel);"
                    .into(),
            );
        }

        for operation in interface_def.all_base_operations(ast) {
            proxy_impl_builder.add_block(proxy_base_operation_impl(interface_def, operation, ast));
        }

        for operation in interface_def.operations(ast) {
            proxy_impl_builder.add_block(proxy_operation_impl(operation, ast));
        }

        // Generate abstract methods and documentation
        let code = format!(
            "\n{interface}\n\n{proxy_impl}",
            interface = proxy_interface,
            proxy_impl = proxy_impl_builder.build()
        );

        self.code_map.insert(interface_def, code.into())
    }
}

fn proxy_impl_static_methods(interface_def: &Interface) -> CodeBlock {
    format!(
        r#"/// <summary>Creates a new <see cref="{prx_impl}"/> from the give connection and path.</summary>
/// <param name="connection">The connection. If it's an outgoing connection, the endpoint of the new proxy is
/// <see cref="Connection.RemoteEndpoint"/>; otherwise, the new proxy has no endpoint.</param>
/// <param name="path">The path of the proxy. If null, the path is set to <see cref="DefaultPath"/>.</param>
/// <param name="invoker">The invoker. If null and connection is an incoming connection, the invoker is set to
/// the server's invoker.</param>
/// <returns>The new proxy.</returns>
public static {prx_impl} FromConnection(
    IceRpc.Connection connection,
    string? path = null,
    IceRpc.IInvoker? invoker = null) =>
    new(IceRpc.Proxy.FromConnection(connection, path ?? DefaultPath, invoker));

/// <summary>Creates a new <see cref="{prx_impl}"/> with the given path and protocol.</summary>
/// <param name="path">The path for the proxy.</param>
/// <param name="protocol">The proxy protocol.</param>
/// <returns>The new proxy.</returns>
public static {prx_impl} FromPath(string path, IceRpc.Protocol? protocol = null) =>
    new(IceRpc.Proxy.FromPath(path, protocol ?? IceRpc.Protocol.Ice2));

/// <summary>Creates a new <see cref="{prx_impl}"/> from a string and invoker.</summary>
/// <param name="s">The string representation of the proxy.</param>
/// <param name="invoker">The invoker of the new proxy.</param>
/// <returns>The new proxy</returns>
/// <exception cref="global::System.FormatException"><c>s</c> does not contain a valid string representation of a proxy.
/// </exception>
public static {prx_impl} Parse(string s, IceRpc.IInvoker? invoker = null) => new(IceRpc.Proxy.Parse(s, invoker));

/// <summary>Creates a new <see cref="{prx_impl}"/> from a string and invoker.</summary>
/// <param name="s">The proxy string representation.</param>
/// <param name="invoker">The invoker of the new proxy.</param>
/// <param name="prx">The new proxy.</param>
/// <returns><c>true</c> if the s parameter was parsed successfully; otherwise, <c>false</c>.</returns>
public static bool TryParse(string s, IceRpc.IInvoker? invoker, out {prx_impl} prx)
{{
    if (IceRpc.Proxy.TryParse(s, invoker, out IceRpc.Proxy? proxy))
    {{
        prx = new(proxy);
        return true;
    }}
    else
    {{
        prx = default;
        return false;
    }}
}}

/// <summary>Constructs an instance of <see cref="{prx_impl}"/>.</summary>
/// <param name="proxy">The proxy to the remote service.</param>
public {prx_impl}(IceRpc.Proxy proxy) => Proxy = proxy;

/// <inheritdoc/>
public override string ToString() => Proxy.ToString();"#,
        prx_impl = interface_def.proxy_implementation_name()
    ).into()
}

/// The actual implementation of the proxy operation.
fn proxy_operation_impl(operation: &Operation, ast: &Ast) -> CodeBlock {
    let namespace = &operation.namespace();
    let operation_name = operation.escape_identifier();
    let async_operation_name = operation_name.clone() + "Async";
    let return_task = operation.return_task(namespace, false, ast);
    let is_oneway = operation.has_attribute("oneway");

    let parameters = operation.non_streamed_params(ast);
    let stream_return = operation.stream_return(ast);

    let invocation_parameter = escape_parameter_name(&operation.parameters(ast), "invocation");
    let cancel_parameter = escape_parameter_name(&operation.parameters(ast), "cancel");

    let sends_classes = operation.sends_classes(ast);
    let void_return = !operation.has_non_streamed_return(ast);

    let mut builder = FunctionBuilder::new(
        "public",
        &return_task,
        &async_operation_name,
        FunctionType::BlockBody,
    );
    builder.set_inherit_doc(true);
    builder.add_operation_parameters(operation, TypeContext::Outgoing, ast);

    let mut body = CodeBlock::new();

    if operation.compress_arguments() {
        body.writeln(&format!(
            "\
if {invocation}?.RequestFeatures.Get<IceRpc.Features.CompressPayload>() == null)
{{
{invocation}??= new IceRpc.Invocation();
{invocation}.RequestFeatures = IceRpc.FeatureCollectionExtensions.CompressPayload({invocation}.RequestFeatures);
}}
",
            invocation = invocation_parameter
        ));
    }

    let payload_encoding = if sends_classes {
        "IceRpc.Encoding.Ice11".to_owned()
    } else {
        body.writeln("var payloadEncoding = Proxy.GetIceEncoding();");
        "payloadEncoding".to_owned()
    };

    let mut invoke_args =
        vec![format!(r#""{}""#, operation.identifier()), payload_encoding.clone()];

    // The payload argument
    if parameters.is_empty() {
        invoke_args.push(format!("{}.CreateEmptyPayload()", payload_encoding));
    } else {
        let mut request_helper_args = vec![parameters.to_argument_tuple("")];

        if !sends_classes {
            request_helper_args.insert(0, payload_encoding.clone());
        }

        invoke_args.push(format!(
            "Request.{}({})",
            operation_name,
            request_helper_args.join(", ")
        ));
    }

    if void_return && stream_return.is_none() {
        invoke_args.push("_defaultIceDecoderFactories".to_owned());
    }

    // Stream parameter (if any)
    if let Some(stream_parameter) = operation.stream_parameter(ast) {
        let stream_parameter_name = stream_parameter.parameter_name();
        match stream_parameter.data_type.definition(ast) {
            Node::Primitive(_, b) if matches!(b, Primitive::Byte) => invoke_args.push(format!(
                "new IceRpc.Slice.ByteStreamParamSender({})",
                stream_parameter_name
            )),
            _ => invoke_args.push(format!(
                "\
new IceRpc.Slice.AsyncEnumerableStreamParamSender<{stream_type}>(
{stream_parameter},
{payload_encoding},
{encode_action}
)",
                stream_type = stream_parameter
                    .data_type
                    .clone_with_streamed(false)
                    .to_type_string(namespace, ast, TypeContext::Outgoing),
                stream_parameter = stream_parameter_name,
                payload_encoding = payload_encoding,
                encode_action = encode_action(
                    &stream_parameter.data_type.clone_with_streamed(false),
                    namespace,
                    true,
                    true,
                    ast
                )
                .indent()
            )),
        }
    } else {
        invoke_args.push("streamParamSender: null".to_owned());
    }

    if !void_return {
        invoke_args.push("Response.".to_owned() + &operation_name);
    } else if let Some(stream_return) = stream_return {
        let stream_return_func = match stream_return.data_type.definition(ast) {
            Node::Primitive(_, b) if matches!(b, Primitive::Byte) => {
                "streamParamReceiver!.ToByteStream()".to_owned()
            }
            _ => {
                format!(
                    "\
streamParamReceiver!.ToAsyncEnumerable<{stream_type}>(
response,
invoker,
response.GetIceDecoderFactory(_defaultIceDecoderFactories),
{decode_func})",
                    stream_type = stream_return
                        .data_type
                        .clone_with_streamed(false)
                        .to_type_string(namespace, ast, TypeContext::Incoming),
                    decode_func = decode_func(
                        &stream_return.data_type.clone_with_streamed(false),
                        namespace,
                        ast
                    )
                    .indent()
                )
            }
        };

        invoke_args.push(format!(
            "(response, invoker, streamParamReceiver) => {}",
            stream_return_func,
        ));
    }

    invoke_args.push("invocation".to_owned());

    if operation.is_idempotent {
        invoke_args.push("idempotent: true".to_owned());
    }

    if void_return && is_oneway {
        invoke_args.push("oneway: true".to_owned());
    }

    if stream_return.is_some() {
        invoke_args.push("returnStreamParamReceiver: true".to_owned());
    }

    invoke_args.push(format!("cancel: {}", cancel_parameter));

    body.writeln(&format!(
        "\
return Proxy.InvokeAsync(
    {});",
        args = invoke_args.join(",\n    ")
    ));

    builder.set_body(body);

    builder.build()
}

fn proxy_base_operation_impl(
    interface_def: &Interface,
    operation: &Operation,
    ast: &Ast,
) -> CodeBlock {
    let async_name = format!("{}Async", operation.escape_identifier());
    let return_task = operation.return_task(&operation.namespace(), false, ast);
    let mut operation_params = operation
        .parameters(ast)
        .iter()
        .map(|p| p.parameter_name())
        .collect::<Vec<_>>();

    operation_params.push(escape_parameter_name(
        &operation.parameters(ast),
        "invocation",
    ));
    operation_params.push(escape_parameter_name(&operation.parameters(ast), "cancel"));

    let base_interface = interface_def
        .all_bases(ast)
        .iter()
        .find(|base| {
            base.operations(ast)
                .iter()
                .any(|op| op.scope() == operation.scope())
        })
        .cloned()
        .unwrap();

    let mut builder = FunctionBuilder::new(
        "public",
        &return_task,
        &async_name,
        FunctionType::ExpressionBody,
    );
    builder.set_inherit_doc(true);
    builder.add_operation_parameters(operation, TypeContext::Outgoing, ast);

    builder.set_body(
        format!(
            "new {base_prx_impl}(Proxy).{async_name}({operation_params})",
            base_prx_impl = base_interface.proxy_implementation_name(),
            async_name = async_name,
            operation_params = operation_params.join(", ")
        )
        .into(),
    );

    builder.build()
}

fn proxy_interface_operations(interface_def: &Interface, ast: &Ast) -> CodeBlock {
    let mut code = CodeBlock::new();
    let namespace = &interface_def.namespace();
    let operations = interface_def.operations(ast);

    for operation in operations {
        // TODO: add returns doc comments (see C++) and obsolete attribute

        code.add_block(
            &FunctionBuilder::new(
                "",
                &operation.return_task(namespace, false, ast),
                &(operation.escape_identifier() + "Async"),
                FunctionType::Declaration,
            )
            .add_container_attributes(interface_def)
            .add_comment("summary", &doc_comment_message(operation))
            .add_operation_parameters(operation, TypeContext::Outgoing, ast)
            .build(),
        );
    }

    code
}

fn request_class(interface_def: &Interface, ast: &Ast) -> CodeBlock {
    let namespace = &interface_def.namespace();
    let operations = interface_def
        .operations(ast)
        .iter()
        .filter(|o| o.has_non_streamed_params(ast))
        .cloned()
        .collect::<Vec<_>>();

    if operations.is_empty() {
        return "".into();
    }

    let mut class_builder = ContainerBuilder::new("public static class", "Request");

    class_builder.add_comment(
        "summary",
        "Converts the arguments of each operation that takes arguments into a request payload.",
    );

    for operation in operations {
        let params: Vec<&Member> = operation.non_streamed_params(ast);

        if params.is_empty() {
            continue;
        }

        let sends_classes = operation.sends_classes(ast);

        let mut builder = FunctionBuilder::new(
            "public static",
            "global::System.ReadOnlyMemory<global::System.ReadOnlyMemory<byte>>",
            &operation.escape_identifier(),
            FunctionType::ExpressionBody,
        );

        builder.add_comment(
            "summary",
            &format!(
                "Creates the request payload for operation {}.",
                operation.identifier()
            ),
        );

        if !sends_classes {
            builder.add_parameter(
                "IceEncoding",
                "encoding",
                None,
                Some("The encoding of the payload."),
            );
        }

        if params.len() == 1 {
            builder.add_parameter(
                &params.to_tuple_type(namespace, ast, TypeContext::Outgoing),
                "arg",
                None,
                Some("The request argument."),
            );
        } else {
            builder.add_parameter(
                &format!(
                    "in {}",
                    params.to_tuple_type(namespace, ast, TypeContext::Outgoing)
                ),
                "args",
                None,
                Some("The request arguments."),
            );
        }

        if sends_classes {
            builder.add_comment("returns", "The payload encoded with encoding 1.1.");
        } else {
            builder.add_comment(
                "returns",
                r#"The payload encoded with <paramref name="encoding"/>."#,
            );
        }

        let body: CodeBlock = format!(
            "\
{encoding}.{name}(
    {args},
    {encode_action})",
            encoding = match sends_classes {
                false => "encoding",
                _ => "Ice11Encoding",
            },
            name = match params.len() {
                1 => "CreatePayloadFromSingleArg",
                _ => "CreatePayloadFromArgs",
            },
            args = if params.len() == 1 { "arg" } else { "in args" },
            encode_action = request_encode_action(operation, ast).indent()
        )
        .into();

        builder.set_body(body);

        class_builder.add_block(builder.build());
    }

    class_builder.build().into()
}

fn response_class(interface_def: &Interface, ast: &Ast) -> CodeBlock {
    let namespace = &interface_def.namespace();
    let operations = interface_def
        .operations(ast)
        .iter()
        .filter(|o| o.has_non_streamed_return(ast))
        .cloned()
        .collect::<Vec<_>>();

    if operations.is_empty() {
        return "".into();
    }

    let mut class_builder = ContainerBuilder::new("public static class", "Response");

    class_builder.add_comment(
        "summary",
    &format!(r#"Holds a <see cref="IceRpc.Gen.ResponseDecodeFunc{{T}}"/> for each non-void remote operation defined in <see cref="{}Prx"/>."#, interface_def.interface_name()));

    for operation in operations {
        let members = operation.return_members(ast);

        if members.is_empty() {
            continue;
        }

        let decoder = if operation.returns_classes(ast) {
            "response.GetIceDecoderFactory(_defaultIceDecoderFactories.Ice11DecoderFactory)"
        } else {
            "response.GetIceDecoderFactory(_defaultIceDecoderFactories)"
        };

        class_builder.add_block(format!(
            r#"
/// <summary>The <see cref="ResponseDecodeFunc{{T}}"/> for the return value type of operation {name}.</summary>
public static {return_type} {escaped_name}(IceRpc.IncomingResponse response, IceRpc.IInvoker? invoker, IceRpc.Slice.StreamParamReceiver? streamParamReceiver) =>
    response.ToReturnValue(
        invoker,
        {decoder},
        {response_decode_func});"#,
            name = operation.identifier(),
            escaped_name = operation.escape_identifier(),
            return_type = members.to_tuple_type( namespace, ast, TypeContext::Incoming),
            decoder = decoder,
            response_decode_func = response_decode_func(operation, ast).indent()
        ).into());
    }

    class_builder.build().into()
}

fn request_encode_action(operation: &Operation, ast: &Ast) -> CodeBlock {
    // TODO: scope
    let namespace = &operation.namespace();

    // We only want the non-streamed params
    let params: Vec<&Member> = operation.non_streamed_params(ast);

    // When the operation's parameter is a T? where T is an interface or a class, there is a
    // built-in encoder, so defaultEncodeAction is true.
    if params.len() == 1
        && get_bit_sequence_size(&params, ast) == 0
        && params.first().unwrap().tag.is_none()
    {
        encode_action(
            &params.first().unwrap().data_type,
            namespace,
            true,
            true,
            ast,
        )
    } else {
        format!(
            "\
(IceEncoder encoder, {_in}{param_type} value) =>
{{
    {encode}
}}",
            _in = if params.len() == 1 { "" } else { "in " },
            param_type = params.to_tuple_type(namespace, ast, TypeContext::Outgoing),
            encode = encode_operation(operation, false, ast).indent()
        )
        .into()
    }
}

fn response_decode_func(operation: &Operation, ast: &Ast) -> CodeBlock {
    let namespace = &operation.namespace();
    // vec of members
    let members = operation.return_members(ast);

    assert!(
        !members.is_empty()
            && (members.len() > 1 || !members.last().unwrap().data_type.is_streamed)
    );

    if members.len() == 1
        && get_bit_sequence_size(&members, ast) == 0
        && members.first().unwrap().tag.is_none()
    {
        decode_func(&members.first().unwrap().data_type, namespace, ast)
    } else {
        format!(
            "decoder => {{ {decode} }}",
            decode = decode_operation(operation, true, ast).indent()
        )
        .into()
    }
}
