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

    pub fn add_content(&mut self, content: CodeBlock) -> &mut Self {
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
    parameters: Vec<(String, String)>,

    body: CodeBlock,

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
        doc_comment: &str,
    ) -> &mut Self {
        self.parameters
            .push((String::from(param_type), String::from(param_name)));
        self.add_comment_with_attribute("param", "name", param_name, doc_comment)
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
{access} {return_type} {name}({parameters}) {body}",
            comments = comments,
            access = self.access,
            return_type = self.return_type,
            name = self.name,
            parameters = self
                .parameters
                .iter()
                .map(|(param_type, param_name)| format!("{} {}", param_type, param_name))
                .collect::<Vec<_>>()
                .join(", "),
            body = body
        )
        .into()
    }
}
