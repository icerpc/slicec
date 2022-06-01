// Copyright (c) ZeroC, Inc. All rights reserved.
use crate::error::Error;
use crate::grammar::*;
use crate::visitor::Visitor;

pub enum ValidationFunction {
    Attributable(Box<dyn Fn(&Attributable) -> Result<(), Vec<Error>>>),
    Class(Box<dyn Fn(&Class) -> Result<(), Vec<Error>>>),
    Members(Box<dyn Fn(&[&DataMember]) -> Result<(), Vec<Error>>>),
    Parameters(Box<dyn Fn(&[&Parameter]) -> Result<(), Vec<Error>>>),
    Parameter(Box<dyn Fn(&Parameter) -> Result<(), Vec<Error>>>),
    Struct(Box<dyn Fn(&Struct) -> Result<(), Vec<Error>>>),
    Enums(Box<dyn Fn(&Enum) -> Result<(), Vec<Error>>>),
    Interface(Box<dyn Fn(&Interface) -> Result<(), Vec<Error>>>),
    Operation(Box<dyn Fn(&Operation) -> Result<(), Vec<Error>>>),
    Exception(Box<dyn Fn(&[&DataMember]) -> Result<(), Vec<Error>>>),
}

pub struct ReeceValidatorThing {
    errors: Vec<Error>,
    validation_functions: Vec<ValidationFunction>,
}

impl ReeceValidatorThing {
    pub fn new() -> ReeceValidatorThing {
        ReeceValidatorThing { errors: Vec::new(), validation_functions: Vec::new() }
    }

    pub fn add_validation_functions(&mut self, validation_functions: Vec<ValidationFunction>) {
        self.validation_functions.extend(validation_functions);
    }

    pub fn errors(&self) -> &Vec<Error> {
        &self.errors
    }
}

impl Visitor for ReeceValidatorThing {
    fn visit_class_start(&mut self, class_def: &Class) {
        let mut errors = vec![];

        self.validation_functions
            .iter()
            .filter_map(|function| match function {
                ValidationFunction::Class(function) => Some(function(class_def)),
                ValidationFunction::Members(function) => {
                    Some(function(class_def.members().as_slice()))
                }
                ValidationFunction::Attributable(function) => Some(function(class_def)),
                _ => None,
            })
            .for_each(|result| match result {
                Ok(_) => (),
                Err(mut errs) => errors.append(&mut errs),
            });
        self.errors.append(&mut errors);
    }

    fn visit_struct_start(&mut self, struct_def: &Struct) {
        let mut errors = vec![];
        self.validation_functions
            .iter()
            .filter_map(|function| match function {
                ValidationFunction::Struct(function) => Some(function(struct_def)),
                ValidationFunction::Members(function) => {
                    Some(function(struct_def.members().as_slice()))
                }
                ValidationFunction::Attributable(function) => Some(function(struct_def)),
                _ => None,
            })
            .for_each(|result| match result {
                Ok(_) => (),
                Err(mut errs) => errors.append(&mut errs),
            });
        self.errors.append(&mut errors);
    }

    fn visit_enum_start(&mut self, enum_def: &Enum) {
        let mut errors = vec![];
        self.validation_functions
            .iter()
            .filter_map(|function| match function {
                ValidationFunction::Enums(function) => Some(function(enum_def)),
                ValidationFunction::Attributable(function) => Some(function(enum_def)),
                _ => None,
            })
            .for_each(|result| match result {
                Ok(_) => (),
                Err(mut errs) => errors.append(&mut errs),
            });
        self.errors.append(&mut errors);
    }

    fn visit_interface_start(&mut self, interface_def: &Interface) {
        let mut errors = vec![];
        self.validation_functions
            .iter()
            .filter_map(|function| match function {
                ValidationFunction::Interface(function) => Some(function(interface_def)),
                ValidationFunction::Attributable(function) => Some(function(interface_def)),
                _ => None,
            })
            .for_each(|result| match result {
                Ok(_) => (),
                Err(mut errs) => errors.append(&mut errs),
            });
        self.errors.append(&mut errors);
    }

    fn visit_operation_start(&mut self, operation: &Operation) {
        let mut errors = vec![];
        self.validation_functions
            .iter()
            .filter_map(|function| match function {
                ValidationFunction::Operation(function) => Some(function(operation)),
                ValidationFunction::Attributable(function) => Some(function(operation)),
                ValidationFunction::Parameters(function) => {
                    Some(function(operation.parameters().as_slice()))
                }
                _ => None,
            })
            .for_each(|result| match result {
                Ok(_) => (),
                Err(mut errs) => errors.append(&mut errs),
            });
        self.errors.append(&mut errors);
    }

    fn visit_parameter(&mut self, parameter: &Parameter) {
        let mut errors = vec![];
        self.validation_functions
            .iter()
            .filter_map(|function| match function {
                ValidationFunction::Parameter(function) => Some(function(parameter)),
                ValidationFunction::Attributable(function) => Some(function(parameter)),
                _ => None,
            })
            .for_each(|result| match result {
                Ok(_) => (),
                Err(mut errs) => errors.append(&mut errs),
            });
        self.errors.append(&mut errors);
    }
}
