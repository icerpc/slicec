// Copyright (c) ZeroC, Inc.

use super::super::*;
use crate::slice_file::Span;
use crate::utils::ptr_util::WeakPtr;

#[derive(Debug)]
pub struct TypeRef<T: Element + ?Sized = dyn Type> {
    pub definition: TypeRefDefinition<T>,
    pub is_optional: bool,
    pub scope: Scope,
    pub attributes: Vec<WeakPtr<Attribute>>,
    pub span: Span,
}

impl<T: Element + ?Sized> TypeRef<T> {
    pub fn definition(&self) -> &T {
        match &self.definition {
            TypeRefDefinition::Patched(ptr) => ptr.borrow(),
            _ => panic!("dereferenced unpatched type reference"),
        }
    }

    pub(crate) fn patch(&mut self, ptr: WeakPtr<T>, additional_attributes: Vec<WeakPtr<Attribute>>) {
        // Assert that the typeref hasn't already been patched.
        debug_assert!(matches!(&self.definition, TypeRefDefinition::Unpatched(_)));

        self.definition = TypeRefDefinition::Patched(ptr);
        self.attributes.extend(additional_attributes);
    }

    pub(crate) fn downcast<U: Element + 'static>(&self) -> Result<TypeRef<U>, ()> {
        let definition = match &self.definition {
            TypeRefDefinition::Patched(ptr) => match ptr.clone().downcast::<U>() {
                Ok(new_ptr) => TypeRefDefinition::Patched(new_ptr),
                Err(_) => return Err(()),
            },
            TypeRefDefinition::Unpatched(identifier) => TypeRefDefinition::Unpatched(identifier.clone()),
        };

        Ok(TypeRef {
            definition,
            is_optional: self.is_optional,
            scope: self.scope.clone(),
            attributes: self.attributes.clone(),
            span: self.span.clone(),
        })
    }
}

impl<T: Type + ?Sized> TypeRef<T> {
    // This intentionally shadows the trait method of the same name on `Type`.
    pub fn type_string(&self) -> String {
        let mut s = self.definition().type_string();
        if self.is_optional {
            s += "?";
        }
        s
    }

    // This intentionally shadows the trait method of the same name on `Type`.
    pub fn fixed_wire_size(&self) -> Option<u32> {
        if self.is_optional {
            None
        } else {
            T::fixed_wire_size(self)
        }
    }
}

impl<T: Element + ?Sized> Attributable for TypeRef<T> {
    fn attributes(&self, include_parent: bool) -> Vec<&Attribute> {
        assert!(!include_parent);
        self.attributes.iter().map(WeakPtr::borrow).collect()
    }

    fn all_attributes(&self) -> Vec<Vec<&Attribute>> {
        vec![self.attributes(false)]
    }
}

impl<T: Element + ?Sized> std::ops::Deref for TypeRef<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.definition()
    }
}

implement_Element_for!(TypeRef<T>, "type reference", Element + ?Sized);
implement_Symbol_for!(TypeRef<T>, Element + ?Sized);
implement_Scoped_Symbol_for!(TypeRef<T>, Element + ?Sized);

#[derive(Debug)]
pub enum TypeRefDefinition<T: Element + ?Sized = dyn Type> {
    Patched(WeakPtr<T>),
    Unpatched(Identifier),
}
