// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::downgrade_as;
use crate::upcast_owned_as;

use crate::grammar::*;
use crate::ptr_visitor::PtrVisitor;
use crate::ptr_util::{OwnedPtr, WeakPtr};
use std::collections::HashMap;

/// The AST (Abstract Syntax Tree) is the heart of the compiler, containing all the
/// slice elements defined by, and used by, the slice files passed into the compiler.
///
/// There is a single instance of the AST per-compile, which is created during initialization.
/// The parser parses each file into it's top level modules, and then moves
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

    /// This lookup table stores [WeakPtr]s for every **user defined** [Type] stored in the AST.
    /// **Module** scoped identifiers are used as keys for each [Type]'s pointer.
    ///
    /// [Primitive] types must be looked up from the [primitive_cache],
    /// and it's impossible to lookup anonymous types.
    pub(crate) type_lookup_table: HashMap<String, WeakPtr<dyn Type>>,

    /// This lookup table stores [WeakPtr]s for every [Entity] stored in the AST.
    /// **Parser** scoped identifiers are used as keys for each [Entity]'s pointer.
    pub(crate) entity_lookup_table: HashMap<String, WeakPtr<dyn Entity>>,
}

impl Ast {
    pub(crate) fn new() -> Ast {
        // Create an empty AST.
        let mut new_ast = Ast {
            ast: Vec::new(),
            anonymous_types: Vec::new(),
            primitive_cache: HashMap::new(),
            type_lookup_table: HashMap::new(),
            entity_lookup_table: HashMap::new(),
        };

        // Create an instance of each primitive and add them directly into the AST.
        // Primitive types are built in to the compiler. Since they aren't defined in Slice,
        // we 'define' them here when the AST is created, to ensure they're always available.
        new_ast.add_cached_primitive("bool", Primitive::Bool);
        new_ast.add_cached_primitive("byte", Primitive::Byte);
        new_ast.add_cached_primitive("short", Primitive::Short);
        new_ast.add_cached_primitive("ushort", Primitive::UShort);
        new_ast.add_cached_primitive("int", Primitive::Int);
        new_ast.add_cached_primitive("uint", Primitive::UInt);
        new_ast.add_cached_primitive("varint", Primitive::VarInt);
        new_ast.add_cached_primitive("varuint", Primitive::VarUInt);
        new_ast.add_cached_primitive("long", Primitive::Long);
        new_ast.add_cached_primitive("ulong", Primitive::ULong);
        new_ast.add_cached_primitive("varlong", Primitive::VarLong);
        new_ast.add_cached_primitive("varulong", Primitive::VarULong);
        new_ast.add_cached_primitive("float", Primitive::Float);
        new_ast.add_cached_primitive("double", Primitive::Double);
        new_ast.add_cached_primitive("string", Primitive::String);
        new_ast
    }

    /// Moves a [Module] into the AST, and returns a [WeakPtr] to it.
    /// It also visits through the module to index it and its contents into the AST's lookup tables.
    ///
    /// This should only be called by the parser.
    pub(crate) fn add_module(&mut self, module_def: Module) -> WeakPtr<Module> {
        // Move the module onto the heap so it can be referenced via pointer.
        let mut module_ptr = OwnedPtr::new(module_def);

        // Add the module into the AST's entity lookup table.
        let entity_ptr = downgrade_as!(module_ptr, dyn Entity);
        self.entity_lookup_table.insert(module_ptr.borrow().parser_scoped_identifier(), entity_ptr);

        // Recursively visit it's contents and add them into the lookup table too.
        let mut visitor = LookupTableBuilder {
            type_lookup_table: &mut self.type_lookup_table,
            entity_lookup_table: &mut self.entity_lookup_table,
        };
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

    /// Moves a [Primitive] into the AST.
    ///
    /// This is invoked by the AST during initialization, once per primitive.
    /// The provided `identifier` and `primitive` are placed in the AST's [primitive_cache] as a key-value pair.
    ///
    /// # Arguments
    ///
    /// * identifier - The slice keyword corresponding to the primitive type (Ex: `varulong`).
    /// * primitive - An instance of the primitive to add into the AST.
    fn add_cached_primitive(&mut self, identifier: &'static str, primitive: Primitive) {
        // Move the primitive onto the heap, so it can referenced via pointer.
        let primitive_ptr = OwnedPtr::new(primitive);

        // Insert an entry in the lookup table for the type, and cache the primitive's instance.
        let weak_ptr = downgrade_as!(primitive_ptr, dyn Type);
        self.type_lookup_table.insert(identifier.to_owned(), weak_ptr);
        self.primitive_cache.insert(identifier, primitive_ptr);
    }

    /// Looks up a [Primitive] type in the AST, by it's slice keyword (Ex: `varulong`).
    /// If the primitive exists, it returns a reference to the `OwnedPtr` holding it.
    ///
    /// # Arguments
    ///
    /// * primitive_cache - A borrow of the AST's [primitive_cache], to search through.
    /// * identifier - The slice keyword corresponding to the primitive type.
    ///
    /// # Panics
    ///
    /// If the provided `identifier` doesn't correspond to a Slice primitive type.
    /// This almost definitely indicates a mistake in the compiler's logic.
    ///
    /// # Examples
    /// ```
    /// let ast = Ast::new();
    /// let ulong = Ast::lookup_type(&ast.primitive_cache, "ulong");
    /// ```
    pub fn lookup_primitive(&self, identifier: &str) -> &OwnedPtr<Primitive> {
        self.primitive_cache.get(identifier)
            .expect(&format!("No Primitive type exists with the name '{}'", identifier))
    }

    // =============================================================================================
    // These lookup functions are associated functions instead of methods so that the AST can be
    // mutated without locking down access to them.
    // Methods require borrowing the entire AST, which is impossible if some of it's contents have
    // been mutably borrowed somewhere else (such as while visiting, or patching).
    // =============================================================================================

    /// Looks up a [Type] in the AST, by it's **module** scoped identifier.
    /// The behavior of this function depends on whether the scoped identifier is global or relative.
    ///
    /// Global identifiers begin with '::' and must be an exact match to return a type.
    ///
    /// Relative identifiers don't begin with '::'. This function tries to resolve relative identifiers
    /// with an inner -> outer approach. It first tries to find the type in the current scope, but if the type
    /// can't be found, it checks one scope higher, and so on, until global scope is reached.
    ///
    /// If the type can't be found, this returns `None`, otherwise it returns a borrowed [WeakPtr] to the type.
    ///
    /// # Arguments
    ///
    /// * type_lookup_table - A borrow of the AST's [type_lookup_table], to search through.
    /// * identifier - The scoped identifier of the type to lookup. Can be either globally or relatively scoped.
    /// * scope - The scope to begin the lookup in. This is only used to resolve relatively scoped types.
    ///
    /// # Examples
    ///
    /// ```
    /// let slice_file = "
    ///     module Foo
    ///     {
    ///         struct MyStruct {}
    ///         struct OnlyInFoo {}
    ///
    ///         module Bar
    ///         {
    ///             struct MyStruct {}
    ///         }
    ///     }
    /// ";
    ///
    /// let parser = parser::SliceParser::new();
    /// parser.parse_string(slice_file)?;
    /// let ast = parser.ast;
    ///
    /// let foo_scope = Scope::new("Foo");
    /// let bar_scope = Scope::new("Foo::Bar");
    ///
    /// ast.lookup_type("::Foo::MyStruct",      bar_scope); // Returns 'Foo::MyStruct'.
    /// ast.lookup_type("::Foo::Bar::MyStruct", bar_scope); // Returns 'Foo::Bar::MyStruct'.
    /// ast.lookup_type("::Bar::MyStruct",      bar_scope); // Returns 'None'.
    ///
    /// ast.lookup_type("MyStruct",  foo_scope); // Returns 'Foo::MyStruct'.
    /// ast.lookup_type("MyStruct",  bar_scope); // Returns 'Foo::Bar::MyStruct'.
    /// ast.lookup_type("OnlyInFoo", bar_scope); // Returns 'Foo::OnlyInFoo'.
    ///
    /// ast.lookup_type("Bar::MyStruct", foo_scope); // Returns 'Foo::Bar::MyStruct'.
    /// ast.lookup_type("Bar::MyStruct", bar_scope); // Returns 'Foo::Bar::MyStruct'.
    /// ast.lookup_type("Foo::MyStruct", bar_scope); // Returns 'Foo::MyStruct'.
    ///
    /// ast.lookup_type("Bar::FakeStruct", foo_scope); // Returns 'None'.
    /// ```
    pub fn lookup_type<'ast>(
        type_lookup_table: &'ast HashMap<String, WeakPtr<dyn Type>>,
        identifier: &str,
        scope: &Scope,
    ) -> Option<&'ast WeakPtr<dyn Type>> {
        // Paths starting with '::' are absolute paths, which can be directly looked up.
        if let Some(unprefixed) = identifier.strip_prefix("::") {
            return type_lookup_table.get(unprefixed);
        }

        // Types are looked up by module scope, since types can only be defined inside modules.
        let mut parents: &[String] = &scope.module_scope;

        // For relative paths, we check each enclosing scope, starting from the bottom
        // (most specified scope), and working our way up to global scope.
        while !parents.is_empty() {
            let candidate = parents.join("::") + "::" + identifier;
            if let Some(result) = type_lookup_table.get(&candidate) {
                return Some(result);
            }
            // Remove the last parent's scope before trying again.
            // It's safe to unwrap here, since we know that `parents` is not empty.
            parents = parents.split_last().unwrap().1;
        }

        // We couldn't find the type in any enclosing scope.
        None
    }

    /// Looks up an [Entity] in the AST, by it's **parser** scoped identifier.
    /// The behavior of this function depends on whether the scoped identifier is global or relative.
    ///
    /// Global identifiers begin with '::' and must be an exact match to return an entity.
    ///
    /// Relative identifiers don't begin with '::'. This function tries to resolve relative identifiers
    /// with an inner -> outer approach. It first tries to find the entity in the current scope, but if the entity
    /// can't be found, it checks one scope higher, and so on, until global scope is reached.
    ///
    /// If the entity can't be found, this returns `None`, otherwise it returns a borrowed [WeakPtr] to the entity.
    ///
    /// # Arguments
    ///
    /// * entity_lookup_table - A borrow of the AST's [entity_lookup_table], to search through.
    /// * identifier - The scoped identifier of the entity to lookup. Can be either globally or relatively scoped.
    /// * scope - The scope to begin the lookup in. This is only used to resolve relatively scoped entities.
    ///
    /// # Examples
    ///
    /// ```
    /// let slice_file = "
    ///     module Foo
    ///     {
    ///         module Cat {}
    ///
    ///         module Bar {
    ///             module Cat {}
    ///         }
    ///     }
    /// ";
    ///
    /// let parser = parser::SliceParser::new();
    /// parser.parse_string(slice_file);
    /// let ast = parser.ast;
    ///
    /// let foo_scope = Scope::new("Foo");
    /// let bar_scope = Scope::new("Foo::Bar");
    ///
    /// ast.lookup_entity("::Foo",      bar_scope); // Returns 'Foo'.
    /// ast.lookup_entity("::Foo::Bar", bar_scope); // Returns 'Foo::Bar'.
    /// ast.lookup_entity("::Bar",      bar_scope); // Returns 'None'.
    ///
    /// ast.lookup_entity("Cat", foo_scope);  // Returns 'Foo::Cat'.
    /// ast.lookup_entity("Cat", bar_scope);  // Returns 'Foo::Bar::Cat'.
    /// ast.lookup_entity("Foo", bar_scope);  // Returns 'Foo'.
    ///
    /// ast.lookup_entity("Bar::Cat", foo_scope); // Returns 'Foo::Bar::Cat'.
    /// ast.lookup_entity("Bar::Cat", bar_scope); // Returns 'Foo::Bar::Cat'.
    /// ast.lookup_entity("Foo::Cat", bar_scope); // Returns 'Foo::Cat'.
    ///
    /// ast.lookup_entity("Dog"); // Returns 'None'.
    /// ```
    pub fn lookup_entity<'ast>(
        entity_lookup_table: &'ast HashMap<String, WeakPtr<dyn Entity>>,
        identifier: &str,
        scope: &Scope,
    ) -> Option<&'ast WeakPtr<dyn Entity>> {
        // Paths starting with '::' are absolute paths, which can be directly looked up.
        if let Some(unprefixed) = identifier.strip_prefix("::") {
            return entity_lookup_table.get(unprefixed);
        }

        // Entites are looked up by parser scope, since entities can be defined anywhere, not
        // just inside modules. Ex: A parameter in an operation.
        let mut parents: &[String] = &scope.parser_scope;

        // For relative paths, we check each enclosing scope, starting from the bottom
        // (most specified scope), and working our way up to global scope.
        while !parents.is_empty() {
            let candidate = parents.join("::") + "::" + identifier;
            if let Some(result) = entity_lookup_table.get(&candidate) {
                return Some(result);
            }
            // Remove the last parent's scope before trying again.
            // It's safe to unwrap here, since we know that `parents` is not empty.
            parents = parents.split_last().unwrap().1;
        }

        // We couldn't find the entity in any enclosing scope.
        None
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
    /// Mutable reference to the AST's [Ast::type_lookup_table].
    ///
    /// Whenever the builder visits a slice element that implements [Type],
    /// it inserts a corresponding entry into this table.
    /// Each entry consists of the type's **module** scoped identifier and a [WeakPtr] to the element.
    ///
    /// Since this builder finds types by visiting through modules, it will only ever find user-defined
    /// types. Builtin types like [Primitive]s, [Sequence]s, and [Dictionary]s will not be indexed.
    type_lookup_table: &'ast mut HashMap<String, WeakPtr<dyn Type>>,

    /// Mutable reference to the AST's [Ast::entity_lookup_table].
    ///
    /// Whenever the builder visits a slice element that implements [Entity],
    /// it inserts a corresponding entry into this table.
    /// Each entry consists of the entity's **parser** scoped identifier and a [WeakPtr] to the element.
    entity_lookup_table: &'ast mut HashMap<String, WeakPtr<dyn Entity>>,
}

impl<'ast> LookupTableBuilder<'ast> {
    /// Adds an entry into the AST's [type_lookup_table] corresponding to the provided definition.
    /// The definition must implement both [Type] *and* [Entity], as this method is only for
    /// user-defined types (which all implement [Entity]).
    fn add_type_entry<T: Type + Entity + 'static>(&mut self, definition: &OwnedPtr<T>) {
        let identifier = definition.borrow().module_scoped_identifier();
        let weak_ptr = downgrade_as!(definition, dyn Type + 'static);
        self.type_lookup_table.insert(identifier, weak_ptr);
    }

    /// Adds an entry into the AST's [entity_lookup_table] corresponding to the provided definition.
    fn add_entity_entry<T: Entity + 'static>(&mut self, definition: &OwnedPtr<T>) {
        let identifier = definition.borrow().parser_scoped_identifier();
        let weak_ptr = downgrade_as!(definition, dyn Entity);
        self.entity_lookup_table.insert(identifier, weak_ptr);
    }
}

impl<'ast> PtrVisitor for LookupTableBuilder<'ast> {
    unsafe fn visit_module_start(&mut self, module_ptr: &mut OwnedPtr<Module>) {
        self.add_entity_entry(module_ptr);
    }

    unsafe fn visit_struct_start(&mut self, struct_ptr: &mut OwnedPtr<Struct>) {
        self.add_type_entry(struct_ptr);
        self.add_entity_entry(struct_ptr);
    }

    unsafe fn visit_class_start(&mut self, class_ptr: &mut OwnedPtr<Class>) {
        self.add_type_entry(class_ptr);
        self.add_entity_entry(class_ptr);
    }

    unsafe fn visit_exception_start(&mut self, exception_ptr: &mut OwnedPtr<Exception>) {
        self.add_entity_entry(exception_ptr);
    }

    unsafe fn visit_interface_start(&mut self, interface_ptr: &mut OwnedPtr<Interface>) {
        self.add_type_entry(interface_ptr);
        self.add_entity_entry(interface_ptr);
    }

    unsafe fn visit_enum_start(&mut self, enum_ptr: &mut OwnedPtr<Enum>) {
        self.add_type_entry(enum_ptr);
        self.add_entity_entry(enum_ptr);
    }

    unsafe fn visit_operation_start(&mut self, operation_ptr: &mut OwnedPtr<Operation>) {
        self.add_entity_entry(operation_ptr);
    }

    unsafe fn visit_type_alias(&mut self, type_alias_ptr: &mut OwnedPtr<TypeAlias>) {
        self.add_type_entry(type_alias_ptr);
        self.add_entity_entry(type_alias_ptr);
    }

    unsafe fn visit_data_member(&mut self, data_member_ptr: &mut OwnedPtr<DataMember>) {
        self.add_entity_entry(data_member_ptr);
    }

    unsafe fn visit_parameter(&mut self, parameter_ptr: &mut OwnedPtr<Parameter>) {
        self.add_entity_entry(parameter_ptr);
    }

    unsafe fn visit_return_member(&mut self, parameter_ptr: &mut OwnedPtr<Parameter>) {
        self.add_entity_entry(parameter_ptr);
    }

    unsafe fn visit_enumerator(&mut self, enumerator_ptr: &mut OwnedPtr<Enumerator>) {
        self.add_entity_entry(enumerator_ptr);
    }
}
