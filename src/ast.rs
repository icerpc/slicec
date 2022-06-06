// Copyright (c) ZeroC, Inc. All rights reserved.

// TODO this file needs some cleanup now.

use crate::{downgrade_as, upcast_owned_as, upcast_weak_as};

use crate::grammar::*;
use crate::ptr_util::{OwnedPtr, WeakPtr};
use crate::ptr_visitor::PtrVisitor;
use std::collections::HashMap;

/// The AST (Abstract Syntax Tree) is the heart of the compiler, containing all the
/// slice elements defined by, and used by, the slice files passed into the compiler.
///
/// There is a single instance of the AST per-compile, which is created during initialization.
/// The parser parses each file into its top level modules, and then moves
/// these modules directly into the shared AST.
///
/// In addition, the AST owns all anonymous types, which are created in the AST as the parser needs
/// them, and it owns all the Slice [Primitive]s, which are added to the AST during initialization,
/// ensuring they're always available.
///
/// The AST is primarily for centralizing ownership of Slice elements, but also features lookup
/// methods for retrieving [primitives](Ast::lookup_primitive), [types](Ast::lookup_type), and
/// [entities](Ast::lookup_entity) by name.
///
/// The AST effectively has a `'static` lifetime. It is created when the compiler starts execution
/// (in [global_state::initialize]), and it stays in scope until the program terminates.
#[derive(Debug)]
pub struct Ast {
    /// The AST vector owns all the top level modules defined in every slice file parsed by the
    /// compiler. These modules in-turn own all their contents. Hence, this vector contains all
    /// **user** defined slice elements. All compiler defined types are stored in either
    /// [anonymous_types] or [primitive_cache].
    pub(crate) ast: Vec<OwnedPtr<Module>>,

    /// This vector owns all the non-primitive anonymous types ([Sequence]s and [Dictionary]s).
    /// These types aren't defined by users, but are still usable in slice definitions behind a
    /// [TypeRef].
    ///
    /// Since these types aren't defined by users, they can't be owned by a module.
    /// Instead they are owned by the AST, and stored here, so that they can be referenced via
    /// [WeakPtr] elsewhere. Types are stored in the order they're parsed, and are only created
    /// when needed by the parser.
    pub(crate) anonymous_types: Vec<OwnedPtr<dyn Type>>,

    /// This cache holds the definitions for all the Slice primitive types, keyed by their Slice
    /// keywords. Primitives are built-in to the compiler, and are always defined, even if
    /// they're not needed. They are kept in a separate cache for stronger typing (this stores
    /// `Primitive`s instead of just `dyn Type`s), and to prevent excessive copies of
    /// primitives being created. A single instance per-primitive is safe, as primitives are
    /// not scope-sensitive, unlike other anonymous types.
    pub(crate) primitive_cache: HashMap<&'static str, OwnedPtr<Primitive>>,

    /// This lookup table stores [WeakPtr]s for every user defined entity stored in this AST.
    /// Each [Entity]'s fully scoped identifier is used as its key in the table.
    pub(crate) lookup_table: HashMap<String, WeakPtr<dyn Entity>>,
}

impl Ast {
    pub(crate) fn new() -> Ast {
        // Primitive types are built in to the compiler. Since they aren't defined in Slice,
        // we 'define' them here when the AST is created, to ensure they're always available.
        let primitive_cache = HashMap::from([
            ("bool", OwnedPtr::new(Primitive::Bool)),
            ("int8", OwnedPtr::new(Primitive::Int8)),
            ("uint8", OwnedPtr::new(Primitive::UInt8)),
            ("int16", OwnedPtr::new(Primitive::Int16)),
            ("uint16", OwnedPtr::new(Primitive::UInt16)),
            ("int32", OwnedPtr::new(Primitive::Int32)),
            ("uint32", OwnedPtr::new(Primitive::UInt32)),
            ("varint32", OwnedPtr::new(Primitive::VarInt32)),
            ("varuint32", OwnedPtr::new(Primitive::VarUInt32)),
            ("int64", OwnedPtr::new(Primitive::Int64)),
            ("uint64", OwnedPtr::new(Primitive::UInt64)),
            ("varint62", OwnedPtr::new(Primitive::VarInt62)),
            ("varuint62", OwnedPtr::new(Primitive::VarUInt62)),
            ("float32", OwnedPtr::new(Primitive::Float32)),
            ("float64", OwnedPtr::new(Primitive::Float64)),
            ("string", OwnedPtr::new(Primitive::String)),
            ("AnyClass", OwnedPtr::new(Primitive::AnyClass)),
        ]);

        // Create an empty AST (apart from the primitive cache).
        Ast {
            ast: Vec::new(),
            anonymous_types: Vec::new(),
            primitive_cache,
            lookup_table: HashMap::new(),
        }
    }

    /// Moves a [Module] into the AST, and returns a [WeakPtr] to it.
    /// It also visits through the module to index it and its contents into the AST's lookup tables.
    ///
    /// This should only be called by the parser.
    pub(crate) fn add_module(&mut self, module_def: Module) -> WeakPtr<Module> {
        // Move the module onto the heap so it can be referenced via pointer.
        let mut module_ptr = OwnedPtr::new(module_def);

        // Create a visitor for adding the module's contents into the AST's lookup tables.
        let mut visitor = LookupTableBuilder {
            lookup_table: &mut self.lookup_table,
        };

        // Add the module into the lookup tables, then recursively add it's contents too.
        //
        // This is always safe; no other references to the module can exist because we own it,
        // and haven't dereferenced any of the pointers to it that we've constructed.
        unsafe {
            module_ptr.visit_ptr_with(&mut visitor);
        }

        // Move the module into the AST and return a WeakPtr to it.
        let weak_ptr = module_ptr.downgrade();
        self.ast.push(module_ptr);
        weak_ptr
    }

    /// Moves an anonymous [Type] into the AST.
    ///
    /// Anonymous types are types that aren't defined by users, but still usable as a type.
    /// Currently this only includes [Sequence]s and [Dictionary]s.
    /// [Primitive]s are also technically anonymous types, but are stored in
    /// their own [cache](primitive_cache), instead of the [anonymous_types] store.
    ///
    /// A separate instance of the anonymous type is created for each place it's used,
    /// even if they're the same type.
    /// ```slice
    /// operation(sequence<int32> i1, sequence<int32> i2); // 2 anonymous types are created here, 1 for each sequence.
    /// ```
    /// Additionally, no indexing is performed on these types since they're un-named and unique to
    /// where they're used. Instead, this function returns a reference to the [OwnedPtr] storing
    /// the type, since there's no way to access the type through a lookup table later.
    ///
    /// This should only be called by the parser.
    pub(crate) fn add_anonymous_type(&mut self, ty: impl Type + 'static) -> &OwnedPtr<dyn Type> {
        let type_ptr = upcast_owned_as!(OwnedPtr::new(ty), dyn Type);
        self.anonymous_types.push(type_ptr);
        self.anonymous_types.last().unwrap()
    }

    pub fn find_entity(&self, fully_scoped_identifier: &str) -> Option<WeakPtr<dyn Entity>> {
        self.lookup_table.get(fully_scoped_identifier).cloned()
    }

    pub fn find_typed_entity<T: Entity + 'static>(&self, fully_scoped_identifier: &str) -> Option<WeakPtr<T>> {
        let entity_ptr = self.find_entity(fully_scoped_identifier);
        entity_ptr.and_then(|ptr| ptr.downcast::<T>().ok())
    }

    pub fn find_type(&self, fully_scoped_identifier: &str) -> Option<WeakPtr<dyn Type>> {
        let result = Self::lookup_type(
            &self.lookup_table,
            &self.primitive_cache,
            fully_scoped_identifier,
            &[], // We always look up types by their fully scoped identifiers.
        );
        result.ok()
    }

    pub fn find_typed_type<T: Type + 'static>(&self, fully_scoped_identifier: &str) -> Option<WeakPtr<T>> {
        let type_ptr = self.find_type(fully_scoped_identifier);
        type_ptr.and_then(|ptr| ptr.downcast::<T>().ok())
    }

    // =============================================================================================
    // These lookup functions are associated functions instead of methods so that the AST can be
    // mutated without locking down access to them.
    // Methods require borrowing the entire AST, which is impossible if some of its contents have
    // been mutably borrowed somewhere else (such as while visiting, or patching).
    // =============================================================================================

    pub fn lookup_entity<'ast>(
        lookup_table: &'ast HashMap<String, WeakPtr<dyn Entity>>,
        identifier: &str,
        mut scopes: &[String],
    ) -> Option<&'ast WeakPtr<dyn Entity>> {
        // If the identifier starts with '::', it's a global identifier, which can be looked up
        // directly.
        if let Some(unprefixed) = identifier.strip_prefix("::") {
            return lookup_table.get(unprefixed);
        }

        // For relative paths, we check each enclosing scope, starting from the bottom
        // (most specific scope), and working our way up to global scope.
        while !scopes.is_empty() {
            let candidate = scopes.join("::") + "::" + identifier;
            if let Some(result) = lookup_table.get(&candidate) {
                return Some(result);
            }
            // Remove the last parent's scope before trying again.
            // It's safe to unwrap here, since we know that `scopes` is not empty.
            scopes = scopes.split_last().unwrap().1;
        }

        // Check for the entity at global scope (without any parent scopes).
        if let Some(result) = lookup_table.get(&identifier.to_owned()) {
            return Some(result);
        }

        // The entity couldn't be found.
        None
    }

    pub fn lookup_type<'ast>(
        lookup_table: &'ast HashMap<String, WeakPtr<dyn Entity>>,
        primitive_cache: &'ast HashMap<&'static str, OwnedPtr<Primitive>>,
        identifier: &str,
        scopes: &[String],
    ) -> Result<WeakPtr<dyn Type>, Option<WeakPtr<dyn Entity>>> {
        if let Some(primitive_ptr) = primitive_cache.get(identifier) {
            Ok(downgrade_as!(primitive_ptr, dyn Type))
        } else if let Some(entity_ptr) = Self::lookup_entity(lookup_table, identifier, scopes) {
            match entity_ptr.borrow().concrete_entity() {
                Entities::Struct(_) => Ok(upcast_weak_as!(
                    entity_ptr.clone().downcast::<Struct>().ok().unwrap(),
                    dyn Type
                )),
                Entities::Exception(_) => Ok(upcast_weak_as!(
                    entity_ptr.clone().downcast::<Exception>().ok().unwrap(),
                    dyn Type
                )),
                Entities::Class(_) => Ok(upcast_weak_as!(
                    entity_ptr.clone().downcast::<Class>().ok().unwrap(),
                    dyn Type
                )),
                Entities::Interface(_) => Ok(upcast_weak_as!(
                    entity_ptr.clone().downcast::<Interface>().ok().unwrap(),
                    dyn Type
                )),
                Entities::Enum(_) => Ok(upcast_weak_as!(
                    entity_ptr.clone().downcast::<Enum>().ok().unwrap(),
                    dyn Type
                )),
                Entities::Trait(_) => Ok(upcast_weak_as!(
                    entity_ptr.clone().downcast::<Trait>().ok().unwrap(),
                    dyn Type
                )),
                Entities::CustomType(_) => Ok(upcast_weak_as!(
                    entity_ptr.clone().downcast::<CustomType>().ok().unwrap(),
                    dyn Type
                )),
                Entities::TypeAlias(_) => Ok(upcast_weak_as!(
                    entity_ptr.clone().downcast::<TypeAlias>().ok().unwrap(),
                    dyn Type
                )),
                _ => Err(Some(entity_ptr.clone())),
            }
        } else {
            Err(None)
        }
    }

    pub fn lookup_primitive(&self, identifier: &str) -> &OwnedPtr<Primitive> {
        self.primitive_cache
            .get(identifier)
            .unwrap_or_else(|| panic!("No primitive type exists with the name '{}'", identifier))
    }
}

/// This struct is used to populate the lookup tables held by the AST.
///
/// When a module is added to the AST with [Ast::add_module], we construct a new
/// instance of [LookupTableBuilder] and visit through the module with it.
/// While visiting, the builder adds entries into the AST's lookup tables.
///
/// Because the builder holds mutable references to the AST's tables, it's
/// lightweight and short-lived to avoid locking up the AST.
///
/// It is only used internally by the [Ast].
struct LookupTableBuilder<'ast> {
    /// Mutable reference to the AST's [Ast::lookup_table].
    ///
    /// Whenever the builder visits a slice [Entity], it inserts a corresponding entry into this
    /// table. Each entry consists of the entity's fully scoped identifier, and a [WeakPtr] to it.
    lookup_table: &'ast mut HashMap<String, WeakPtr<dyn Entity>>,
}

impl<'ast> LookupTableBuilder<'ast> {
    /// Adds an entry into the AST's [Ast::lookup_table] for the provided definition.
    fn add_entry<T: Entity + 'static>(&mut self, definition: &OwnedPtr<T>) {
        let identifier = definition.borrow().parser_scoped_identifier();
        let weak_ptr = downgrade_as!(definition, dyn Entity);
        self.lookup_table.insert(identifier, weak_ptr);
    }
}

impl<'ast> PtrVisitor for LookupTableBuilder<'ast> {
    unsafe fn visit_module_start(&mut self, module_ptr: &mut OwnedPtr<Module>) {
        self.add_entry(module_ptr);
    }

    unsafe fn visit_struct_start(&mut self, struct_ptr: &mut OwnedPtr<Struct>) {
        self.add_entry(struct_ptr);
    }

    unsafe fn visit_class_start(&mut self, class_ptr: &mut OwnedPtr<Class>) {
        self.add_entry(class_ptr);
    }

    unsafe fn visit_exception_start(&mut self, exception_ptr: &mut OwnedPtr<Exception>) {
        self.add_entry(exception_ptr);
    }

    unsafe fn visit_interface_start(&mut self, interface_ptr: &mut OwnedPtr<Interface>) {
        self.add_entry(interface_ptr);
    }

    unsafe fn visit_enum_start(&mut self, enum_ptr: &mut OwnedPtr<Enum>) {
        self.add_entry(enum_ptr);
    }

    unsafe fn visit_trait(&mut self, trait_ptr: &mut OwnedPtr<Trait>) {
        self.add_entry(trait_ptr);
    }

    unsafe fn visit_custom_type(&mut self, custom_type_ptr: &mut OwnedPtr<CustomType>) {
        self.add_entry(custom_type_ptr);
    }

    unsafe fn visit_type_alias(&mut self, type_alias_ptr: &mut OwnedPtr<TypeAlias>) {
        self.add_entry(type_alias_ptr);
    }

    unsafe fn visit_operation_start(&mut self, operation_ptr: &mut OwnedPtr<Operation>) {
        self.add_entry(operation_ptr);
    }

    unsafe fn visit_data_member(&mut self, data_member_ptr: &mut OwnedPtr<DataMember>) {
        self.add_entry(data_member_ptr);
    }

    unsafe fn visit_parameter(&mut self, parameter_ptr: &mut OwnedPtr<Parameter>) {
        self.add_entry(parameter_ptr);
    }

    unsafe fn visit_return_member(&mut self, parameter_ptr: &mut OwnedPtr<Parameter>) {
        self.add_entry(parameter_ptr);
    }

    unsafe fn visit_enumerator(&mut self, enumerator_ptr: &mut OwnedPtr<Enumerator>) {
        self.add_entry(enumerator_ptr);
    }
}
