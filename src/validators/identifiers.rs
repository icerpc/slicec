// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::error::Error;
use crate::grammar::*;
use crate::validators::{Validate, ValidationChain, ValidationResult};

pub fn identifier_validators() -> ValidationChain {
    vec![
        Validate::Identifiers(check_for_redefinition),
        Validate::InheritedIdentifiers(check_for_shadowing),
    ]
}

pub fn check_for_redefinition(identifiers: Vec<&Identifier>) -> ValidationResult {
    let mut errors = vec![];
    let mut identifiers = identifiers;
    // Sort first so that we can use windows to search for duplicates.
    identifiers.sort_by_key(|identifier| identifier.value.to_owned());
    identifiers.windows(2).for_each(|window| {
        if window[0].value == window[1].value {
            errors.push(Error {
                message: format!("redefinition of {}", window[1].value),
                location: Some(window[1].location.clone()),
                severity: crate::error::ErrorLevel::Error,
            });
            errors.push(Error {
                message: format!("{} was previously defined here", window[0].value),
                location: Some(window[0].location.clone()),
                severity: crate::error::ErrorLevel::Error,
            });
        }
    });
    match errors.is_empty() {
        true => Ok(()),
        false => Err(errors),
    }
}

pub fn check_for_shadowing(
    identifiers: Vec<&Identifier>,
    inherited_symbols: Vec<&Identifier>,
) -> ValidationResult {
    let mut errors = vec![];
    identifiers.iter().for_each(|identifier| {
        inherited_symbols
            .iter()
            .filter(|inherited_identifier| inherited_identifier.value == identifier.value)
            .for_each(|inherited_identifier| {
                errors.push(Error {
                    message: format!("{} shadows another symbol", identifier.value),
                    location: Some(identifier.location.clone()),
                    severity: crate::error::ErrorLevel::Error,
                });
                errors.push(Error {
                    message: format!("{} was previously defined here", inherited_identifier.value),
                    location: Some(inherited_identifier.location.clone()),
                    severity: crate::error::ErrorLevel::Error,
                });
            });
    });
    match errors.is_empty() {
        true => Ok(()),
        false => Err(errors),
    }
}

trait EntityIdentifiersExtension {
    fn get_identifiers(&self) -> Vec<&Identifier>;
}

impl<T> EntityIdentifiersExtension for Vec<&T>
where
    T: Entity,
{
    fn get_identifiers(&self) -> Vec<&Identifier> {
        self.iter().map(|member| member.raw_identifier()).collect()
    }
}
