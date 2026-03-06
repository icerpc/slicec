// Copyright (c) ZeroC, Inc.

// Pull in all the mapped compiler-definition types.
use crate::definition_types::*;

// Pull in the core compiler types, using aliases to disambiguate.
// Any type that starts with 'Compiler' is a compiler type, not a mapped type.
use slicec::grammar::Attribute as CompilerAttribute;
use slicec::grammar::CustomType as CompilerCustomType;
use slicec::grammar::Definition as CompilerDefinition;
use slicec::grammar::Dictionary as CompilerDictionary;
use slicec::grammar::DocComment as CompilerDocComment;
use slicec::grammar::Enum as CompilerEnum;
use slicec::grammar::Enumerator as CompilerEnumerator;
use slicec::grammar::Field as CompilerField;
use slicec::grammar::Identifier as CompilerIdentifier;
use slicec::grammar::Interface as CompilerInterface;
use slicec::grammar::MessageComponent as CompilerMessageComponent;
use slicec::grammar::Operation as CompilerOperation;
use slicec::grammar::Parameter as CompilerParameter;
use slicec::grammar::ResultType as CompilerResultType;
use slicec::grammar::Sequence as CompilerSequence;
use slicec::grammar::Struct as CompilerStruct;
use slicec::grammar::Types as CompilerTypes;
use slicec::grammar::TypeAlias as CompilerTypeAlias;
use slicec::grammar::TypeRef as CompilerTypeRef;
use slicec::grammar::attributes::Allow;
use slicec::grammar::attributes::Compress;
use slicec::grammar::attributes::Deprecated;
use slicec::grammar::attributes::Oneway;
use slicec::grammar::attributes::SlicedFormat;
use slicec::grammar::attributes::Unparsed;
use slicec::slice_file::SliceFile as CompilerSliceFile;

// Pull in traits from 'slicec' so we can call their functions.
use slicec::grammar::Attributable;
use slicec::grammar::Commentable;
use slicec::grammar::Contained;
use slicec::grammar::Entity;
use slicec::grammar::Member;
use slicec::grammar::NamedSymbol;
use slicec::grammar::Type;







fn get_entity_info_for(element: &impl Commentable) -> EntityInfo {
    EntityInfo {
        identifier: element.identifier().to_owned(),
        attributes: get_attributes_from(element.attributes()),
        comment: element.comment().map(Into::into),
    }
}

fn get_doc_comment_for_parameter(parameter: &CompilerParameter) -> Option<DocComment> {
    let operation_comment = parameter.parent().comment()?;

    // We get the parameter's doc-comment in 3 steps:
    // 1) Try to find a matching '@param' tag on the operation's doc-comment.
    // 2) If one was present, extract just it's `Message` field, and convert it to the mapped type.
    // 3) Construct a mapped `DocComment` which contains the mapped message.
    operation_comment.params.iter()
        .find(|param_tag| param_tag.identifier.value == parameter.identifier())
        .map(|param_tag| param_tag.message.value.iter().map(Into::into).collect())
        .map(|message| DocComment { overview: message, see_tags: Vec::new() })
}

fn convert_doc_comment_link(link_result: Result<&dyn Entity, &CompilerIdentifier>) -> String {
    match link_result {
        Ok(entity) => entity.parser_scoped_identifier(),
        Err(identifier) => identifier.value.clone(),
    }
}

fn get_attributes_from(attributes: Vec<&CompilerAttribute>) -> Vec<Attribute> {
    attributes.into_iter().map(|attribute| Attribute {
        directive: attribute.kind.directive().to_owned(),
        args: get_attribute_args(attribute),
    })
    .collect()
}

// TODO this is a temporary hack because we know all the possible attributes.
// The `Attribute` API doesn't offer a way to convert parsed-arguments back to a string.
// And this entire API will be rewritten after porting slicec-cs, so no point changing it now.
fn get_attribute_args(attribute: &CompilerAttribute) -> Vec<String> {
    if let Some(unparsed) = attribute.downcast::<Unparsed>() {
        return unparsed.args.clone();
    }

    if let Some(allow) = attribute.downcast::<Allow>() {
        return allow.allowed_lints.clone();
    }

    if let Some(compress) = attribute.downcast::<Compress>() {
        let mut args = Vec::new();
        if compress.compress_args {
            args.push("Args".to_owned());
        }
        if compress.compress_return {
            args.push("Return".to_owned());
        }
        return args;
    }

    if let Some(deprecated) = attribute.downcast::<Deprecated>() {
        return deprecated.reason.iter().cloned().collect();
    }

    if let Some(_) = attribute.downcast::<Oneway>() {
        return Vec::new();
    }

    if let Some(sliced_format) = attribute.downcast::<SlicedFormat>() {
        let mut args = Vec::new();
        if sliced_format.sliced_args {
            args.push("Args".to_owned());
        }
        if sliced_format.sliced_return {
            args.push("Return".to_owned());
        }
        return args;
    }

    panic!("Impossible attribute encountered")
}








impl From<&CompilerSliceFile> for SliceFile {
    fn from(slice_file: &CompilerSliceFile) -> Self {
        // Convert the slice_file's module declaration.
        let module = slice_file.module.as_ref().unwrap().borrow();
        let converted_module = Module {
            identifier: module.identifier().to_owned(),
            attributes: get_attributes_from(module.attributes()),
        };

        // Return a converted slice file.
        SliceFile {
            path: slice_file.relative_path.clone(),
            module_declaration: converted_module,
            attributes: get_attributes_from(slice_file.attributes()),
            contents: SliceFileContentsConverter::convert(&slice_file.contents),
        }
    }
}

impl From<&CompilerDocComment> for DocComment {
    fn from(doc_comment: &CompilerDocComment) -> Self {
        let overview = doc_comment.overview.as_ref().map(|message| {
            message.value.iter().map(Into::into)
        });

        let see_tags = doc_comment.see.iter().map(|tag| {
            convert_doc_comment_link(tag.linked_entity())
        });

        DocComment {
            overview: overview.map_or(Vec::new(), |v| v.collect()),
            see_tags: see_tags.collect(),
        }
    }
}

impl From<&CompilerMessageComponent> for MessageComponent {
    fn from(component: &CompilerMessageComponent) -> Self {
        match component {
            CompilerMessageComponent::Text(text) => MessageComponent::Text(text.clone()),
            CompilerMessageComponent::Link(tag) => {
                MessageComponent::Link(convert_doc_comment_link(tag.linked_entity()))
            }
        }
    }
}












#[derive(Debug)]
pub struct SliceFileContentsConverter {
    converted_contents: Vec<Symbol>,
}

impl SliceFileContentsConverter {
    pub fn convert(contents: &[CompilerDefinition]) -> Vec<Symbol> {
        // Create a new converter.
        let mut converter = SliceFileContentsConverter { converted_contents: Vec::new() };

        // Iterate through the provided file's contents, and convert each of it's top-level definitions.
        for definition in contents {
            let converted = match definition {
                CompilerDefinition::Struct(v) => Symbol::Struct(converter.convert_struct(v.borrow())),
                CompilerDefinition::Interface(v) => Symbol::Interface(converter.convert_interface(v.borrow())),
                CompilerDefinition::Enum(v) => Symbol::Enum(converter.convert_enum(v.borrow())),
                CompilerDefinition::CustomType(v) => Symbol::CustomType(converter.convert_custom_type(v.borrow())),
                CompilerDefinition::TypeAlias(v) => Symbol::TypeAlias(converter.convert_type_alias(v.borrow())),
                _ => panic!("TODO: remove classes and exceptions"),
            };
            converter.converted_contents.push(converted);
        }

        // Return all the converted elements, consuming the converter.
        converter.converted_contents
    }

    fn convert_type_ref(&mut self, type_ref: &CompilerTypeRef) -> TypeRef {
        TypeRef {
            type_id: self.get_type_id_for(type_ref),
            is_optional: type_ref.is_optional,
            type_attributes: get_attributes_from(type_ref.attributes()),
        }
    }

    fn convert_struct(&mut self, struct_def: &CompilerStruct) -> Struct {
        Struct {
            entity_info: get_entity_info_for(struct_def),
            is_compact: struct_def.is_compact,
            fields: struct_def.fields().into_iter().map(|e| self.convert_field(e)).collect(),
        }
    }

    fn convert_field(&mut self, field: &CompilerField) -> Field {
        Field {
            entity_info: get_entity_info_for(field),
            tag: field.tag.as_ref().map(|integer| integer.value as i32),
            data_type: self.convert_type_ref(field.data_type()),
        }
    }

    fn convert_interface(&mut self, interface_def: &CompilerInterface) -> Interface {
        let bases = interface_def.base_interfaces();

        Interface {
            entity_info: get_entity_info_for(interface_def),
            bases: bases.into_iter().map(|i| i.module_scoped_identifier()).collect(),
            operations: interface_def.operations().into_iter().map(|e| self.convert_operation(e)).collect(),
        }
    }

    fn convert_operation(&mut self, operation: &CompilerOperation) -> Operation {
        Operation {
            entity_info: get_entity_info_for(operation),
            is_idempotent: operation.is_idempotent,
            parameters: operation.parameters().into_iter().map(|e| self.convert_parameter(e)).collect(),
            has_streamed_parameter: operation.streamed_parameter().is_some(),
            return_type: operation.return_members().into_iter().map(|e| self.convert_parameter(e)).collect(),
            has_streamed_return: operation.streamed_return_member().is_some(), 
        }
    }

    fn convert_parameter(&mut self, parameter: &CompilerParameter) -> Field {
        let parameter_info = EntityInfo {
            identifier: parameter.identifier().to_owned(),
            attributes: get_attributes_from(parameter.attributes()),
            comment: get_doc_comment_for_parameter(parameter),
        };

        Field {
            entity_info: parameter_info,
            tag: parameter.tag.as_ref().map(|integer| integer.value as i32),
            data_type: self.convert_type_ref(parameter.data_type()),
        }
    }

    fn convert_enum(&mut self, enum_def: &CompilerEnum) -> Enum {
        Enum {
            entity_info: get_entity_info_for(enum_def),
            is_compact: enum_def.is_compact,
            is_unchecked: enum_def.is_unchecked,
            underlying: enum_def.underlying.as_ref().map(|type_ref| type_ref.type_string()),
            enumerators: enum_def.enumerators().into_iter().map(|e| self.convert_enumerator(e)).collect(), 
        }
    }

    fn convert_enumerator(&mut self, enumerator: &CompilerEnumerator) -> Enumerator {
        let entity_info = get_entity_info_for(enumerator);
        let raw_value = enumerator.value();
        let value = Discriminant {
            absolute_value: raw_value.abs() as u64,
            is_positive: raw_value.is_positive(),
        };
        let fields = enumerator.fields().into_iter().map(|e| self.convert_field(e)).collect();

        Enumerator { entity_info, value, fields }
    }

    fn convert_custom_type(&mut self, custom_type: &CompilerCustomType) -> CustomType {
        CustomType { entity_info: get_entity_info_for(custom_type) }
    }

    fn convert_type_alias(&mut self, type_alias: &CompilerTypeAlias) -> TypeAlias {
        TypeAlias {
            entity_info: get_entity_info_for(type_alias),
            underlying_type: self.convert_type_ref(&type_alias.underlying),
        }
    }

    fn convert_sequence(&mut self, sequence: &CompilerSequence) -> SequenceType {
        SequenceType {
            element_type: self.convert_type_ref(&sequence.element_type),
        }
    }

    fn convert_dictionary(&mut self, dictionary: &CompilerDictionary) -> DictionaryType {
        DictionaryType {
            key_type: self.convert_type_ref(&dictionary.key_type),
            value_type: self.convert_type_ref(&dictionary.value_type),
        }
    }

    fn convert_result_type(&mut self, result_type: &CompilerResultType) -> ResultType {
        ResultType {
            success_type: self.convert_type_ref(&result_type.success_type),
            failure_type: self.convert_type_ref(&result_type.failure_type),
        }
    }

    fn get_type_id_for(&mut self, type_ref: &CompilerTypeRef) -> String {
        match type_ref.concrete_type() {
            CompilerTypes::Struct(v) => v.module_scoped_identifier(),
            CompilerTypes::Enum(v) => v.module_scoped_identifier(),
            CompilerTypes::CustomType(v) => v.module_scoped_identifier(),
            CompilerTypes::Primitive(v) => v.type_string(),
            CompilerTypes::ResultType(v) => {
                let converted_symbol = Symbol::ResultType(self.convert_result_type(v));
                self.converted_contents.push(converted_symbol);
                (self.converted_contents.len() - 1).to_string()
            },
            CompilerTypes::Sequence(v) => {
                let converted_symbol = Symbol::SequenceType(self.convert_sequence(v));
                self.converted_contents.push(converted_symbol);
                (self.converted_contents.len() - 1).to_string()
            },
            CompilerTypes::Dictionary(v) => {
                let converted_symbol = Symbol::DictionaryType(self.convert_dictionary(v));
                self.converted_contents.push(converted_symbol);
                (self.converted_contents.len() - 1).to_string()
            },

            CompilerTypes::Class(_) => panic!("TODO: remove classes!"),
        }
    }
}
