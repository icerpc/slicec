// Copyright (c) ZeroC, Inc.

// Pull in all the mapped compiler-definition types.
use crate::definition_types::*;

// Pull in traits from 'slicec' so we can call their functions.
use slicec::grammar::Attributable;
use slicec::grammar::Commentable;
use slicec::grammar::Contained;
use slicec::grammar::Member;
use slicec::grammar::NamedSymbol;
use slicec::grammar::Type;
use slicec::visitor::Visitor;

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
use slicec::grammar::Interface as CompilerInterface;
use slicec::grammar::Module as CompilerModule;
use slicec::grammar::Operation as CompilerOperation;
use slicec::grammar::Parameter as CompilerParameter;
use slicec::grammar::ResultType as CompilerResultType;
use slicec::grammar::Sequence as CompilerSequence;
use slicec::grammar::Struct as CompilerStruct;
use slicec::grammar::Types as CompilerTypes;
use slicec::grammar::TypeAlias as CompilerTypeAlias;
use slicec::grammar::TypeRef as CompilerTypeRef;
use slicec::slice_file::SliceFile as CompilerSliceFile;

pub struct AnonymousTypeEncoder {
    anonymous_types: Vec<Symbol>,
}

impl Visitor for AnonymousTypeEncoder {
    // TODO check that this visits all the places that we care about correctly!
    fn visit_type_ref(&mut self, type_ref: &CompilerTypeRef) {
        match type_ref.concrete_type() {
            CompilerTypes::Sequence(v) => self.anonymous_types.push(Symbol::SequenceType(v.into())),
            CompilerTypes::Dictionary(v) => self.anonymous_types.push(Symbol::DictionaryType(v.into())),
            CompilerTypes::ResultType(v) => self.anonymous_types.push(Symbol::ResultType(v.into())),
            _ => {}
        }
    }
}

fn get_entity_info_for(element: &impl Commentable) -> EntityInfo {
    EntityInfo {
        identifier: element.identifier().to_owned(),
        attributes: get_attributes_from(element.attributes()),
        comment: element.comment().map(Into::into),
    }
}

fn get_attributes_from(attributes: Vec<&CompilerAttribute>) -> Vec<Attribute> {

}

impl From<&CompilerSliceFile> for SliceFile {
    fn from(slice_file: &CompilerSliceFile) -> Self {
        // TODO run the symbol visitor from this spot!

        SliceFile {
            path: slice_file.relative_path.clone(),
            module_declaration: slice_file.module.as_ref().unwrap().borrow().into(),
            attributes: get_attributes_from(slice_file.attributes()),
            contents: slice_file.contents.iter().map(Into::into).collect(),
        }
    }
}

impl From<&CompilerModule> for Module {
    fn from(module: &CompilerModule) -> Self {
        Module {
            identifier: module.identifier().to_owned(),
            attributes: get_attributes_from(module.attributes()),
        }
    }
}

impl From<&CompilerDefinition> for Symbol {
    fn from(definition: &CompilerDefinition) -> Self {
        match definition {
            CompilerDefinition::Struct(v) => Symbol::Struct(v.borrow().into()),
            CompilerDefinition::Interface(v) => Symbol::Interface(v.borrow().into()),
            CompilerDefinition::Enum(v) => Symbol::Enum(v.borrow().into()),
            CompilerDefinition::CustomType(v) => Symbol::CustomType(v.borrow().into()),
            CompilerDefinition::TypeAlias(v) => Symbol::TypeAlias(v.borrow().into()),
            _ => panic!("TODO: remove classes and exceptions"),
        }
    }
}

impl From<&CompilerStruct> for Struct {
    fn from(struct_def: &CompilerStruct) -> Self {
        Struct {
            entity_info: get_entity_info_for(struct_def),
            is_compact: struct_def.is_compact,
            fields: struct_def.fields().into_iter().map(Into::into).collect(),
        }
    }
}

impl From<&CompilerField> for Field {
    fn from(field: &CompilerField) -> Self {
        Field {
            entity_info: get_entity_info_for(field),
            tag: field.tag.as_ref().map(|integer| integer.value as i32),
            data_type: field.data_type().into(),
        }
    }
}

impl From<&CompilerInterface> for Interface {
    fn from(interface: &CompilerInterface) -> Self {
        let bases = interface.base_interfaces();

        Interface {
            entity_info: get_entity_info_for(interface),
            bases: bases.into_iter().map(|i| i.parser_scoped_identifier()).collect(),
            operations: interface.operations().into_iter().map(Into::into).collect(),
        }
    }
}

impl From<&CompilerOperation> for Operation {
    fn from(operation: &CompilerOperation) -> Self {
        Operation {
            entity_info: get_entity_info_for(operation),
            is_idempotent: operation.is_idempotent,
            parameters: operation.parameters().into_iter().map(Into::into).collect(),
            has_streamed_parameter: operation.streamed_parameter().is_some(),
            return_type: operation.return_members().into_iter().map(Into::into).collect(),
            has_streamed_return: operation.streamed_return_member().is_some(), 
        }
    }
}

impl From<&CompilerParameter> for Field {
    fn from(parameter: &CompilerParameter) -> Self {
        // Check if this parameter's parent operation has a corresponding doc-comment on it.
        let parameter_comment = parameter.parent().comment().and_then(|comment| {
            comment.params.iter()
                .find(|param_tag| param_tag.identifier.value == parameter.identifier())
                .map(|param_tag| param_tag.message)
        });

        let parameter_info = EntityInfo {
            identifier: parameter.identifier().to_owned(),
            attributes: get_attributes_from(parameter.attributes()),
            comment: parameter_comment,
        };

        Field {
            entity_info: parameter_info,
            tag: parameter.tag.as_ref().map(|integer| integer.value as i32),
            data_type: parameter.data_type().into(),
        }
    }
}

impl From<&CompilerEnum> for Enum {
    fn from(enum_def: &CompilerEnum) -> Self {
        Enum {
            entity_info: get_entity_info_for(enum_def),
            is_compact: enum_def.is_compact,
            is_unchecked: enum_def.is_unchecked,
            underlying: enum_def.underlying.as_ref().map(|type_ref| type_ref.type_string()),
            enumerators: enum_def.enumerators().into_iter().map(Into::into).collect(), 
        }
    }
}

impl From<&CompilerEnumerator> for Enumerator {
    fn from(enumerator: &CompilerEnumerator) -> Self {
        let entity_info = get_entity_info_for(enumerator);
        let raw_value = enumerator.value();
        let value = Discriminant {
            absolute_value: raw_value.abs() as u64,
            is_positive: raw_value.is_positive(),
        };
        let fields = enumerator.fields().into_iter().map(Into::into).collect();

        Enumerator { entity_info, value, fields }
    }
}

impl From<&CompilerCustomType> for CustomType {
    fn from(custom_type: &CompilerCustomType) -> Self {
        CustomType { entity_info: get_entity_info_for(custom_type) }
    }
}

impl From<&CompilerTypeAlias> for TypeAlias {
    fn from(type_alias: &CompilerTypeAlias) -> Self {
        TypeAlias {
            entity_info: get_entity_info_for(type_alias),
            underlying_type: (&type_alias.underlying).into()
        }
    }
}

impl From<&CompilerSequence> for SequenceType {
    fn from(sequence: &CompilerSequence) -> Self {
        SequenceType {
            element_type: (&sequence.element_type).into(),
        }
    }
}

impl From<&CompilerDictionary> for DictionaryType {
    fn from(dictionary: &CompilerDictionary) -> Self {
        DictionaryType {
            key_type: (&dictionary.key_type).into(),
            value_type: (&dictionary.value_type).into(),
        }
    }
}

impl From<&CompilerResultType> for ResultType {
    fn from(result_type: &CompilerResultType) -> Self {
        ResultType {
            success_type: (&result_type.success_type).into(),
            failure_type: (&result_type.failure_type).into(),
        }
    }
}

impl From<&CompilerTypeRef> for TypeRef {
    fn from(type_ref: &CompilerTypeRef) -> Self {
        let type_string = match type_ref.concrete_type() {
            CompilerTypes::Struct(v) => v.module_scoped_identifier(),
            CompilerTypes::Enum(v) => v.module_scoped_identifier(),
            CompilerTypes::CustomType(v) => v.module_scoped_identifier(),
            CompilerTypes::Primitive(v) => v.type_string(),
            CompilerTypes::ResultType(v) => todo!(),
            CompilerTypes::Sequence(v) => todo!(),
            CompilerTypes::Dictionary(v) => todo!(),

            CompilerTypes::Class(_) => panic!("remove classes!"),
        };

        TypeRef {
            type_id: type_string,
            is_optional: type_ref.is_optional,
            type_attributes: get_attributes_from(type_ref.attributes()),
        }
    }
}






impl From<&CompilerDocComment> for DocComment {
    fn from(value: &CompilerDocComment) -> Self {
        
    }
}
