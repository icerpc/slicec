// Copyright (c) ZeroC, Inc. All rights reserved.

use slice::grammar::{Interface, NamedSymbol};
use slice::util::{fix_case, CaseStyle};

pub trait CsInterface {
    fn interface_name(&self) -> String;
    fn proxy_name(&self) -> String;
    fn proxy_implementation_name(&self) -> String;

    fn format_type(&self) -> String;
}

impl CsInterface for Interface {
    fn interface_name(&self) -> String {
        let identifier = fix_case(self.identifier(), CaseStyle::Pascal);
        let mut chars = identifier.chars();

        // Check if the interface already follows the 'I' prefix convention.
        if identifier.chars().count() > 2
            && chars.next().unwrap() == 'I'
            && chars.next().unwrap().is_uppercase()
        {
            identifier.to_owned()
        } else {
            format!("I{}", identifier)
        }
    }

    fn proxy_name(&self) -> String {
        self.interface_name() + "Prx"
    }

    fn proxy_implementation_name(&self) -> String {
        self.proxy_name().chars().skip(1).collect()
    }

    fn format_type(&self) -> String {
        // TODO: Austin - Implement this :)
        "default".to_owned()
    }
}
