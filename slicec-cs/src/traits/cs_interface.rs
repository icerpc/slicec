// Copyright (c) ZeroC, Inc. All rights reserved.

use slice::grammar::{Interface, NamedSymbol};
use slice::util::{fix_case, CaseStyle};

pub trait CsInterfaceInfo {
    /// The name of the generated C# interface for this Slice interface.
    /// eg. If the slice interface is `Foo`, the C# interface is `IFoo`.
    /// The name is always prefixed with `I` and the first letter is always
    /// capitalized.
    fn interface_name(&self) -> String;

    /// Name of the generated implementation struct for this Slice interface's proxy.
    /// eg. If teh slice interface is `Foo`, the C# proxy implementation is `FooPrx`.
    fn proxy_implementation_name(&self) -> String {
        self.proxy_name().chars().skip(1).collect()
    }

    /// The name of the generated C# proxy struct for this Slice interface.
    /// eg. If teh slice interface is `Foo`, the C# proxy is `IFooPrx`.
    fn proxy_name(&self) -> String {
        self.interface_name() + "Prx"
    }
}

impl CsInterfaceInfo for Interface {
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
}
