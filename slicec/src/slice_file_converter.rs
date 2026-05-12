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

use slicec::diagnostics::Diagnostic as SlicecDiagnostic;
use slicec::diagnostics::DiagnosticKind as SlicecDiagnosticKind;
use slicec::diagnostics::Error as SlicecError;
use slicec::diagnostics::Lint as SlicecLint;

use slicec::ast::Ast;
use slicec::diagnostics::Diagnostics;
use slicec::slice_file::Span;

// Pull in all the mapped Slice-compiler definition types.
use crate::definition_types::*;

use std::fmt::Write;

// =============================== //
// Diagnostic conversion functions //
// =============================== //

const CRITICAL_FLAW_STRING: &str = "this indicates a critical flaw in the plugin that generated this diagnostic";

/// Converts a `Diagnostic` emitted from a plugin, to a `Diagnostic` that can be used by the Slice compiler.
/// 
/// Instead of directly returning the converted `Diagnostic`, this function pushes it into a provided `Diagnostics`
/// container. This is because a single plugin-diagnostic may actually produce multiple compiler-diagnostics, for
/// example, if a plugin emits a malformed or unknown diagnostic.
pub fn convert_diagnostic(diagnostic: Diagnostic, ast: &Ast, files: &[GrammarSliceFile], output: &mut Diagnostics) {
    // Perform the actual conversion between `DiagnosticKind` types.
    let kind = match diagnostic.kind {
        DiagnosticKind::Info { message } => SlicecDiagnosticKind::Info(message),

        DiagnosticKind::Warning { message } => SlicecDiagnosticKind::Lint(SlicecLint::Other { message }),

        DiagnosticKind::Error { message } => SlicecDiagnosticKind::Error(SlicecError::Other { message }),

        DiagnosticKind::InvalidAttribute { directive }
            => SlicecDiagnosticKind::Error(SlicecError::InvalidAttribute { directive }),

        DiagnosticKind::UnknownAttribute { directive }
            => SlicecDiagnosticKind::Error(SlicecError::UnknownAttribute { directive }),

        DiagnosticKind::MissingRequiredAttribute { expected_attribute }
            => SlicecDiagnosticKind::Error(SlicecError::MissingRequiredAttribute { expected_attribute }),

        DiagnosticKind::AttributeIsNotRepeatable { directive }
            => SlicecDiagnosticKind::Error(SlicecError::AttributeIsNotRepeatable { directive }),

        DiagnosticKind::InvalidAttributeArgument { directive, argument }
            => SlicecDiagnosticKind::Error(SlicecError::InvalidAttributeArgument { directive, argument }),

        DiagnosticKind::IncorrectAttributeArgumentCount { directive, min_expected, max_expected, actual_count } => {
            let min_expected = if min_expected == u8::MAX { usize::MAX } else { min_expected as usize };
            let max_expected = if max_expected == u8::MAX { usize::MAX } else { max_expected as usize };
            SlicecDiagnosticKind::Error(SlicecError::IncorrectAttributeArgumentCount {
                directive,
                expected_count: min_expected..(max_expected + 1),
                actual_count: actual_count as usize,
            })
        }

        DiagnosticKind::Unknown { discriminant, fields_payload } => {
            let mut message = format!("received an unknown diagnostic with a code of '{discriminant}'");
            if !fields_payload.is_empty() {
                write!(message, " and field-payload of:\n{fields_payload:?}").unwrap();
            }
            SlicecDiagnosticKind::Error(SlicecError::Other { message})
        }
    };

    // Convert any provided notes.
    let notes = diagnostic.notes.into_iter().map(|note| {
        let DiagnosticNote {message, source} = note;
        let (span, _) = convert_source(source.as_deref(), ast, files, output);
        slicec::diagnostics::Note { message, span }
    }).collect();

    // If a 'source' was provided, convert it to a corresponding 'span' and 'scope'.
    let (span, scope) = convert_source(diagnostic.source.as_deref(), ast, files, output);

    // Construct and push the converted 'slicec' `Diagnostic`.
    let converted_diagnostic = slicec::diagnostics::Diagnostic { kind, span, scope, notes };
    output.push(converted_diagnostic);
}

/// Parses a diagnostic 'source' string, and returns the corresponding 'span' and 'scope' for the referenced element.
fn convert_source(source: Option<&str>, ast: &Ast, files: &[GrammarSliceFile], output: &mut Diagnostics) -> (Option<Span>, Option<String>) {
    // If no source was provided, we can immediately return `(None, None)` since there's nothing to convert.
    let Some(source) = source else {
        return (None, None);
    };

    // Otherwise, split the provided source into 'the scoped id of a symbol' and an 'optional extension'.
    // This optional extension always begins with a '$' and can be used to refer to meta-elements attached to the symbol
    // like attributes or doc-comments. For example: `"MyModule::MyClass::$attributes::1"` for attribute 1 on "MyClass".
    let (symbol_id, extension) = if let Some((symbol_id, extension)) = source.split_once("::$") {
        (symbol_id, Some(extension))
    } else {
        (source, None)
    };

    // If the source starts with a '#', then it is referencing a file. Otherwise it is referencing a named symbol.
    if let Some(diagnostic_file_path) = symbol_id.strip_prefix('#') {
        // Lookup the file in the list of slice files and if it doesn't exist, emit a diagnostic and return immediately.
        let slice_file = match files.iter().find(|file| file.relative_path == diagnostic_file_path) {
            Some(file) => file,
            None => {
                let message = format!("no file with the relative path '{diagnostic_file_path}' was parsed by 'slicec'");
                SlicecDiagnostic::from_error(SlicecError::Other { message })
                    .add_note(CRITICAL_FLAW_STRING, None)
                    .push_into(output);
                return (None, None);
            }
        };

        // Determine which 'span' to return based off the optional extension.
        // The 'scope' is always `None`, since files do not logically have a Slice scope like symbols do.
        let scope = None;
        let span = match extension {
            // If there was no extension, this diagnostic is referencing the file itself.
            // We return an empty span that just points to the start of the file in this case.
            None => Some(Span::new((1, 1).into(), (1, 1).into(), &slice_file.relative_path)),

            // If the extension starts with 'attributes::', then this diagnostic is referencing the file's attributes.
            Some(ext) if ext.starts_with("attributes::") => get_attribute_span(slice_file, ext, output),

            Some(unknown) => {
                let message = format!("the diagnostic source '{source}' has an unrecognized extension '{unknown}'");
                let error = SlicecDiagnostic::from_error(SlicecError::Other { message });
                output.push(error);
                None // There is no meaningful span to return.
            }
        };
        (span, scope)
    } else {
        // Lookup the named symbol in the AST and if it doesn't exist, emit a diagnostic and return immediately.
        let named_symbol = match ast.find_element::<dyn NamedSymbol>(symbol_id) {
            Ok(named_symbol) => named_symbol,
            Err(err) => {
                SlicecDiagnostic::from_error(err.into())
                    .add_note(CRITICAL_FLAW_STRING, None)
                    .push_into(output);
                return (None, None);
            }
        };

        // Determine which 'span' to return based off the optional extension.
        // The 'scope' is always the fully-scoped identifier of the symbol, since meta-elements don't have Slice scopes.
        let scope = Some(named_symbol.parser_scoped_identifier());
        let span = match extension {
            // If there was no extension, this diagnostic is referencing the named symbol itself.
            None => Some(named_symbol.span().clone()),

            // If the extension starts with 'attributes::', then this diagnostic is referencing the symbol's attributes.
            Some(ext) if ext.starts_with("attributes::") => get_attribute_span(named_symbol, ext, output),

            Some(unknown) => {
                let message = format!("the diagnostic source '{source}' has an unrecognized extension '{unknown}'");
                let error = SlicecDiagnostic::from_error(SlicecError::Other { message });
                output.push(error);
                Some(named_symbol.span().clone()) // Fallback to using the symbol's span.
            }
        };
        (span, scope)
    }
}

fn get_attribute_span<T: Attributable + ?Sized>(symbol: &T, extension: &str, output: &mut Diagnostics) -> Option<Span> {
    // Split the extension string by '::' and ensure it starts with 'attributes' and has at least 2 parts.
    let indices = extension.split("::").collect::<Vec<_>>();
    assert!(indices.len() > 1 && indices[0] == "attributes");

    // Make sure we either 1 or 2 indices after 'attributes'.
    let (attribute_index_str, argument_index_str) = match &indices[1..] {
        [i] => (i.parse::<usize>(), None),
        [i, j] => (i.parse::<usize>(), Some(j.parse::<usize>())),

        [] => unreachable!("'get_attribute_span' had 0 indices despite asserting that there was at least 1!"),
        _ => {
            let message = format!("{} indices were supplied to the '$attributes' diagnostic source", indices.len() - 1);
            let error = SlicecDiagnostic::from_error(SlicecError::Other { message })
                .add_note(CRITICAL_FLAW_STRING, None);
            output.push(error);
            return None;
        }
    };

    // Retrieve the attribute being referenced based on the first index after "attributes::".
    let Ok(attribute_index) = attribute_index_str else {
        let message = format!("{} is not a valid integer", indices[1]);
        let error = SlicecDiagnostic::from_error(SlicecError::Other { message })
            .add_note(CRITICAL_FLAW_STRING, None);
        output.push(error);
        return None;
    };
    let Some(&attribute) = symbol.attributes().get(attribute_index) else {
        let message = format!("attribute index '{attribute_index}' is out of bounds");
        let error = SlicecDiagnostic::from_error(SlicecError::Other { message })
            .add_note(CRITICAL_FLAW_STRING, None);
        output.push(error);
        return None;
    };
    // If there was no 2nd index, then we want to return the span of the entire attribute itself.
    if argument_index_str.is_none() {
        return Some(attribute.span.clone());
    }

    //// Otherwise, we know there was a 2nd index; retrieve the argument being referenced based on this 2nd index.
    //let Ok(argument_index) = argument_index_str.unwrap() else {
    //    let message = format!("{} is not a valid integer", indices[2]);
    //    let error = SlicecDiagnostic::from_error(SlicecError::Other { message })
    //        .add_note(CRITICAL_FLAW_STRING, None);
    //    output.push(error);
    //    return None;
    //};
    //let Some(argument) = attribute.arguments().get(argument_index) else {
    //    let message = format!("argument index '{argument_index}' is out of bounds");
    //    let error = SlicecDiagnostic::from_error(SlicecError::Other { message })
    //        .add_note(CRITICAL_FLAW_STRING, None);
    //    output.push(error);
    //    return None;
    //};
    //// Return the span of the argument.
    //Some(argument.span().clone())

    // TODO: we need to refactor the 'Attributes' API to expose arguments and their spans before we can implement this.
    Some(attribute.span.clone())
}

// =================================== //
// Grammar conversion helper functions //
// =================================== //

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
struct SliceFileContentsConverter {
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
    fn convert(contents: &[GrammarDefinition]) -> Vec<Symbol> {
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
            has_streamed_parameter: operation
                .parameters
                .last()
                .is_some_and(|parameter| parameter.borrow().is_streamed),
            return_type: operation.return_members().into_iter().map(|e| self.convert_parameter(e)).collect(),
            has_streamed_return: operation
                .return_type
                .last()
                .is_some_and(|parameter| parameter.borrow().is_streamed),
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
