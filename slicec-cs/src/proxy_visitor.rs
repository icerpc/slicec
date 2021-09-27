// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::builders::{ContainerBuilder, FunctionBuilder};
use crate::code_block::CodeBlock;
use crate::comments::*;
use crate::cs_util::*;
use crate::decoding::*;
use crate::encoding::*;
use slice::ast::Ast;
use slice::grammar::*;
use slice::util::*;
use slice::visitor::Visitor;
use slice::writer::Writer;

pub struct ProxyVisitor<'a> {
    output: &'a mut Writer,
}

impl<'a> ProxyVisitor<'a> {
    pub fn new(output: &'a mut Writer) -> ProxyVisitor<'a> {
        ProxyVisitor { output }
    }
}

impl Visitor for ProxyVisitor<'_> {
    fn visit_module_start(&mut self, module_def: &Module, _: usize, _: &Ast) {
        // write_comment(&mut self.output, module_def);
        let content = format!("\nnamespace {}\n{{", module_def.identifier());
        self.output.write(&content);
        self.output.indent_by(4);
    }

    fn visit_module_end(&mut self, _: &Module, _: usize, _: &Ast) {
        self.output.clear_line_separator();
        self.output.indent_by(-4);
        self.output.write("\n}");
        self.output.write_line_separator();
    }

    fn visit_interface_start(&mut self, interface_def: &Interface, _: usize, ast: &Ast) {
        let prx_interface = format!("{}Prx", interface_name(interface_def)); // IFooPrx
        let prx_impl: String = prx_interface.chars().skip(1).collect(); // IFooPrx -> FooPrx

        let all_bases: Vec<&Interface> = interface_def.all_bases(ast);
        let bases: Vec<&Interface> = interface_def.bases(ast);

        let mut prx_impl_bases: Vec<String> = vec![prx_interface.clone(), "IPrx".to_owned()];

        let mut all_base_impl: Vec<String> = all_bases
            .iter()
            .map(|b| interface_name(b).chars().skip(1).collect::<String>() + "Prx")
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
        let prx_bases: Vec<String> = bases
            .into_iter()
            .map(|b| escape_scoped_identifier(b, CaseStyle::Pascal, interface_def.scope()))
            .collect();

        // writeProxyDocComment(p, getDeprecateReason(p));
        // emitTypeIdAttribute(p->scoped());
        // emitCustomAttributes(p);
        // TODO: above doc comments and attributes

        let interface = ContainerBuilder::new("public partial interface", &prx_interface)
            .add_comment("summary", "///TODO:")
            .add_bases(&prx_bases)
            .add_content(prx_operations(interface_def, ast))
            .build();

        // TODO: add type id attribute and custom attribtues
        let mut proxy_impl_builder =
            ContainerBuilder::new("public readonly partial record struct", &prx_impl);

        proxy_impl_builder.add_bases(&prx_bases)
            .add_comment("summary", &format!(r#"Typed proxy record struct. It implements <see cref="{}"/> by sending requests to a remote IceRPC service."#, prx_interface))
            .add_content(request_class(interface_def, ast))
            .add_content(response_class(interface_def, ast));

        proxy_impl_builder.add_content(format!(
            r#"
/// <summary>The default path for services that implement Slice interface <c>{interface_name}</c>.</summary>
public static readonly string DefaultPath = typeof({prx_impl}).GetDefaultPath();

private static readonly DefaultIceDecoderFactories _defaultIceDecoderFactories = new (typeof({prx_impl}).Assembly);

/// <summary>The proxy to the remote service.</summary>
public IceRpc.Proxy Proxy {{ get; init; }}"#,
            interface_name = interface_def.identifier(),
            prx_impl = interface_name(interface_def)
        ).into());

        for base_impl in all_base_impl {
            proxy_impl_builder.add_content(
                format!(
                    r#"
/// <summary>Implicit conversion to <see cref="{base_impl}"/>.</summary>
public static implicit operator {base_impl}({prx_impl} prx) => new (prx.Proxy);"#,
                    base_impl = base_impl,
                    prx_impl = prx_impl
                )
                .into(),
            );
        }

        let static_methods = format!(
            r#"/// <summary>Creates a new <see=cref="{prx_impl}"/> from the give connection and path.</summary>
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
public static {prx_impl} FromPath(string path, IceRpc.Protocol protocol = IceRpc.Protocol.Ice2) =>
    new(IceRpc.Proxy.FromPath(path, protocol));

/// <summary>Creates a new <see cref="{prx_impl}"/> from a string and invoker.</summary>
/// <param name="s">The string representation of the proxy.</param>
/// <param name="invoker">The invoker of the new proxy.</param>
/// <returns>The new proxy</returns>
/// <exception cref="global::System.FormatException"><c>s</c> does not contain a valid string representation of a proxy.</exception>
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
public override string ToString() => Proxy.ToString();

        "#,
            prx_impl = interface_name(interface_def)
        );

        proxy_impl_builder.add_content(static_methods.into());

        if add_service_prx {
            let f = format!(
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
            );
            proxy_impl_builder.add_content(f.into());
        }

        for operation in interface_def.all_base_operations(ast) {
            let async_name = escape_identifier(operation, CaseStyle::Pascal) + "Async";
            let return_task = return_type_to_string(
                &operation.return_members(ast),
                interface_def.scope(),
                ast,
                TypeContext::Outgoing,
            );
            let invocation_params = get_invocation_params(operation, ast);

            let mut proxy_params = operation
                .parameters(ast)
                .iter()
                .map(|p| member_name(p, "", true))
                .collect::<Vec<_>>();
            proxy_params.push(escape_member_name(&operation.parameters(ast), "invocation"));
            proxy_params.push(escape_member_name(&operation.parameters(ast), "cancel"));

            // TODO: base interface
            // InterfaceDefPtr baseInterface = InterfaceDefPtr::dynamicCast(operation->container());
            // string basePrxImpl = getUnqualified(getNamespace(baseInterface) + "." +
            // interfaceName(baseInterface).substr(1) + "Prx", ns);

            format!(
                "\
/// <inheritdoc/>
public {return_task} {async_name}({invocation_params}) =>
    new {base_prx_impl}(Proxy).{async_name}({proxy_params})",
                return_task = return_task,
                async_name = async_name,
                invocation_params = invocation_params.join(", "),
                base_prx_impl = "TODO",
                proxy_params = proxy_params.join(", ")
            );
        }

        // Generate abstract methods and documentation
        writeln!(
            self.output,
            "\n{interface}\n\n{proxy_impl}",
            interface = interface,
            proxy_impl = proxy_impl_builder.build()
        );
    }
}

pub fn interface_name(interface_def: &Interface) -> String {
    let identifier = fix_case(interface_def.identifier(), CaseStyle::Pascal);
    let mut chars = identifier.chars();

    // Check if the interface already follows the 'I' prefix convention.
    if identifier.chars().count() > 2
        && chars.next().unwrap() == 'I'
        && chars.next().unwrap().is_uppercase()
    {
        identifier.to_owned()
    } else {
        format!("I{}", identifier)
    }
}

fn prx_operations(interface_def: &Interface, ast: &Ast) -> CodeBlock {
    let mut code = CodeBlock::new();

    let operations = interface_def.operations(ast);

    for operation in operations {
        let operation_name = escape_identifier(operation, CaseStyle::Pascal);
        let async_name = operation_name + "Async";

        let deprecate_reason = match &operation.comment {
            Some(comment) if comment.deprecate_reason.is_some() => {
                format!(
                    r#"[global::System::Obsolete("{}")]"#,
                    comment.deprecate_reason.as_ref().unwrap()
                )
            }
            _ => "".to_owned(),
        };

        writeln!(
            code,
            "{doc_comment}{deprecate_reason}\n{return} {name}({params});\n",
            doc_comment = operation_doc_comment(operation, false, ast),
            deprecate_reason = deprecate_reason,
            return = operation_return_task(operation, false, ast),
            name = async_name,
            params = get_invocation_params(operation, ast).join(", ")
        )
    }

    code
}

pub fn operation_return_task(operation: &Operation, is_dispatch: bool, ast: &Ast) -> String {
    let return_members = operation.return_members(ast);
    if return_members.is_empty() {
        if is_dispatch {
            "global::System.Threading.Tasks.ValueTask".to_owned()
        } else {
            "global::System.Threading.Tasks.Task".to_owned()
        }
    } else {
        let return_type = operation_return_type(operation, is_dispatch, ast);
        if is_dispatch {
            format!("global::System.Threading.Tasks.ValueTask<{}>", return_type)
        } else {
            format!("global::System.Threading.Tasks.Task<{}>", return_type)
        }
    }
}

pub fn operation_return_type(operation: &Operation, is_dispatch: bool, ast: &Ast) -> String {
    let return_type = &operation.return_type;

    let has_marshaled_result = false; // TODO: do we still want to keep this?

    if is_dispatch && has_marshaled_result {
        return "".to_owned();
    }

    let return_members = operation.return_members(ast);
    match return_members.len() {
        0 => "void".to_owned(),
        1 => param_type_to_string(&return_members[0].data_type, is_dispatch, ast),
        _ => to_tuple_type(&return_members, is_dispatch, ast),
    }
}

pub fn to_tuple_type(members: &[&Member], is_dispatch: bool, ast: &Ast) -> String {
    match members.len() {
        0 => panic!("tuple type with no members"),
        1 => param_type_to_string(&members[0].data_type, is_dispatch, ast),
        _ => format!(
            "({})",
            members
                .into_iter()
                .map(|m| param_type_to_string(&m.data_type, is_dispatch, ast)
                    + " "
                    + &field_name(&m, ""))
                .collect::<Vec<String>>()
                .join(", ")
        ),
    }
}

pub fn to_tuple_return(members: &[&Member], prefix: &str, ast: &Ast) -> String {
    match members.len() {
        0 => panic!("tuple type with no members"),
        1 => member_name(&members[0], prefix, true),
        _ => format!(
            "({})",
            members
                .iter()
                .map(|m| member_name(m, prefix, true))
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}

// TODO: maybe rename operation_param_to_string
pub fn param_type_to_string(type_ref: &TypeRef, is_dispatch: bool, ast: &Ast) -> String {
    let context = if is_dispatch {
        TypeContext::Incoming
    } else {
        TypeContext::Outgoing
    };

    type_to_string(type_ref, type_ref.scope(), ast, context)
}

pub fn get_invocation_params(operation: &Operation, ast: &Ast) -> Vec<String> {
    let mut params = Vec::new();

    let operation_parameters = operation.parameters(ast);

    for p in operation.parameters(ast) {
        params.push(format!(
            "{attributes}{param_type} {param_name}",
            attributes = "", // TOOD: getParamAttributes(p)
            param_type = type_to_string(&p.data_type, p.scope(), ast, TypeContext::Outgoing),
            param_name = member_name(p, "", true)
        ))
    }

    params.push(format!(
        "IceRpc.Invocation? {} = null",
        escape_member_name(&operation_parameters, "invocation")
    ));
    params.push(format!(
        "global::System.Threading.CancellationToken {} = default",
        escape_member_name(&operation_parameters, "cancel")
    ));

    params
}

fn request_class(interface_def: &Interface, ast: &Ast) -> CodeBlock {
    let operations = interface_def.operations(ast);

    if !operations.iter().any(|o| o.has_non_streamed_params(ast)) {
        return "".into();
    }

    let mut class_builder = ContainerBuilder::new("public static class", "Request");

    class_builder.add_comment(
        "summary",
        "Converts the arguments of each operation that takes arguments into a request payload.",
    );

    for operation in operations {
        let params: Vec<&Member> = operation.non_streamed_params(ast);

        if params.len() == 0 {
            continue;
        }

        let sends_classes = operation.sends_classes(ast);

        let mut builder = FunctionBuilder::new(
            "public static",
            "global::System.ReadOnlyMemory<global::System.ReadOnlyMemory<byte>>",
            &escape_identifier(operation, CaseStyle::Pascal),
        );

        builder.add_comment(
            "summary",
            &format!(
                "Creates the request payload for operation {}.",
                operation.identifier()
            ),
        );

        if !sends_classes {
            builder.add_parameter("IceEncoding", "encoding", "The encoding of the payload.");
        }

        if params.len() == 1 {
            builder.add_parameter(
                &to_tuple_type(&params, true, ast),
                "arg",
                "The request argument.",
            );
        } else {
            builder.add_parameter(
                &format!("in {}", to_tuple_type(&params, true, ast)),
                "args",
                "The request arguments.",
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
IceRpc.Payload.{name}(
    {args},
    {encode_action},
    {class_format})",
            name = if params.len() == 1 {
                "CreatePayloadFromSingleArg"
            } else {
                "CreatePayloadFromArgs"
            },
            args = if params.len() == 1 { "arg" } else { "in args" },
            encode_action = request_encode_action(operation, ast).indent(),
            class_format = operation_format_type_to_string(operation)
        )
        .into();

        builder.use_expression_body(true).set_body(body);

        class_builder.add_content(builder.build());
    }

    class_builder.build().into()
}

fn response_class(interface_def: &Interface, ast: &Ast) -> CodeBlock {
    let operations = interface_def.operations(ast);

    if !operations.iter().any(|o| o.has_non_streamed_return(ast)) {
        return "".into();
    }

    let mut class_builder = ContainerBuilder::new("public static class", "Response");

    class_builder.add_comment(
        "summary",
    &format!(r#"Holds a <see cref="IceRpc.Gen.ResponseDecodeFunc{{T}}"/> for each non-void remote operation defined in <see cref="{}Prx"/>."#, interface_name(interface_def)));

    for operation in operations {
        let members = operation.return_members(ast);

        if members.len() == 0 {
            continue;
        }

        let decoder = if operation.returns_classes(ast) {
            "response.GetIceDecoderFactory(_defaultIceDecoderFactories.Ice11DecoderFactory)"
        } else {
            "response.GetIceDecoderFactory(_defaultIceDecoderFactories)"
        };

        let mut builder = FunctionBuilder::new(
            "public static",
            &to_tuple_type(&members, false, ast),
            &escape_identifier(operation, CaseStyle::Pascal),
        );

        builder
        .add_comment("summary", &format!(r#"The <see cref="ResponseDecodeFunc{{T}}"/> for the return value {} type of operation"#, operation.identifier()))
        .add_parameter("IceRpc.IncomingResponse", "response", "")
        .add_parameter("IceRpc.IInvoker?", "invoker", "")
        .use_expression_body(true)
        .set_body(format!("
response.ToReturnValue(
    invoker,
    {decoder},
    {response_decode_func})
        ",
        decoder= decoder,
        response_decode_func = response_decode_func(operation, ast)).into());

        class_builder.add_content(builder.build());
    }

    class_builder.build().into()
}

fn request_encode_action(operation: &Operation, ast: &Ast) -> CodeBlock {
    // TODO: scope
    let ns = get_namespace(operation);

    // We only want the non-streamed params
    let params: Vec<&Member> = operation.non_streamed_params(ast);

    // When the operation's parameter is a T? where T is an interface or a class, there is a
    // built-in encoder, so defaultEncodeAction is true.
    if params.len() == 1
        && get_bit_sequence_size(&params, ast) == 0
        && params.first().unwrap().tag.is_none()
    {
        encode_action(&params.first().unwrap().data_type, &ns, true, true, ast)
    } else {
        format!(
            "\
(IceRpc.IceEncoder encoder, {_in}{param_type} value) =>
{{
    {encode}
}}",
            _in = if params.len() == 1 { "" } else { "in " },
            param_type = to_tuple_type(&params, true, ast),
            encode = encode_operation(operation, false, ast).indent()
        )
        .into()
    }
}

fn response_decode_func(operation: &Operation, ast: &Ast) -> CodeBlock {
    let ns = get_namespace(operation);
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
        decode_func(&members.first().unwrap().data_type, &ns, ast)
    } else {
        format!(
            "decoder => {{ {decode} }}",
            decode = decode_operation(operation, true, ast).indent()
        )
        .into()
    }
}
