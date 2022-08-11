// Copyright (c) ZeroC, Inc. All rights reserved.

use super::comments::CommentParser;
use crate::ast::Ast;
use crate::errors::*;
use crate::grammar::*;
use crate::slice_file::{SliceFile, Span};
use crate::upcast_weak_as;
use crate::utils::ptr_util::{OwnedPtr, WeakPtr};
use std::cell::RefCell;
use std::convert::TryInto;
use std::default::Default;
use std::fs;
use std::ops::RangeInclusive;

use pest::error::ErrorVariant as PestErrorVariant;
use pest_consume::{match_nodes, Error as PestError, Parser as PestParser};

type PestResult<T> = Result<T, PestError<Rule>>;
type PestNode<'a, 'b, 'ast> = pest_consume::Node<'a, Rule, &'b RefCell<ParserData<'ast>>>;

fn get_span_for(input: &PestNode) -> Span {
    let span = input.as_span();
    Span {
        start: span.start_pos().line_col(),
        end: span.end_pos().line_col(),
        file: input.user_data().borrow().current_file.clone(),
    }
}

fn get_scope(input: &PestNode) -> Scope {
    input.user_data().borrow().current_scope.clone()
}

fn push_scope(input: &PestNode, identifier: &str, is_module: bool) {
    let scope = &mut input.user_data().borrow_mut().current_scope;
    scope.push_scope(identifier, is_module);
}

fn pop_scope(input: &PestNode) {
    let scope = &mut input.user_data().borrow_mut().current_scope;
    scope.pop_scope();
}

#[derive(Debug)]
struct ParserData<'a> {
    ast: &'a mut Ast,
    current_file: String,
    current_encoding: Encoding,
    current_enum_value: Option<i64>,
    is_in_return_tuple: bool,
    current_scope: Scope,
    error_reporter: &'a mut ErrorReporter,
}

#[derive(PestParser)]
#[grammar = "parser/slice.pest"]
pub(super) struct SliceParser<'a> {
    pub error_reporter: &'a mut ErrorReporter,
}

impl<'a> SliceParser<'a> {
    pub fn try_parse_file(&mut self, file: &str, is_source: bool, ast: &mut Ast) -> Option<SliceFile> {
        match self.parse_file(file, is_source, ast) {
            Ok(slice_file) => Some(slice_file),
            Err(message) => {
                self.error_reporter.report(ErrorKind::Syntax(message), None);
                None
            }
        }
    }

    fn parse_file(&mut self, file: &str, is_source: bool, ast: &mut Ast) -> Result<SliceFile, String> {
        let user_data = RefCell::new(ParserData {
            ast,
            current_file: file.to_owned(),
            current_encoding: Encoding::default(),
            current_enum_value: None,
            is_in_return_tuple: false,
            current_scope: Scope::default(),
            error_reporter: self.error_reporter,
        });

        // Read the raw text from the file, and parse it into a raw ast.
        let raw_text = fs::read_to_string(&file).map_err(|e| e.to_string())?;
        let node = SliceParser::parse_with_userdata(Rule::main, &raw_text, &user_data).map_err(|e| e.to_string())?; // TODO maybe make this error print prettier?
        let raw_ast = node.single().expect("Failed to unwrap raw_ast!");

        // Consume the raw ast into an unpatched ast, then store it in a `SliceFile`.
        let (file_attributes, top_level_modules, file_encoding) =
            SliceParser::main(raw_ast).map_err(|e| e.to_string())?;

        Ok(SliceFile::new(
            file.to_owned(),
            raw_text,
            top_level_modules,
            file_attributes,
            file_encoding,
            is_source,
        ))
    }

    pub fn try_parse_string(&mut self, identifier: &str, input: &str, ast: &mut Ast) -> Option<SliceFile> {
        match self.parse_string(identifier, input, ast) {
            Ok(slice_file) => Some(slice_file),
            Err(message) => {
                self.error_reporter.report(ErrorKind::Syntax(message), None);
                None
            }
        }
    }

    fn parse_string(&mut self, identifier: &str, input: &str, ast: &mut Ast) -> Result<SliceFile, String> {
        let user_data = RefCell::new(ParserData {
            ast,
            current_file: identifier.to_owned(),
            current_encoding: Encoding::default(),
            current_enum_value: None,
            is_in_return_tuple: false,
            current_scope: Scope::default(),
            error_reporter: self.error_reporter,
        });

        // Parse the file into a file-specific AST.
        let node = SliceParser::parse_with_userdata(Rule::main, input, &user_data);

        let unwrapped_node = node.map_err(|e| e.to_string())?;

        let raw_ast = unwrapped_node.single().expect("Failed to unwrap AST");

        // Consume the contents of the file and add them into the AST.
        let (file_attributes, top_level_modules, file_encoding) =
            SliceParser::main(raw_ast).map_err(|e| e.to_string())?;

        let slice_file = SliceFile::new(
            identifier.to_owned(),
            input.to_owned(),
            top_level_modules,
            file_attributes,
            file_encoding,
            false, // skip code generation
        );

        Ok(slice_file)
    }
}

// Make Clippy happy until Pest goes away.
type MainReturnType = PestResult<(Vec<Attribute>, Vec<WeakPtr<Module>>, Option<FileEncoding>)>;

#[pest_consume::parser]
impl<'a> SliceParser<'a> {
    fn main(input: PestNode) -> MainReturnType {
        let module_ids = match_nodes!(input.into_children();
            [file_attributes(attributes), module_def(modules).., EOI(_)] => {
                (attributes, modules.collect(), None)
            },
            [file_attributes(attributes), file_level_module(module), EOI(_)] => {
                (attributes, vec![module], None)
            },
            [file_encoding(encoding), file_attributes(attributes), module_def(modules).., EOI(_)] => {
                (attributes, modules.collect(), Some(encoding))
            },
            [file_encoding(encoding), file_attributes(attributes), file_level_module(module), EOI(_)] => {
                (attributes, vec![module], Some(encoding))
            }
        );
        Ok(module_ids)
    }

    fn file_encoding(input: PestNode) -> PestResult<FileEncoding> {
        Ok(match_nodes!(input.children();
            [_, encoding_version(encoding)] => {
                input.user_data().borrow_mut().current_encoding = encoding;
                FileEncoding { version: encoding, span: get_span_for(&input) }
            }
        ))
    }

    fn encoding_version(input: PestNode) -> PestResult<Encoding> {
        match input.as_str() {
            "1" => Ok(Encoding::Slice1),
            "2" => Ok(Encoding::Slice2),
            _ => Err(PestError::new_from_span(
                PestErrorVariant::CustomError {
                    message: format!("Unknown slice encoding version: {input}"),
                },
                input.as_span(),
            )),
        }
    }

    fn definition(input: PestNode) -> PestResult<Definition> {
        Ok(match_nodes!(input.into_children();
            [module_def(module_ptr)]       => Definition::Module(module_ptr),
            [struct_def(struct_ptr)]       => Definition::Struct(struct_ptr),
            [class_def(class_ptr)]         => Definition::Class(class_ptr),
            [exception_def(exception_ptr)] => Definition::Exception(exception_ptr),
            [interface_def(interface_ptr)] => Definition::Interface(interface_ptr),
            [enum_def(enum_ptr)]           => Definition::Enum(enum_ptr),
            [trait_def(trait_ptr)]         => Definition::Trait(trait_ptr),
            [custom_type(custom_type_ptr)] => Definition::CustomType(custom_type_ptr),
            [type_alias(type_alias_ptr)]   => Definition::TypeAlias(type_alias_ptr),
        ))
    }

    fn module_start(input: PestNode) -> PestResult<(Identifier, Span)> {
        let span = get_span_for(&input);
        let identifier = match_nodes!(input.children();
            [_, scoped_identifier(ident)] => ident,
        );

        // Split the identifier in case it uses nested module syntax, and push a scope for each.
        for module_identifier in identifier.value.split("::") {
            push_scope(&input, module_identifier, true);
        }
        Ok((identifier, span))
    }

    fn module_def(input: PestNode) -> PestResult<WeakPtr<Module>> {
        Self::parse_module(input, true)
    }

    fn file_level_module(input: PestNode) -> PestResult<WeakPtr<Module>> {
        Self::parse_module(input, false)
    }

    fn struct_start(input: PestNode) -> PestResult<(bool, Identifier, Span)> {
        let span = get_span_for(&input);
        Ok(match_nodes!(input.children();
            [compact_modifier(is_compact), _, identifier(identifier)] => {
                push_scope(&input, &identifier.value, false);
                (is_compact, identifier, span)
            }
        ))
    }

    fn struct_def(input: PestNode) -> PestResult<WeakPtr<Struct>> {
        let scope = get_scope(&input);
        Ok(match_nodes!(input.children();
            [prelude(prelude), struct_start(struct_start), data_member_list(members)] => {
                let (is_compact, identifier, span) = struct_start;
                let (attributes, comment) = prelude;
                let mut struct_def = Struct::new(identifier, is_compact, scope, attributes, comment, span);
                for member in members {
                    struct_def.add_member(member);
                }
                pop_scope(&input);

                let ast = &mut input.user_data().borrow_mut().ast;
                ast.add_named_element(OwnedPtr::new(struct_def))
            },
        ))
    }

    #[allow(clippy::type_complexity)]
    fn class_start(input: PestNode) -> PestResult<(Identifier, Option<u32>, Span, Option<TypeRef<Class>>)> {
        let span = get_span_for(&input);
        Ok(match_nodes!(input.children();
            [_, identifier(identifier), compact_id(compact_id)] => {
                push_scope(&input, &identifier.value, false);
                (identifier, compact_id, span, None)
            },
            [_, identifier(identifier), compact_id(compact_id), _, inheritance_list(bases)] => {
                // Classes can only inherit from a single base class.
                if bases.len() > 1 {
                    input.user_data().borrow_mut().error_reporter.report(
                        LogicKind::CanOnlyInheritFromSingleBase("class".to_string()),
                        Some(&span),
                    );
                }

                push_scope(&input, &identifier.value, false);

                let base = bases.into_iter().next().unwrap().downcast::<Class>().unwrap();
                (identifier, compact_id, span, Some(base))
            }
        ))
    }

    fn class_def(input: PestNode) -> PestResult<WeakPtr<Class>> {
        let scope = get_scope(&input);
        Ok(match_nodes!(input.children();
            [prelude(prelude), class_start(class_start), data_member_list(members)] => {
                let (identifier, compact_id, span, base) = class_start;
                let (attributes, comment) = prelude;
                let mut class = Class::new(identifier, compact_id, base, scope, attributes, comment, span);
                for member in members {
                    class.add_member(member);
                }
                pop_scope(&input);

                let ast = &mut input.user_data().borrow_mut().ast;
                ast.add_named_element(OwnedPtr::new(class))
            },
        ))
    }

    fn exception_start(input: PestNode) -> PestResult<(Identifier, Span, Option<TypeRef<Exception>>)> {
        let span = get_span_for(&input);
        Ok(match_nodes!(input.children();
            [_, identifier(identifier)] => {
                push_scope(&input, &identifier.value, false);
                (identifier, span, None)
            },
            [_, identifier(identifier), _, inheritance_list(bases)] => {
                // Exceptions can only inherit from a single base exception.
                if bases.len() > 1 {
                    input.user_data().borrow_mut().error_reporter.report(
                        LogicKind::CanOnlyInheritFromSingleBase("exception".to_string()),
                        Some(&span),
                    )
                }

                push_scope(&input, &identifier.value, false);

                let base = bases.into_iter().next().unwrap().downcast::<Exception>().unwrap();
                (identifier, span, Some(base))
            }
        ))
    }

    fn exception_def(input: PestNode) -> PestResult<WeakPtr<Exception>> {
        let scope = get_scope(&input);
        Ok(match_nodes!(input.children();
            [prelude(prelude), exception_start(exception_start), data_member_list(members)] => {
                let (identifier, span, base) = exception_start;
                let (attributes, comment) = prelude;
                let mut exception = Exception::new(identifier, base, scope, attributes, comment, span);
                for member in members {
                    exception.add_member(member);
                }
                pop_scope(&input);

                let ast = &mut input.user_data().borrow_mut().ast;
                ast.add_named_element(OwnedPtr::new(exception))
            },
        ))
    }

    fn interface_start(input: PestNode) -> PestResult<(Identifier, Span, Vec<TypeRef<Interface>>)> {
        let span = get_span_for(&input);
        Ok(match_nodes!(input.children();
            [_, identifier(identifier)] => {
                push_scope(&input, &identifier.value, false);
                (identifier, span, Vec::new())
            },
            [_, identifier(identifier), _, inheritance_list(bases)] => {
                let mut bases_vector = Vec::new();
                for base in bases {
                    bases_vector.push(base.downcast::<Interface>().unwrap());
                }
                push_scope(&input, &identifier.value, false);
                (identifier, span, bases_vector)
            }
        ))
    }

    fn interface_def(input: PestNode) -> PestResult<WeakPtr<Interface>> {
        let scope = get_scope(&input);
        Ok(match_nodes!(input.children();
            [prelude(prelude), interface_start(interface_start), operation(operations)..] => {
                let (identifier, span, bases) = interface_start;
                let (attributes, comment) = prelude;
                let mut interface = Interface::new(
                    identifier,
                    bases,
                    scope,
                    attributes,
                    comment,
                    span,
                );
                for operation in operations {
                    interface.add_operation(operation);
                }
                pop_scope(&input);

                let ast = &mut input.user_data().borrow_mut().ast;
                ast.add_named_element(OwnedPtr::new(interface))
            },
        ))
    }

    fn enum_start(input: PestNode) -> PestResult<(bool, Identifier, Span, Option<TypeRef<Primitive>>)> {
        // Reset the current enumerator value back to None.
        input.user_data().borrow_mut().current_enum_value = None;

        let span = get_span_for(&input);
        Ok(match_nodes!(input.children();
            [unchecked_modifier(unchecked), _, identifier(identifier)] => {
                push_scope(&input, &identifier.value, false);
                (unchecked, identifier, span, None)
            },
            [unchecked_modifier(unchecked), _, identifier(identifier), _, typeref(type_ref)] => {
                let underlying = match type_ref.downcast::<Primitive>() {
                    Ok(primitive_def) => primitive_def,
                    _ => panic!("MUST BE A PRIMITIVE TODO"),
                };
                push_scope(&input, &identifier.value, false);
                (unchecked, identifier, span, Some(underlying))
            },
        ))
    }

    fn enum_def(input: PestNode) -> PestResult<WeakPtr<Enum>> {
        let scope = get_scope(&input);
        Ok(match_nodes!(input.children();
            [prelude(prelude), enum_start(enum_start), enumerator_list(enumerators)] => {
                let (is_unchecked, identifier, span, underlying) = enum_start;
                let (attributes, comment) = prelude;
                let mut enum_def = Enum::new(
                    identifier,
                    underlying,
                    is_unchecked,
                    scope,
                    attributes,
                    comment,
                    span,
                );
                for enumerator in enumerators {
                    enum_def.add_enumerator(enumerator);
                }
                pop_scope(&input);

                let ast = &mut input.user_data().borrow_mut().ast;
                ast.add_named_element(OwnedPtr::new(enum_def))
            },
            [prelude(prelude), enum_start(enum_start)] => {
                let (is_unchecked, identifier, span, underlying) = enum_start;
                let (attributes, comment) = prelude;
                pop_scope(&input);
                let enum_def = Enum::new(
                    identifier,
                    underlying,
                    is_unchecked,
                    scope,
                    attributes,
                    comment,
                    span,
                );

                let ast = &mut input.user_data().borrow_mut().ast;
                ast.add_named_element(OwnedPtr::new(enum_def))
            },
        ))
    }

    fn trait_def(input: PestNode) -> PestResult<WeakPtr<Trait>> {
        let span = get_span_for(&input);
        let scope = get_scope(&input);
        Ok(match_nodes!(input.children();
            [prelude(prelude), _, identifier(identifier)] => {
                let (attributes, comment) = prelude;
                let trait_def = Trait::new(identifier, scope, attributes, comment, span);

                let ast = &mut input.user_data().borrow_mut().ast;
                ast.add_named_element(OwnedPtr::new(trait_def))
            },
        ))
    }

    fn custom_type(input: PestNode) -> PestResult<WeakPtr<CustomType>> {
        let span = get_span_for(&input);
        let scope = get_scope(&input);
        Ok(match_nodes!(input.children();
            [prelude(prelude), _, identifier(identifier)] => {
                let (attributes, comment) = prelude;
                let custom_type = CustomType::new(identifier, scope, attributes, comment, span);

                let ast = &mut input.user_data().borrow_mut().ast;
                ast.add_named_element(OwnedPtr::new(custom_type))
            },
        ))
    }

    // Parses an operation's return type. There are 2 possible syntaxes for a return type:
    //   A single unnamed return type, specified by a typename.
    //   A return tuple, specified as a list of named elements enclosed in parenthesis.
    fn return_type(input: PestNode) -> PestResult<Vec<WeakPtr<Parameter>>> {
        let span = get_span_for(&input);
        let scope = get_scope(&input);
        Ok(match_nodes!(input.children();
            [return_tuple(tuple)] => tuple,
            [local_attributes(attributes), tag_modifier(tag), stream_modifier(is_streamed), typeref(data_type)] => {
                let identifier = Identifier::new("returnValue".to_owned(), span.clone());
                let parameter = Parameter::new(
                    identifier,
                    data_type,
                    tag,
                    is_streamed,
                    true,
                    scope,
                    attributes,
                    None,
                    span,
                );

                let ast = &mut input.user_data().borrow_mut().ast;
                vec![ast.add_named_element(OwnedPtr::new(parameter))]
            },
        ))
    }

    // Parses a return type that is written in return tuple syntax.
    fn return_tuple(input: PestNode) -> PestResult<Vec<WeakPtr<Parameter>>> {
        input.user_data().borrow_mut().is_in_return_tuple = true;
        let result = match_nodes!(input.children();
            // Return tuple elements and parameters have the same syntax, so we re-use the parsing
            // for parameter lists, then change their member type here, after the fact.
            [parameter_list(return_elements)] => {
                // Validate that return tuples must contain at least two elements.
                // TODO: should we move this into the validators, instead of a parse-time check?
                if return_elements.len() < 2 {
                    let span = get_span_for(&input);
                    input.user_data().borrow_mut().error_reporter.report(
                        LogicKind::ReturnTuplesMustContainAtLeastTwoElements,
                        Some(&span),
                    );
                }
                return_elements
            },
        );
        input.user_data().borrow_mut().is_in_return_tuple = false;
        Ok(result)
    }

    fn operation_start(input: PestNode) -> PestResult<(bool, Identifier)> {
        Ok(match_nodes!(input.children();
            [idempotent_modifier(is_idempotent), identifier(identifier)] => {
                push_scope(&input, &identifier.value, false);
                (is_idempotent, identifier)
            },
        ))
    }

    fn operation_return(input: PestNode) -> PestResult<Vec<WeakPtr<Parameter>>> {
        Ok(match_nodes!(input.into_children();
            [] => Vec::new(),
            [return_type(return_type)] => return_type,
        ))
    }

    fn operation(input: PestNode) -> PestResult<WeakPtr<Operation>> {
        let span = get_span_for(&input);
        let scope = get_scope(&input);
        let operation = match_nodes!(input.children();
            [prelude(prelude), operation_start(operation_start), parameter_list(parameters), operation_return(return_type)] => {
                let (attributes, comment) = prelude;
                let (is_idempotent, identifier) = operation_start;
                let encoding = input.user_data().borrow().current_encoding;

                let mut operation = Operation::new(identifier, return_type, is_idempotent, encoding, scope, attributes, comment, span);
                for parameter in parameters {
                    operation.add_parameter(parameter);
                }
                pop_scope(&input);

                let ast = &mut input.user_data().borrow_mut().ast;
                ast.add_named_element(OwnedPtr::new(operation))
            },
        );
        Ok(operation)
    }

    fn data_member_list(input: PestNode) -> PestResult<Vec<WeakPtr<DataMember>>> {
        Ok(match_nodes!(input.into_children();
            [] => Vec::new(),
            [data_member(data_member)] => vec![data_member],
            [data_member(data_member), data_member_list(mut list)] => {
                // The data_member comes before the data_member_list when parsing, so we have to
                // insert the new data member at the front of the list.
                list.insert(0, data_member);
                list
            },
        ))
    }

    fn data_member(input: PestNode) -> PestResult<WeakPtr<DataMember>> {
        let span = get_span_for(&input);
        let scope = get_scope(&input);
        Ok(match_nodes!(input.children();
            [prelude(prelude), identifier(identifier), tag_modifier(tag), typeref(mut data_type)] => {
                let (attributes, comment) = prelude;

                // Forward the member's attributes to the data type.
                // TODO: in the future we should only forward type metadata by filtering metadata.
                data_type.attributes = attributes.clone();

                let data_member = DataMember::new(
                    identifier,
                    data_type,
                    tag,
                    scope,
                    attributes,
                    comment,
                    span,
                );

                let ast = &mut input.user_data().borrow_mut().ast;
                ast.add_named_element(OwnedPtr::new(data_member))
            },
        ))
    }

    fn parameter_list(input: PestNode) -> PestResult<Vec<WeakPtr<Parameter>>> {
        Ok(match_nodes!(input.into_children();
            [] => Vec::new(),
            [parameter(parameter)] => vec![parameter],
            [parameter(parameter), parameter_list(mut list)] => {
                // The parameter comes before the parameter_list when parsing, so we have to
                // insert the new parameter at the front of the list.
                list.insert(0, parameter);
                list
            },
        ))
    }

    fn parameter(input: PestNode) -> PestResult<WeakPtr<Parameter>> {
        let span = get_span_for(&input);
        let scope = get_scope(&input);
        Ok(match_nodes!(input.children();
            [prelude(prelude), identifier(identifier), tag_modifier(tag), stream_modifier(is_streamed), typeref(mut data_type)] => {
                let (attributes, comment) = prelude;

                // Forward the member's attributes to the data type.
                // TODO: in the future we should only forward type metadata by filtering metadata.
                data_type.attributes = attributes.clone();

                let parameter = Parameter::new(
                    identifier,
                    data_type,
                    tag,
                    is_streamed,
                    input.user_data().borrow().is_in_return_tuple,
                    scope,
                    attributes,
                    comment,
                    span,
                );

                let ast = &mut input.user_data().borrow_mut().ast;
                ast.add_named_element(OwnedPtr::new(parameter))
            },
        ))
    }

    fn tag(input: PestNode) -> PestResult<u32> {
        Ok(match_nodes!(input.children();
            [_, integer(integer)] => {
                // Checking that tags must fit in an i32 and be non-negative.
                if !RangeInclusive::new(0, i32::MAX as i64).contains(&integer) {
                    let span = get_span_for(&input);
                    input.user_data().borrow_mut().error_reporter.report(LogicKind::TagValueOutOfBounds, Some(&span));
                }
                integer as u32
            }
        ))
    }

    fn tag_modifier(input: PestNode) -> PestResult<Option<u32>> {
        Ok(match_nodes!(input.into_children();
            [] => None,
            [tag(tag)] => Some(tag),
        ))
    }

    fn enumerator_list(input: PestNode) -> PestResult<Vec<WeakPtr<Enumerator>>> {
        Ok(match_nodes!(input.into_children();
            [enumerator(enumerator)] => {
                vec![enumerator]
            },
            [enumerator(enumerator), enumerator_list(mut list)] => {
                // The enumerator comes before the enumerator_list when parsing, so we have to
                // insert the new enumerator at the front of the list.
                list.insert(0, enumerator);
                list
            },
        ))
    }

    fn enumerator(input: PestNode) -> PestResult<WeakPtr<Enumerator>> {
        let span = get_span_for(&input);
        let scope = get_scope(&input);

        let enum_value: i64;

        let enumerator = match_nodes!(input.children();
            [prelude(prelude), identifier(ident)] => {
                let (attributes, comment) = prelude;

                // The user did not specify an enum value, so we increment the previous value.
                enum_value = match input.user_data().borrow().current_enum_value {
                    Some(value) if value == i64::MAX => {
                        let input_str = input.as_str();
                        Err(PestError::new_from_span(
                            PestErrorVariant::CustomError {
                                message: format!("Enumerator value out of range: {input_str}")
                            },
                        input.as_span(),
                    ))},
                    Some(value) => Ok(value + 1),
                    None => Ok(0),
                }?;

                Enumerator::new(ident, enum_value, scope, attributes, comment, span)
            },
            [prelude(prelude), identifier(ident), integer(value)] => {
                enum_value = value;
                let (attributes, comment) = prelude;
                Enumerator::new(ident, value, scope, attributes, comment, span)
            },
        );

        {
            let parser_data = &mut input.user_data().borrow_mut();
            parser_data.current_enum_value = Some(enum_value);
        }

        let ast = &mut input.user_data().borrow_mut().ast;
        Ok(ast.add_named_element(OwnedPtr::new(enumerator)))
    }

    fn inheritance_list(input: PestNode) -> PestResult<Vec<TypeRef>> {
        Ok(match_nodes!(input.into_children();
            [typeref(typeref)] => {
                vec![typeref]
            },
            [typeref(typeref), inheritance_list(mut list)] => {
                // The typename comes before the inheritance_list when parsing, so we have to
                // insert the new typename at the front of the list.
                list.insert(0, typeref);
                list
            },
        ))
    }

    fn type_alias(input: PestNode) -> PestResult<WeakPtr<TypeAlias>> {
        let span = get_span_for(&input);
        let scope = get_scope(&input);
        Ok(match_nodes!(input.children();
            [prelude(prelude), _, identifier(identifier), typeref(type_ref)] => {
                let (attributes, comment) = prelude;
                let type_alias = TypeAlias::new(identifier, type_ref, scope, attributes, comment, span);

                let ast = &mut input.user_data().borrow_mut().ast;
                ast.add_named_element(OwnedPtr::new(type_alias))
            },
        ))
    }

    fn typeref(input: PestNode) -> PestResult<TypeRef> {
        let span = get_span_for(&input);
        let mut nodes = input.children();

        // The first node is always a `local_attribute`. This is guaranteed by the grammar rules.
        let attributes = SliceParser::local_attributes(nodes.next().unwrap()).unwrap();
        // The second node is the type.
        let type_node = nodes.next().unwrap();

        // Get the typename as a string, with any whitespace removed from it.
        let type_name = type_node.as_str().chars().filter(|c| !c.is_whitespace()).collect();

        let is_optional = input.as_str().ends_with('?');
        let scope = get_scope(&input);
        let mut type_ref: TypeRef<dyn Type> = TypeRef::new(type_name, is_optional, scope, attributes, span);

        // Resolve and/or construct non user defined types.
        match type_node.as_rule() {
            Rule::primitive => {
                let primitive = Self::primitive(type_node).unwrap();
                type_ref.definition = upcast_weak_as!(primitive, dyn Type);
            }
            Rule::sequence => {
                let sequence = Self::sequence(type_node).unwrap();
                type_ref.definition = upcast_weak_as!(sequence, dyn Type);
            }
            Rule::dictionary => {
                let dictionary = Self::dictionary(type_node).unwrap();
                type_ref.definition = upcast_weak_as!(dictionary, dyn Type);
            }
            // Nothing to do, we wait until after we've generated a lookup table to patch user
            // defined types.
            _ => {}
        }
        Ok(type_ref)
    }

    fn sequence(input: PestNode) -> PestResult<WeakPtr<Sequence>> {
        Ok(match_nodes!(input.children();
            [_, typeref(element_type)] => {
                let sequence = Sequence { element_type };
                let ast = &mut input.user_data().borrow_mut().ast;
                ast.add_element(OwnedPtr::new(sequence))
            },
        ))
    }

    fn dictionary(input: PestNode) -> PestResult<WeakPtr<Dictionary>> {
        Ok(match_nodes!(input.children();
            [_, typeref(key_type), typeref(value_type)] => {
                let dictionary = Dictionary { key_type, value_type };
                let ast = &mut input.user_data().borrow_mut().ast;
                ast.add_element(OwnedPtr::new(dictionary))
            },
        ))
    }

    fn primitive(input: PestNode) -> PestResult<WeakPtr<Primitive>> {
        // Look the primitive up in the AST's primitive cache.
        Ok(input
            .user_data()
            .borrow()
            .ast
            .find_node(input.as_str())
            .unwrap()
            .try_into()
            .unwrap())
    }

    fn identifier(input: PestNode) -> PestResult<Identifier> {
        Ok(Identifier::new(input.as_str().to_owned(), get_span_for(&input)))
    }

    fn scoped_identifier(input: PestNode) -> PestResult<Identifier> {
        Ok(Identifier::new(input.as_str().to_owned(), get_span_for(&input)))
    }

    fn global_identifier(input: PestNode) -> PestResult<Identifier> {
        Ok(Identifier::new(input.as_str().to_owned(), get_span_for(&input)))
    }

    fn prelude(input: PestNode) -> PestResult<(Vec<Attribute>, Option<DocComment>)> {
        Ok(match_nodes!(input.into_children();
            [local_attributes(mut attributes1), doc_comment(comment), local_attributes(attributes2)] => {
                // Combine the attributes into a single list, by moving the elements of 2 into 1.
                attributes1.extend(attributes2);
                (attributes1, comment)
            },
        ))
    }

    fn file_attributes(input: PestNode) -> PestResult<Vec<Attribute>> {
        Ok(match_nodes!(input.into_children();
            [attribute(attributes)..] => attributes.collect(),
        ))
    }

    fn local_attributes(input: PestNode) -> PestResult<Vec<Attribute>> {
        Ok(match_nodes!(input.into_children();
            [attribute(attributes)..] => attributes.collect(),
        ))
    }

    fn attribute(input: PestNode) -> PestResult<Attribute> {
        let span = get_span_for(&input);

        Ok(match_nodes!(input.into_children();
            [attribute_directive(attribute)] => {
                let (prefix, directive) = attribute;
                Attribute::new(prefix, directive, Vec::new(), span)
            },
            [attribute_directive(attribute), attribute_arguments(arguments)] => {
                let (prefix, directive) = attribute;
                Attribute::new(prefix, directive, arguments, span)
            },
        ))
    }

    fn attribute_directive(input: PestNode) -> PestResult<(Option<String>, String)> {
        Ok(match_nodes!(input.into_children();
            [attribute_identifier(name)] => (None, name),
            [attribute_identifier(prefix), attribute_identifier(name)] => (Some(prefix), name)
        ))
    }

    fn attribute_identifier(input: PestNode) -> PestResult<String> {
        Ok(input.as_str().to_owned())
    }

    fn attribute_argument(input: PestNode) -> PestResult<String> {
        let argument = input.as_str();
        // If the argument was wrapped in quotes, remove them.
        if argument.starts_with('"') && argument.ends_with('"') {
            let mut chars = argument.chars();
            // Skip the first and last characters (they're just quotes).
            chars.next();
            chars.next_back();
            Ok(chars.collect::<String>())
        } else {
            Ok(argument.to_owned())
        }
    }

    fn attribute_arguments(input: PestNode) -> PestResult<Vec<String>> {
        Ok(match_nodes!(input.into_children();
            [attribute_argument(argument)] => {
                vec![argument]
            },
            [attribute_argument(argument), attribute_arguments(mut list)] => {
                // The argument comes before the rest of the arguments when parsing, so we have to
                // insert the new argument at the front of the list.
                list.insert(0, argument);
                list
            },
        ))
    }

    fn doc_comment(input: PestNode) -> PestResult<Option<DocComment>> {
        let span = get_span_for(&input);
        Ok(match_nodes!(input.into_children();
            [] => {
                None
            },
            [line_doc_comment(comments)..] => {
                // Merge all the line comments together.
                let combined = comments.collect::<Vec<String>>().join("\n");
                Some(CommentParser::parse_doc_comment(&combined, span))
            },
            [block_doc_comment(comment)] => {
                Some(CommentParser::parse_doc_comment(&comment, span))
            }
        ))
    }

    fn line_doc_comment(input: PestNode) -> PestResult<String> {
        Ok(input.as_str().to_owned())
    }

    fn block_doc_comment(input: PestNode) -> PestResult<String> {
        Ok(input.as_str().to_owned())
    }

    fn integer(input: PestNode) -> PestResult<i64> {
        let int = input.as_str().parse::<i64>();
        match int {
            Ok(int) => Ok(int),
            Err(err) => Err(PestError::new_from_span(
                PestErrorVariant::CustomError {
                    message: format!("Failed to parse integer: {err}"),
                },
                input.as_span(),
            )),
        }
    }

    fn compact_id(input: PestNode) -> PestResult<Option<u32>> {
        Ok(match_nodes!(input.into_children();
            []               => None,
            [integer(value)] => {
                // compact ids must fit in an i32 and be non-negative.
                if value < 0 || value > i32::MAX.into() {
                    // TODO let span = from_span(&input);
                    // TODO let error_string = if integer < 0 {
                    // TODO     format!("ID is out of range: {}. Compact IDs must be positive", integer)
                    // TODO } else {
                    // TODO     format!(
                    // TODO         "ID is out of range: {}. Compact IDs must be less than {}",
                    // TODO         integer, i32::MAX
                    // TODO     )
                    // TODO };
                    // TODO report an error here!
                }
                Some(value as u32)
            }
        ))
    }

    fn stream_modifier(input: PestNode) -> PestResult<bool> {
        Ok(match_nodes!(input.into_children();
            []                => false,
            [stream_kw(_)] => true
        ))
    }

    fn compact_modifier(input: PestNode) -> PestResult<bool> {
        Ok(match_nodes!(input.into_children();
            []              => false,
            [compact_kw(_)] => true
        ))
    }

    fn idempotent_modifier(input: PestNode) -> PestResult<bool> {
        Ok(match_nodes!(input.into_children();
            []                 => false,
            [idempotent_kw(_)] => true
        ))
    }

    fn unchecked_modifier(input: PestNode) -> PestResult<bool> {
        Ok(match_nodes!(input.into_children();
            []                => false,
            [unchecked_kw(_)] => true
        ))
    }

    fn module_kw(input: PestNode) -> PestResult<()> {
        Ok(())
    }

    fn struct_kw(input: PestNode) -> PestResult<()> {
        Ok(())
    }

    fn class_kw(input: PestNode) -> PestResult<()> {
        Ok(())
    }

    fn exception_kw(input: PestNode) -> PestResult<()> {
        Ok(())
    }

    fn interface_kw(input: PestNode) -> PestResult<()> {
        Ok(())
    }

    fn enum_kw(input: PestNode) -> PestResult<()> {
        Ok(())
    }

    fn type_alias_kw(input: PestNode) -> PestResult<()> {
        Ok(())
    }

    fn trait_kw(input: PestNode) -> PestResult<()> {
        Ok(())
    }

    fn custom_kw(input: PestNode) -> PestResult<()> {
        Ok(())
    }

    fn sequence_kw(input: PestNode) -> PestResult<()> {
        Ok(())
    }

    fn dictionary_kw(input: PestNode) -> PestResult<()> {
        Ok(())
    }

    fn bool_kw(input: PestNode) -> PestResult<()> {
        Ok(())
    }

    fn int8_kw(input: PestNode) -> PestResult<()> {
        Ok(())
    }

    fn uint8_kw(input: PestNode) -> PestResult<()> {
        Ok(())
    }

    fn int16_kw(input: PestNode) -> PestResult<()> {
        Ok(())
    }

    fn uint16_kw(input: PestNode) -> PestResult<()> {
        Ok(())
    }

    fn int32_kw(input: PestNode) -> PestResult<()> {
        Ok(())
    }

    fn uint32_kw(input: PestNode) -> PestResult<()> {
        Ok(())
    }

    fn varint32_kw(input: PestNode) -> PestResult<()> {
        Ok(())
    }

    fn varuint32_kw(input: PestNode) -> PestResult<()> {
        Ok(())
    }

    fn int64_kw(input: PestNode) -> PestResult<()> {
        Ok(())
    }

    fn uint64_kw(input: PestNode) -> PestResult<()> {
        Ok(())
    }

    fn varint62_kw(input: PestNode) -> PestResult<()> {
        Ok(())
    }

    fn varuint62_kw(input: PestNode) -> PestResult<()> {
        Ok(())
    }

    fn float32_kw(input: PestNode) -> PestResult<()> {
        Ok(())
    }

    fn float64_kw(input: PestNode) -> PestResult<()> {
        Ok(())
    }

    fn string_kw(input: PestNode) -> PestResult<()> {
        Ok(())
    }

    fn any_class_kw(input: PestNode) -> PestResult<()> {
        Ok(())
    }

    fn tag_kw(input: PestNode) -> PestResult<()> {
        Ok(())
    }

    fn stream_kw(input: PestNode) -> PestResult<()> {
        Ok(())
    }

    fn extends_kw(input: PestNode) -> PestResult<()> {
        Ok(())
    }

    fn compact_kw(input: PestNode) -> PestResult<()> {
        Ok(())
    }

    fn idempotent_kw(input: PestNode) -> PestResult<()> {
        Ok(())
    }

    fn unchecked_kw(input: PestNode) -> PestResult<()> {
        Ok(())
    }

    fn encoding_kw(input: PestNode) -> PestResult<()> {
        Ok(())
    }

    fn EOI(input: PestNode) -> PestResult<()> {
        Ok(())
    }
}

impl<'a> SliceParser<'a> {
    fn parse_module(input: PestNode, allow_sub_modules: bool) -> PestResult<WeakPtr<Module>> {
        Ok(match_nodes!(input.children();
            [prelude(prelude), module_start(module_start), definition(definitions)..] => {
                let (identifier, span) = module_start;
                let (attributes, comment) = prelude;

                // Split the identifier in case it uses nested module syntax.
                // We iterate in reverse, since we construct them in inner-to-outermost order.
                let mut modules = identifier.value.rsplit("::");

                // Pop the scope of the inner-most module (the module can't be in its own scope).
                pop_scope(&input);
                // Construct the inner-most module first.
                let mut last_module = Module::new(
                    // There must be at least one module identifier, so it's safe to unwrap here.
                    Identifier::new(
                        modules.next().unwrap().to_owned(),
                        identifier.span.clone(),
                    ),
                    get_scope(&input),
                    attributes,
                    comment,
                    span.clone(),
                );
                // Add the definitions into the inner-most module.
                for definition in definitions {
                    // Report an error if sub-modules aren't allowed and the definition is a module.
                    // Files using a file-level module don't support module nesting within the file.
                    if !allow_sub_modules {
                        if let Definition::Module(module_def) = &definition {
                            let error_reporter = &mut input.user_data().borrow_mut().error_reporter;

                            error_reporter.report(
                                ErrorKind::Syntax("file level modules cannot contain sub-modules".to_owned()),
                                Some(&module_def.borrow().span),
                            );

                            error_reporter.report(
                                ErrorKind::new_note(format!("file level module '{}' declared here", &identifier.value)),
                                Some(&span),
                            );
                        }
                    }
                    last_module.add_definition(definition);
                }

                // Construct any enclosing modules.
                for module in modules {
                    // Pop the module's scope, and then construct it.
                    pop_scope(&input);
                    let mut new_module = Module::new(
                        Identifier::new(module.to_owned(), identifier.span.clone()),
                        get_scope(&input),
                        Vec::new(),
                        None,
                        span.clone(),
                    );
                    // Add the inner module to the outer module, than swap their variables.
                    let ast = &mut input.user_data().borrow_mut().ast;
                    new_module.add_definition(Definition::Module(ast.add_named_element(OwnedPtr::new(last_module))));
                    last_module = new_module;
                }

                // Return the outer-most module.
                let ast = &mut input.user_data().borrow_mut().ast;
                ast.add_named_element(OwnedPtr::new(last_module))
            },
        ))
    }
}
