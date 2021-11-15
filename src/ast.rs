// Copyright (c) ZeroC, Inc. All rights reserved.

// TODO this file needs some cleanup now.

use crate::{downgrade_as, upcast_owned_as, upcast_weak_as};

use crate::grammar::*;
use crate::ptr_visitor::PtrVisitor;
use crate::ptr_util::{OwnedPtr, WeakPtr};
use std::collections::HashMap;

/// The AST (Abstract Syntax Tree) is the heart of the compiler, containing all the
/// slice elements defined by, and used by, the slice files passed into the compiler.
///
/// There is a single instance of the AST per-compile, which is created during initialization.
/// The parser parses each file into its top level modules, and then moves
/// these modules directly into the shared AST.
///
/// In addition, the AST owns all anonymous types, which are created in the AST as the parser needs them,
/// and it owns all the Slice [Primitive]s, which are added to the AST during initialization,
/// ensuring they're always available.
///
/// The AST is primarly for centralizing ownership of Slice elements, but also features lookup
/// methods for retrieving [primitives](Ast::lookup_primitive), [types](Ast::lookup_type), and
/// [entities](Ast::lookup_entity) by name.
///
/// The AST effectively has a `'static` lifetime. It is created when the compiler starts execution
/// (in [global_state::initialize]), and it stays in scope until the program terminates.
#[derive(Debug)]
pub struct Ast {
    /// The AST vector owns all the top level modules defined in every slice file parsed by the compiler.
    /// These modules in-turn own all their contents. Hence, this vector contains all **user** defined slice elements.
    /// All compiler defined types are stored in either [anonymous_types] or [primitive_cache].
    pub(crate) ast: Vec<OwnedPtr<Module>>,

    /// This vector owns all the non-primitive anonymous types ([Sequence]s and [Dictionary]s).
    /// These types aren't defined by users, but are still usable in slice definitions behind a [TypeRef].
    ///
    /// Since these types aren't defined by users, they can't be owned by a module.
    /// Instead they are owned by the AST, and stored here, so that they can be referenced via [WeakPtr] elsewhere.
    /// Types are stored in the order they're parsed, and are only created when needed by the parser.
    pub(crate) anonymous_types: Vec<OwnedPtr<dyn Type>>,

    /// This cache holds the definitions for all the Slice primitive types, keyed by their Slice keywords.
    /// Primitives are built-in to the compiler, and are always defined, even if they're not needed.
    /// They are kept in a separate cache for stronger typing (this stores `Primitive`s instead of just `dyn Type`s),
    /// and to prevent excessive copies of primitives being created. A single instance per-primitive is safe,
    /// as primitives are not scope-sensative, unlike other anonymous types.
    pub(crate) primitive_cache: HashMap<&'static str, OwnedPtr<Primitive>>,

    /// This lookup table stores [WeakPtr]s for every user defined entity that is defined in global
    /// or module scope. Each [Entity]'s **module** scoped identifier is used as its key in the table.
    pub(crate) module_scoped_lookup_table: HashMap<String, WeakPtr<dyn Entity>>,

    /// This lookup table stores [WeakPtr]s for every user defined entity that is stored in the AST.
    /// Each [Entity]'s **parser** scoped identifier is used as its key in the table.
    pub(crate) parser_scoped_lookup_table: HashMap<String, WeakPtr<dyn Entity>>,
}

impl Ast {
    pub(crate) fn new() -> Ast {
        // Primitive types are built in to the compiler. Since they aren't defined in Slice,
        // we 'define' them here when the AST is created, to ensure they're always available.
        let primitive_cache = HashMap::from([
            ("bool", OwnedPtr::new(Primitive::Bool)),
            ("byte", OwnedPtr::new(Primitive::Byte)),
            ("short", OwnedPtr::new(Primitive::Short)),
            ("ushort", OwnedPtr::new(Primitive::UShort)),
            ("int", OwnedPtr::new(Primitive::Int)),
            ("uint", OwnedPtr::new(Primitive::UInt)),
            ("varint", OwnedPtr::new(Primitive::VarInt)),
            ("varuint", OwnedPtr::new(Primitive::VarUInt)),
            ("long", OwnedPtr::new(Primitive::Long)),
            ("ulong", OwnedPtr::new(Primitive::ULong)),
            ("varlong", OwnedPtr::new(Primitive::VarLong)),
            ("varulong", OwnedPtr::new(Primitive::VarULong)),
            ("float", OwnedPtr::new(Primitive::Float)),
            ("double", OwnedPtr::new(Primitive::Double)),
            ("string", OwnedPtr::new(Primitive::String)),
            ("AnyClass", OwnedPtr::new(Primitive::AnyClass)),
        ]);

        // Create an empty AST (apart from the primitive cache).
        Ast {
            ast: Vec::new(),
            anonymous_types: Vec::new(),
            primitive_cache,
            module_scoped_lookup_table: HashMap::new(),
            parser_scoped_lookup_table: HashMap::new(),
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
            module_scoped_lookup_table: &mut self.module_scoped_lookup_table,
            parser_scoped_lookup_table: &mut self.parser_scoped_lookup_table,
        };

        // Add the module into the lookup tables, then recursively add it's contents too.
        //
        // This is always safe; no other references to the module can exist because we own it,
        // and haven't dereferenced any of the pointers to it that we've constructed.
        unsafe { module_ptr.visit_ptr_with(&mut visitor); }

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
    /// ```
    /// operation(sequence<int> i1, sequence<int> i2); // 2 anonymous types are created here, 1 for each sequence.
    /// ```
    /// Additionally, no indexing is performed on these types since they're un-named and unique to where they're used.
    /// Instead, this function returns a reference to the [OwnedPtr] storing the type,
    /// since there's no way to access the type through a lookup table later.
    ///
    /// This should only be called by the parser.
    pub(crate) fn add_anonymous_type(&mut self, ty: impl Type + 'static) -> &OwnedPtr<dyn Type> {
        let type_ptr = upcast_owned_as!(OwnedPtr::new(ty), dyn Type);
        self.anonymous_types.push(type_ptr);
        self.anonymous_types.last().unwrap()
    }

    fn lookup_entity<'ast>(
        lookup_table: &'ast HashMap<String, WeakPtr<dyn Entity>>,
        identifier: &str,
        mut scopes: &[String],
    ) -> Option<&'ast WeakPtr<dyn Entity>> {
        // If the identifier starts with '::', it's a global identifier, which can be looked up directly.
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

    // =============================================================================================
    // These lookup functions are associated functions instead of methods so that the AST can be
    // mutated without locking down access to them.
    // Methods require borrowing the entire AST, which is impossible if some of its contents have
    // been mutably borrowed somewhere else (such as while visiting, or patching).
    // =============================================================================================

    pub fn lookup_module_scoped_entity<'ast>(
        module_scoped_lookup_table: &'ast HashMap<String, WeakPtr<dyn Entity>>,
        identifier: &str,
        scope: &Scope,
    ) -> Option<&'ast WeakPtr<dyn Entity>> {
        Ast::lookup_entity(module_scoped_lookup_table, identifier, &scope.module_scope)
    }

    pub fn lookup_parser_scoped_entity<'ast>(
        parser_scoped_lookup_table: &'ast HashMap<String, WeakPtr<dyn Entity>>,
        identifier: &str,
        scope: &Scope,
    ) -> Option<&'ast WeakPtr<dyn Entity>> {
        Ast::lookup_entity(parser_scoped_lookup_table, identifier, &scope.parser_scope)
    }

    pub fn lookup_type<'ast>(
        module_scoped_lookup_table: &'ast HashMap<String, WeakPtr<dyn Entity>>,
        primitive_cache: &'ast HashMap<&'static str, OwnedPtr<Primitive>>,
        identifier: &str,
        scope: &Scope
    ) -> Result<WeakPtr<dyn Type>, Option<WeakPtr<dyn Entity>>> {
        if let Some(primitive_ptr) = primitive_cache.get(identifier) {
            Ok(downgrade_as!(primitive_ptr, dyn Type))
        } else if let Some(entity_ptr) =
        Self::lookup_module_scoped_entity(module_scoped_lookup_table, identifier, scope) {
            match entity_ptr.borrow().concrete_entity() {
                Entities::Struct(_) => {
                    Ok(upcast_weak_as!(entity_ptr.clone().downcast::<Struct>().ok().unwrap(), dyn Type))
                }
                Entities::Class(_) => {
                    Ok(upcast_weak_as!(entity_ptr.clone().downcast::<Class>().ok().unwrap(), dyn Type))
                }
                Entities::Interface(_) => {
                    Ok(upcast_weak_as!(entity_ptr.clone().downcast::<Interface>().ok().unwrap(), dyn Type))
                }
                Entities::Enum(_) => {
                    Ok(upcast_weak_as!(entity_ptr.clone().downcast::<Enum>().ok().unwrap(), dyn Type))
                }
                Entities::TypeAlias(_) => {
                    Ok(upcast_weak_as!(entity_ptr.clone().downcast::<TypeAlias>().ok().unwrap(), dyn Type))
                }
                _ => Err(Some(entity_ptr.clone()))
            }
        } else {
            Err(None)
        }
    }

    pub fn lookup_primitive(&self, identifier: &str) -> &OwnedPtr<Primitive>{
        self.primitive_cache.get(identifier).unwrap_or_else(||
            panic!("No primitive type exists with the name '{}'", identifier)
        )
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
    /// Mutable reference to the AST's [Ast::module_scoped_lookup_table].
    ///
    /// Whenever the builder visits a slice [Entity] that is defined at global or module scope, it
    /// inserts a corresponding entry into this table. Each entry consists of the entity's
    /// **module** scoped identifier, and a [WeakPtr] to the entity.
    module_scoped_lookup_table: &'ast mut HashMap<String, WeakPtr<dyn Entity>>,

    /// Mutable reference to the AST's [Ast::parser_scoped_lookup_table].
    ///
    /// Whenever the builder visits a slice [Entity], it inserts a corresponding entry into this table.
    /// Each entry consists of the entity's **parser** scoped identifier, and a [WeakPtr] to the entity.
    parser_scoped_lookup_table: &'ast mut HashMap<String, WeakPtr<dyn Entity>>,
}

impl<'ast> LookupTableBuilder<'ast> {
    /// Adds an entry into the AST's [module_scoped_lookup_table] for the provided definition.
    fn add_module_scoped_entry<T: Entity + 'static>(&mut self, definition: &OwnedPtr<T>) {
        let identifier = definition.borrow().module_scoped_identifier();
        let weak_ptr = downgrade_as!(definition, dyn Entity);
        self.module_scoped_lookup_table.insert(identifier, weak_ptr);
    }

    /// Adds an entry into the AST's [parser_scoped_lookup_table] for the provided definition.
    fn add_parser_scoped_entry<T: Entity + 'static>(&mut self, definition: &OwnedPtr<T>) {
        let identifier = definition.borrow().parser_scoped_identifier();
        let weak_ptr = downgrade_as!(definition, dyn Entity);
        self.parser_scoped_lookup_table.insert(identifier, weak_ptr);
    }
}

impl<'ast> PtrVisitor for LookupTableBuilder<'ast> {
    unsafe fn visit_module_start(&mut self, module_ptr: &mut OwnedPtr<Module>) {
        self.add_module_scoped_entry(module_ptr);
        self.add_parser_scoped_entry(module_ptr);
    }

    unsafe fn visit_struct_start(&mut self, struct_ptr: &mut OwnedPtr<Struct>) {
        self.add_module_scoped_entry(struct_ptr);
        self.add_parser_scoped_entry(struct_ptr);
    }

    unsafe fn visit_class_start(&mut self, class_ptr: &mut OwnedPtr<Class>) {
        self.add_module_scoped_entry(class_ptr);
        self.add_parser_scoped_entry(class_ptr);
    }

    unsafe fn visit_exception_start(&mut self, exception_ptr: &mut OwnedPtr<Exception>) {
        self.add_module_scoped_entry(exception_ptr);
        self.add_parser_scoped_entry(exception_ptr);
    }

    unsafe fn visit_interface_start(&mut self, interface_ptr: &mut OwnedPtr<Interface>) {
        self.add_module_scoped_entry(interface_ptr);
        self.add_parser_scoped_entry(interface_ptr);
    }

    unsafe fn visit_enum_start(&mut self, enum_ptr: &mut OwnedPtr<Enum>) {
        self.add_module_scoped_entry(enum_ptr);
        self.add_parser_scoped_entry(enum_ptr);
    }

    unsafe fn visit_type_alias(&mut self, type_alias_ptr: &mut OwnedPtr<TypeAlias>) {
        self.add_module_scoped_entry(type_alias_ptr);
        self.add_parser_scoped_entry(type_alias_ptr);
    }

    unsafe fn visit_operation_start(&mut self, operation_ptr: &mut OwnedPtr<Operation>) {
        self.add_parser_scoped_entry(operation_ptr);
    }

    unsafe fn visit_data_member(&mut self, data_member_ptr: &mut OwnedPtr<DataMember>) {
        self.add_parser_scoped_entry(data_member_ptr);
    }

    unsafe fn visit_parameter(&mut self, parameter_ptr: &mut OwnedPtr<Parameter>) {
        self.add_parser_scoped_entry(parameter_ptr);
    }

    unsafe fn visit_return_member(&mut self, parameter_ptr: &mut OwnedPtr<Parameter>) {
        self.add_parser_scoped_entry(parameter_ptr);
    }

    unsafe fn visit_enumerator(&mut self, enumerator_ptr: &mut OwnedPtr<Enumerator>) {
        self.add_parser_scoped_entry(enumerator_ptr);
    }
}
