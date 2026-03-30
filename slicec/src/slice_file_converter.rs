// Copyright (c) ZeroC, Inc.

// Pull in the core 'slicec' types using aliases to disambiguate them from the Slice-compiler definitions.
// Any type that starts with 'Grammar' is a slicec type, not a Slice-compiler definition type.
#![cfg_attr(rustfmt, rustfmt_skip)] // Keep the `use ... as ...` one-per-line.
use slicec::grammar::Attribute as GrammarAttribute;
use slicec::grammar::CustomType as GrammarCustomType;
use slicec::grammar::Definition as GrammarDefinition;
use slicec::grammar::Dictionary as GrammarDictionary;
use slicec::grammar::DocComment as GrammarDocComment;
use slicec::grammar::Enum as GrammarEnum;
use slicec::grammar::Enumerator as GrammarEnumerator;
use slicec::grammar::Field as GrammarField;
use slicec::grammar::Identifier as GrammarIdentifier;
use slicec::grammar::Interface as GrammarInterface;
use slicec::grammar::MessageComponent as GrammarMessageComponent;
use slicec::grammar::Operation as GrammarOperation;
use slicec::grammar::Parameter as GrammarParameter;
use slicec::grammar::ResultType as GrammarResultType;
use slicec::grammar::Sequence as GrammarSequence;
use slicec::grammar::Struct as GrammarStruct;
use slicec::grammar::Types as GrammarTypes;
use slicec::grammar::TypeAlias as GrammarTypeAlias;
use slicec::grammar::TypeRef as GrammarTypeRef;
use slicec::slice_file::SliceFile as GrammarSliceFile;

// Pull in traits from 'slicec' so we can call their functions.
use slicec::grammar::{Attributable, Commentable, Contained, Entity, Member, NamedSymbol, Type};
// Pull in the attribute types without aliases, since they're not ambiguous.
use slicec::grammar::attributes::{Allow, Compress, Deprecated, Oneway, SlicedFormat, Unparsed};

// Pull in all the mapped Slice-compiler definition types.
use crate::definition_types::*;

/// Returns an [EntityInfo] describing the provided element.
fn get_entity_info_for(element: &impl Commentable) -> EntityInfo {
    EntityInfo {
        identifier: element.identifier().to_owned(),
        attributes: get_attributes_from(element.attributes()),
        comment: element.comment().map(Into::into),
    }
}

/// Returns a [`DocComment`] describing the provided parameter if one is present.
///
/// In Slice, doc-comments are not allowed on parameters. Instead, you would use a '@param' tag applied to an enclosing
/// operation. But this is an implementation detail of the language, not something code-generators should deal with.
fn get_doc_comment_for_parameter(parameter: &GrammarParameter) -> Option<DocComment> {
    let operation_comment = parameter.parent().comment()?;

    // We get the parameter's doc-comment in 3 steps:
    // 1) Try to find a matching '@param' tag on the operation's doc-comment.
    // 2) If one was present, extract just its `Message` field, and convert it to the mapped type.
    // 3) Construct a mapped `DocComment` which contains the mapped message.
    operation_comment.params.iter()
        .find(|param_tag| param_tag.identifier.value == parameter.identifier())
        .map(|param_tag| param_tag.message.value.iter().map(Into::into).collect())
        .map(|message| DocComment {
            overview: message,
            see_tags: Vec::new(),
        })
}

/// Helper function to convert the result of `tag.linked_entity()` into an [`EntityId`].
fn convert_doc_comment_link(link_result: Result<&dyn Entity, &GrammarIdentifier>) -> EntityId {
    match link_result {
        Ok(entity) => entity.parser_scoped_identifier(),
        Err(identifier) => identifier.value.clone(),
    }
}

/// Helper function to convert a [`Vec`] of compiler-attributes to mapped-attributes.
fn get_attributes_from(attributes: Vec<&GrammarAttribute>) -> Vec<Attribute> {
    attributes.into_iter().map(|attribute| Attribute {
        directive: attribute.kind.directive().to_owned(),
        args: get_attribute_args(attribute),
    })
    .collect()
}

// TODO this is a temporary hack because we know all the possible attributes.
// The `Attribute` API doesn't offer a way to convert parsed-arguments back to a string.
// And this entire API will be rewritten after porting slicec-cs, so no point changing it now.
fn get_attribute_args(attribute: &GrammarAttribute) -> Vec<String> {
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

    if attribute.downcast::<Oneway>().is_some() {
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

// =========================== //
// Direct conversion functions //
// =========================== //

impl From<&GrammarSliceFile> for SliceFile {
    fn from(slice_file: &GrammarSliceFile) -> Self {
        // Convert the slice_file's module declaration.
        // TODO this crashes on an empty Slice file, we need to filter out empty files at an earlier stage.
        let module = slice_file.module.as_ref().unwrap().borrow();
        let converted_module = Module {
            identifier: module.nested_module_identifier().to_owned(),
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

impl From<&GrammarDocComment> for DocComment {
    fn from(doc_comment: &GrammarDocComment) -> Self {
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

impl From<&GrammarMessageComponent> for MessageComponent {
    fn from(component: &GrammarMessageComponent) -> Self {
        match component {
            GrammarMessageComponent::Text(text) => MessageComponent::Text(text.clone()),
            GrammarMessageComponent::Link(tag) => {
                MessageComponent::Link(convert_doc_comment_link(tag.linked_entity()))
            }
        }
    }
}

/// This struct exposes a function ([`SliceFileContentsConverter::convert`]) that converts the contents of a Slice file
/// from their AST representation, to a representation that can be encoded with the Slice encoding.
//
// This struct is necessary due to anonymous types, which need their own symbols. So, when you convert a `Field`, that
// may need just a `Field` symbol, but it might also need a `Field`, `Sequence`, and `Dictionary` symbol if the field's
// type uses a sequence of dictionaries. To handle this, we need to keep some state (`converted_contents`), which
// symbols can be pushed into at any time. Since there's no way to know how many symbols a definition will need upfront.
#[derive(Debug)]
pub struct SliceFileContentsConverter {
    converted_contents: Vec<Symbol>,
}

impl SliceFileContentsConverter {
    /// Converts the contents of SliceFile from their representation in the AST (as [`GrammarDefinition`]s), to their
    /// representation in the `Compiler` Slice module (as [`Symbol`]s).
    ///
    /// Specifically, this iterates through the top-level definitions of a Slice-file (in definition order) converting
    /// and storing them. In addition to top-level definitions, the returned [`Vec`] also contains [`Symbol`]s for each
    /// anonymous type encountered while iterating. Anonymous types always appear in the returned contents _before_
    /// the [`Symbol`]s that referenced them.
    pub fn convert(contents: &[GrammarDefinition]) -> Vec<Symbol> {
        // Create a new converter.
        let mut converter = SliceFileContentsConverter {
            converted_contents: Vec::new()
        };

        // Iterate through the provided file's contents, and convert each of it's top-level definitions.
        for definition in contents {
            let converted = match definition {
                GrammarDefinition::Struct(v) => Symbol::Struct(converter.convert_struct(v.borrow())),
                GrammarDefinition::Interface(v) => Symbol::Interface(converter.convert_interface(v.borrow())),
                GrammarDefinition::Enum(v) => converter.convert_enum(v.borrow()),
                GrammarDefinition::CustomType(v) => Symbol::CustomType(converter.convert_custom_type(v.borrow())),
                GrammarDefinition::TypeAlias(v) => Symbol::TypeAlias(converter.convert_type_alias(v.borrow())),
                _ => panic!("TODO: remove exceptions"),
            };
            converter.converted_contents.push(converted);
        }

        // Return all the converted elements, consuming the converter.
        converter.converted_contents
    }

    fn convert_type_ref(&mut self, type_ref: &GrammarTypeRef) -> TypeRef {
        TypeRef {
            type_id: self.get_type_id_for(type_ref),
            is_optional: type_ref.is_optional,
            type_attributes: get_attributes_from(type_ref.attributes()),
        }
    }

    fn convert_struct(&mut self, struct_def: &GrammarStruct) -> Struct {
        Struct {
            entity_info: get_entity_info_for(struct_def),
            is_compact: struct_def.is_compact,
            fields: struct_def.fields().into_iter().map(|e| self.convert_field(e)).collect(),
        }
    }

    fn convert_field(&mut self, field: &GrammarField) -> Field {
        Field {
            entity_info: get_entity_info_for(field),
            tag: field.tag.as_ref().map(|integer| integer.value as i32),
            data_type: self.convert_type_ref(field.data_type()),
        }
    }

    fn convert_interface(&mut self, interface_def: &GrammarInterface) -> Interface {
        let bases = interface_def.base_interfaces();

        Interface {
            entity_info: get_entity_info_for(interface_def),
            bases: bases.into_iter().map(|i| i.module_scoped_identifier()).collect(),
            operations: interface_def.operations().into_iter().map(|e| self.convert_operation(e)).collect(),
        }
    }

    fn convert_operation(&mut self, operation: &GrammarOperation) -> Operation {
        Operation {
            entity_info: get_entity_info_for(operation),
            is_idempotent: operation.is_idempotent,
            parameters: operation.parameters().into_iter().map(|e| self.convert_parameter(e)).collect(),
            has_streamed_parameter: operation.streamed_parameter().is_some(),
            return_type: operation.return_members().into_iter().map(|e| self.convert_parameter(e)).collect(),
            has_streamed_return: operation.streamed_return_member().is_some(),
        }
    }

    fn convert_parameter(&mut self, parameter: &GrammarParameter) -> Field {
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

    // This returns a `Symbol` because the `enum` grammar construct can map to either a `BasicEnum` or a `VariantEnum`.
    fn convert_enum(&mut self, enum_def: &GrammarEnum) -> Symbol {
        if let Some(underlying_type) = enum_def.underlying.as_ref() {
            Symbol::BasicEnum(BasicEnum {
                entity_info: get_entity_info_for(enum_def),
                is_unchecked: enum_def.is_unchecked,
                underlying: underlying_type.type_string(),
                enumerators: enum_def.enumerators().into_iter().map(|e| self.convert_enumerator(e)).collect(),
            })
        } else {
            Symbol::VariantEnum(VariantEnum {
                entity_info: get_entity_info_for(enum_def),
                is_compact: enum_def.is_compact,
                is_unchecked: enum_def.is_unchecked,
                variants: enum_def.enumerators().into_iter().map(|e| self.convert_variant(e)).collect(),
            })
        }
    }

    fn convert_enumerator(&mut self, enumerator: &GrammarEnumerator) -> Enumerator {
        let entity_info = get_entity_info_for(enumerator);
        let absolute_value = enumerator.value().unsigned_abs() as u64;
        let has_negative_value = enumerator.value().is_negative();

        Enumerator { entity_info, absolute_value, has_negative_value }
    }

    fn convert_variant(&mut self, enumerator: &GrammarEnumerator) -> Variant {
        let entity_info = get_entity_info_for(enumerator);
        let discriminant = enumerator.value().try_into().unwrap();
        let fields = enumerator.fields().into_iter().map(|e| self.convert_field(e)).collect();

        Variant { entity_info, discriminant, fields }
    }

    fn convert_custom_type(&mut self, custom_type: &GrammarCustomType) -> CustomType {
        CustomType {
            entity_info: get_entity_info_for(custom_type)
        }
    }

    fn convert_type_alias(&mut self, type_alias: &GrammarTypeAlias) -> TypeAlias {
        TypeAlias {
            entity_info: get_entity_info_for(type_alias),
            underlying_type: self.convert_type_ref(&type_alias.underlying),
        }
    }

    fn convert_sequence(&mut self, sequence: &GrammarSequence) -> SequenceType {
        SequenceType {
            element_type: self.convert_type_ref(&sequence.element_type),
        }
    }

    fn convert_dictionary(&mut self, dictionary: &GrammarDictionary) -> DictionaryType {
        DictionaryType {
            key_type: self.convert_type_ref(&dictionary.key_type),
            value_type: self.convert_type_ref(&dictionary.value_type),
        }
    }

    fn convert_result_type(&mut self, result_type: &GrammarResultType) -> ResultType {
        ResultType {
            success_type: self.convert_type_ref(&result_type.success_type),
            failure_type: self.convert_type_ref(&result_type.failure_type),
        }
    }

    /// Returns a [TypeId] for the provided `type_ref`. This is a fully-scoped identifier for user-defined types,
    /// the corresponding keyword for primitive types, and for anonymous types, we do the following:
    /// 1) Recursively convert the anonymous type (and any nested types) to the mapped definition types.
    /// 2) Add these directly to [Self::converted_contents] (so these types appear in the contents before their users)
    /// 3) Return its index in [Self::converted_contents] as a numeric TypeId.
    fn get_type_id_for(&mut self, type_ref: &GrammarTypeRef) -> TypeId {
        match type_ref.concrete_type() {
            GrammarTypes::Struct(v) => v.module_scoped_identifier(),
            GrammarTypes::Enum(v) => v.module_scoped_identifier(),
            GrammarTypes::CustomType(v) => v.module_scoped_identifier(),
            GrammarTypes::Primitive(v) => v.type_string(),
            GrammarTypes::ResultType(v) => {
                let converted_symbol = Symbol::ResultType(self.convert_result_type(v));
                self.converted_contents.push(converted_symbol);
                (self.converted_contents.len() - 1).to_string()
            }
            GrammarTypes::Sequence(v) => {
                let converted_symbol = Symbol::SequenceType(self.convert_sequence(v));
                self.converted_contents.push(converted_symbol);
                (self.converted_contents.len() - 1).to_string()
            }
            GrammarTypes::Dictionary(v) => {
                let converted_symbol = Symbol::DictionaryType(self.convert_dictionary(v));
                self.converted_contents.push(converted_symbol);
                (self.converted_contents.len() - 1).to_string()
            }
        }
    }
}
