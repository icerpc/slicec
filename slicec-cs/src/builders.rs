use crate::code_block::CodeBlock;
use crate::comments::CommentTag;

#[derive(Clone, Debug)]
pub struct ContainerBuilder {
    container_type: String,
    name: String,
    bases: Vec<String>,
    contents: Vec<CodeBlock>,
    comments: Vec<CommentTag>,
}

impl ContainerBuilder {
    pub fn new(container_type: &str, name: &str) -> Self {
        Self {
            container_type: container_type.to_owned(),
            name: name.to_owned(),
            bases: vec![],
            contents: vec![],
            comments: vec![],
        }
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
        format!(
            "\
{comments}
{container_type} {name}{bases}
{{
    {contents}
}}",
            comments = self
                .comments
                .iter()
                .map(|c| c.to_string())
                .collect::<Vec<_>>()
                .join("\n\n\n"),
            container_type = self.container_type,
            name = self.name,
            bases = if self.bases.is_empty() {
                "".to_string()
            } else {
                format!(" : {bases}", bases = self.bases.join(", "))
            },
            contents = self
                .contents
                .iter()
                .map(|c| c.clone().indent().to_string())
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>()
                .join("\n\n    "),
        )
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
            base_arguments: Vec::new(),
            use_expression_body: false,
        }
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

    pub fn use_expression_body(&mut self, use_expression_body: bool) -> &mut Self {
        self.use_expression_body = use_expression_body;
        self
    }

    pub fn build(&mut self) -> CodeBlock {
        let body = if self.use_expression_body {
            format!("=>\n    {};", self.body.indent())
        } else {
            format!("\n{{\n    {}\n}}", self.body.indent())
        };

        let comments: CodeBlock = self
            .comments
            .iter()
            .map(|c| c.to_string())
            .collect::<Vec<_>>()
            .join("\n")
            .into();

        format!(
            "\
{comments}
{access}{return_type:^return_width$}{name}({parameters}){base} {body}",
            comments = comments,
            access = self.access,
            return_type = self.return_type,
            return_width = if self.return_type.is_empty() {
                1
            } else {
                self.return_type.len() + 2
            },
            name = self.name,
            parameters = self.parameters.join(", "),
            base = match self.base_arguments.len() {
                0 => "".to_string(),
                _ => format!("\n    : base({})", self.base_arguments.join(", ")),
            },
            body = body
        )
        .into()
    }
}
