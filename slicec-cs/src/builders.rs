use slice::grammar::{Class, NamedSymbol};

use crate::attributes::{compact_id_attribute, custom_attributes, type_id_attribute};
use crate::code_block::CodeBlock;
use crate::comments::CommentTag;

trait Builder {
    fn build(&self) -> String;
}

#[derive(Clone, Debug)]
pub struct ContainerBuilder {
    container_type: String,
    name: String,
    bases: Vec<String>,
    contents: Vec<CodeBlock>,
    attributes: Vec<String>,
    comments: Vec<CommentTag>,
}

impl ContainerBuilder {
    pub fn new(container_type: &str, name: &str) -> Self {
        Self {
            container_type: container_type.to_owned(),
            name: name.to_owned(),
            bases: vec![],
            contents: vec![],
            attributes: vec![],
            comments: vec![],
        }
    }

    pub fn add_type_id_attribute(&mut self, named_symbol: &dyn NamedSymbol) -> &mut Self {
        self.add_attribute(&type_id_attribute(named_symbol));
        self
    }

    pub fn add_compact_type_id_attribute(&mut self, class_def: &Class) -> &mut Self {
        if let Some(attribute) = compact_id_attribute(class_def) {
            self.add_attribute(&attribute);
        }
        self
    }

    pub fn add_custom_attributes(&mut self, named_symbol: &dyn NamedSymbol) -> &mut Self {
        for attribute in custom_attributes(named_symbol) {
            self.add_attribute(&attribute);
        }
        self
    }

    pub fn add_attribute(&mut self, attribute: &str) -> &mut Self {
        self.attributes.push(attribute.to_owned());
        self
    }

    pub fn add_base(&mut self, base: String) -> &mut Self {
        self.bases.push(base);
        self
    }

    pub fn add_bases(&mut self, bases: &[String]) -> &mut Self {
        self.bases.extend_from_slice(bases);
        self
    }

    pub fn add_block(&mut self, content: CodeBlock) -> &mut Self {
        self.contents.push(content);
        self
    }

    pub fn add_comment(&mut self, tag: &str, content: &str) -> &mut Self {
        self.comments.push(CommentTag::new(tag, "", "", content));
        self
    }

    pub fn build(&self) -> String {
        let mut code = CodeBlock::new();

        for comment in &self.comments {
            code.writeln(&comment.to_string());
        }

        for attribute in &self.attributes {
            code.writeln(&format!("[{}]", attribute));
        }

        writeln!(
            code,
            "{container_type} {name}{bases}",
            container_type = self.container_type,
            name = self.name,
            bases = if self.bases.is_empty() {
                "".to_string()
            } else {
                format!(" : {bases}", bases = self.bases.join(", "))
            },
        );

        let mut body_content: CodeBlock = self.contents.iter().cloned().collect();

        if body_content.is_empty() {
            code.writeln("{{\n}}");
        } else {
            writeln!(code, "{{\n    {body}\n}}", body = body_content.indent());
        }

        code.to_string()
    }
}

#[derive(Clone, Debug)]
pub struct FunctionBuilder {
    access: String,
    name: String,
    return_type: String,
    parameters: Vec<String>,
    body: CodeBlock,
    base_arguments: Vec<String>,
    comments: Vec<CommentTag>,
    attributes: Vec<String>,
    use_expression_body: bool,
}

impl FunctionBuilder {
    pub fn new(access: &str, return_type: &str, name: &str) -> FunctionBuilder {
        FunctionBuilder {
            parameters: Vec::new(),
            access: String::from(access),
            name: String::from(name),
            return_type: String::from(return_type),
            body: CodeBlock::new(),
            comments: Vec::new(),
            attributes: Vec::new(),
            base_arguments: Vec::new(),
            use_expression_body: false,
        }
    }

    pub fn add_attribute(&mut self, attribute: &str) -> &mut Self {
        self.attributes.push(attribute.to_owned());
        self
    }

    pub fn add_comment(&mut self, tag: &str, content: &str) -> &mut Self {
        self.comments.push(CommentTag::new(tag, "", "", content));
        self
    }

    fn add_comment_with_attribute(
        &mut self,
        tag: &str,
        attribute_name: &str,
        attribute_value: &str,
        content: &str,
    ) -> &mut Self {
        self.comments.push(CommentTag::new(
            tag,
            attribute_name,
            attribute_value,
            content,
        ));
        self
    }

    pub fn add_parameter(
        &mut self,
        param_type: &str,
        param_name: &str,
        default_value: Option<&str>,
        doc_comment: &str,
    ) -> &mut Self {
        self.parameters.push(format!(
            "{param_type} {param_name}{default_value}",
            param_type = param_type,
            param_name = param_name,
            default_value = match default_value {
                Some(value) => format!(" = {}", value),
                None => "".to_string(),
            }
        ));

        self.add_comment_with_attribute("param", "name", param_name, doc_comment)
    }

    pub fn add_parameters(&mut self, parameters: &[String]) -> &mut Self {
        for p in parameters {
            self.parameters.push(p.clone());
        }
        self
    }

    pub fn add_base_argument(&mut self, argument: &str) -> &mut Self {
        self.base_arguments.push(argument.to_owned());
        self
    }

    pub fn add_base_arguments(&mut self, arguments: &[String]) -> &mut Self {
        for arg in arguments {
            self.base_arguments.push(arg.to_owned());
        }
        self
    }

    pub fn set_body(&mut self, body: CodeBlock) -> &mut Self {
        self.body = body;
        self
    }

    pub fn add_never_editor_browsable_attribute(&mut self) -> &mut Self {
        self.add_attribute(
            "global::System.ComponentModel.EditorBrowsable(global::System.ComponentModel.EditorBrowsableState.Never)");
        self
    }

    pub fn use_expression_body(&mut self, use_expression_body: bool) -> &mut Self {
        self.use_expression_body = use_expression_body;
        self
    }

    pub fn build(&mut self) -> CodeBlock {
        let mut code = CodeBlock::new();

        for comment in &self.comments {
            code.writeln(&comment.to_string());
        }

        for attribute in &self.attributes {
            code.writeln(&format!("[{}]", attribute));
        }

        write!(
            code,
            "{access}{return_type}{name}({parameters}){base}",
            access = self.access,
            return_type = if self.return_type.is_empty() {
                " ".to_owned()
            } else {
                format!(" {} ", self.return_type)
            },
            name = self.name,
            parameters = self.parameters.join(", "),
            base = match self.base_arguments.as_slice() {
                [] => "".to_string(),
                _ => format!("\n    : base({})", self.base_arguments.join(", ")),
            }
        );

        if self.body.is_empty() {
            if self.use_expression_body {
                code.writeln("=> {{}};")
            } else {
                code.writeln("\n{\n}");
            }
        } else if self.use_expression_body {
            writeln!(code, "=>\n    {};", self.body.indent());
        } else {
            writeln!(code, "\n{{\n    {body}\n}}", body = self.body.indent());
        }

        code
    }
}
