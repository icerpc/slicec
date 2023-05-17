// Copyright (c) ZeroC, Inc.

use std::any::TypeId;

/// Represents a pointer that owns the data it's pointing to.
/// When this pointer is dropped, it ensures the pointed to data is dropped as well.
///
/// This can be used in conjunction with [`WeakPtr`] to form complex (and sometimes cyclic) data structures while still
/// adhering to Rust's ownership rules, and avoiding un-droppable memory cycles.
#[derive(Debug)]
pub struct OwnedPtr<T: ?Sized> {
    data: Box<T>,
    concrete_type_id: TypeId,
}

impl<T: Sized + 'static> OwnedPtr<T> {
    pub fn new(value: T) -> Self {
        OwnedPtr {
            data: Box::new(value),
            concrete_type_id: TypeId::of::<T>(),
        }
    }
}

impl<T: ?Sized> OwnedPtr<T> {
    #[allow(clippy::should_implement_trait)]
    pub fn borrow(&self) -> &T {
        &self.data
    }

    /// # Safety
    ///
    /// This function doesn't use unsafe Rust, but is marked unsafe because the invoker must
    /// GUARANTEE there are no other references to the underlying data when calling this function.
    ///
    /// The borrow checker can ensure that there are no other references through this `OwnedPtr`,
    /// but it's possible to obtain a reference via `WeakPtr::borrow` instead. Because `WeakPtr`
    /// uses raw pointers, the borrow checker can't reason about these accesses. So we have to
    /// enforce Rust's borrow policy manually by ensuring this mutable borrow is the ONLY borrow.
    ///
    /// Mutating the underlying data while another reference to it still exists, is undefined
    /// behavior. So ONLY call this function if you are CERTAIN that NO other references exist.
    #[allow(clippy::should_implement_trait)]
    pub unsafe fn borrow_mut(&mut self) -> &mut T {
        &mut self.data
    }

    pub fn downgrade(&self) -> WeakPtr<T> {
        WeakPtr {
            data: Some(&*self.data),
            concrete_type_id: self.concrete_type_id,
        }
    }

    pub fn downcast<U: 'static>(self) -> Result<OwnedPtr<U>, OwnedPtr<T>> {
        // Make sure that the original concrete type of this `OwnedPtr` matches the requested type.
        // If it doesn't, return an error holding the uncasted `OwnedPtr`.
        if self.concrete_type_id == TypeId::of::<U>() {
            // Convert the underlying box into a raw pointer so we can forcibly cast it.
            let inner = Box::into_raw(self.data);
            // Cast the pointer to the original concrete type and re-box it.
            let converted = unsafe { Box::from_raw(inner as *mut U) };
            // Construct a new OwnedPtr with the downcasted type.
            Ok(OwnedPtr::from_inner((converted, self.concrete_type_id)))
        } else {
            Err(self)
        }
    }

    pub fn from_inner(inner: (Box<T>, TypeId)) -> Self {
        OwnedPtr {
            data: inner.0,
            concrete_type_id: inner.1,
        }
    }

    pub fn into_inner(self) -> (Box<T>, TypeId) {
        (self.data, self.concrete_type_id)
    }
}

/// Represents a pointer that only references the data it's pointing to.
/// Unlike [`OwnedPtr`], dropping this pointer has no effect on the underlying data, this pointer can only immutably
/// access the underlying data, and care must be taken to prevent this pointer from dangling.
///
/// This can be used in conjunction with [`OwnedPtr`] to form complex (and sometimes cyclic) data structures while
/// still adhering to Rust's ownership rules, and avoiding un-droppable memory cycles.
#[derive(Debug)]
pub struct WeakPtr<T: ?Sized> {
    data: Option<*const T>,
    concrete_type_id: TypeId,
}

impl<T: ?Sized + 'static> WeakPtr<T> {
    pub fn create_uninitialized() -> Self {
        WeakPtr {
            data: None,
            concrete_type_id: TypeId::of::<T>(),
        }
    }
}

impl<T: ?Sized> WeakPtr<T> {
    pub fn is_initialized(&self) -> bool {
        self.data.is_some()
    }

    // This isn't marked as unsafe because it's assumed all WeakPtr live inside the AST, alongside
    // the OwnedPtr. Since the entire AST goes out of scope at the same time when the program ends,
    // it's impossible to have a dangling pointer here, and so, this function is always safe.
    //
    // Note that it IS still possible to call this on an uninitialized WeakPtr, which will cause a
    // panic. But this isn't 'unsafe' in the technical sense of involving unsafe Rust.
    #[allow(clippy::should_implement_trait)]
    pub fn borrow(&self) -> &T {
        unsafe { &*self.data.unwrap() }
    }

    pub fn downcast<U: 'static>(self) -> Result<WeakPtr<U>, WeakPtr<T>> {
        // Make sure that the original concrete type of this `WeakPtr` matches the requested type.
        // If it doesn't, return an error holding the uncasted `WeakPtr`.
        if self.concrete_type_id == TypeId::of::<U>() {
            // Forcibly downcast the underlying pointer to the original concrete type.
            let converted = self.data.map(|ptr| ptr as *const U);
            // Construct a new WeakPtr with the downcasted type.
            Ok(WeakPtr::from_inner((converted, self.concrete_type_id)))
        } else {
            Err(self)
        }
    }

    pub fn from_inner(inner: (Option<*const T>, TypeId)) -> Self {
        WeakPtr {
            data: inner.0,
            concrete_type_id: inner.1,
        }
    }

    pub fn into_inner(self) -> (Option<*const T>, TypeId) {
        (self.data, self.concrete_type_id)
    }
}

impl<T: ?Sized> Clone for WeakPtr<T> {
    fn clone(&self) -> Self {
        WeakPtr {
            data: self.data,
            concrete_type_id: self.concrete_type_id,
        }
    }
}

// TODO
// Implementing these traits would give our pointers support for implicit upcasting (casting a
// concrete type to a trait type it implements). But the trait is still marked as unstable.
// When it's stabilized, this should be uncommented, and the macros beneath this deleted.
//
// impl<T: ?Sized + Unsize<U>, U: ?Sized> CoerceUnsized<OwnedPtr<U>> for OwnedPtr<T> {}
// impl<T: ?Sized + Unsize<U>, U: ?Sized> CoerceUnsized<WeakPtr<U>> for WeakPtr<T> {}

#[macro_export]
macro_rules! downgrade_as {
    ($owned:expr, $new_type:ty) => {
        $crate::upcast_weak_as!($owned.downgrade(), $new_type)
    };
}

#[macro_export]
macro_rules! upcast_owned_as {
    ($owned:expr, $new_type:ty) => {{
        let (data, type_id) = $owned.into_inner();
        OwnedPtr::from_inner((data as Box<$new_type>, type_id))
    }};
}

#[macro_export]
macro_rules! upcast_weak_as {
    ($weak:expr, $new_type:ty) => {{
        let (data, type_id) = $weak.into_inner();
        WeakPtr::from_inner((data.map(|ptr| ptr as *const $new_type), type_id))
    }};
}

impl<'a, T: ?Sized, U: ?Sized> PartialEq<&'a T> for OwnedPtr<U> {
    /// Returns true if this pointer and the provided reference both point to the same memory address.
    ///
    /// Note that this may return true in some unintuitive/exotic cases:
    /// - If you have 2 references to the same piece of data, with different types, this will return true. For example,
    ///   comparing `String` to `dyn Display` is valid, and will return true if they're actually the same.
    /// - If one or both of the types are zero-sized: since it's address may overlap with another piece of data.
    /// - Comparing the address of a struct to the address of its first field: these are conceptually different things,
    ///   but both live at the same address, since structs are stored as a list of it's fields.
    ///
    /// See <https://doc.rust-lang.org/std/ptr/fn.eq.html> for more information. This function uses the same semantics.
    fn eq(&self, other: &&'a T) -> bool {
        // Convert this pointer's box and the other borrow to raw pointers, then strip their typing, and convert any
        // DST fat pointers to thin pointers to avoid checking their v-tables (which can be transient).
        let self_ptr = (&*self.data as *const U).cast::<()>();
        let other_ptr = (*other as *const T).cast::<()>();
        // Check if the data pointers point to the same location in memory.
        std::ptr::eq(self_ptr, other_ptr)
    }
}

impl<'a, T: ?Sized, U: ?Sized> PartialEq<&'a T> for WeakPtr<U> {
    /// Returns true if this pointer and the provided reference both point to the same memory address.
    ///
    /// Note that this may return true in some unintuitive/exotic cases:
    /// - If you have 2 references to the same piece of data, with different types, this will return true. For example,
    ///   comparing `String` to `dyn Display` is valid, and will return true if they're actually the same.
    /// - If one or both of the types are zero-sized: since it's address may overlap with another piece of data.
    /// - Comparing the address of a struct to the address of its first field: these are conceptually different things,
    ///   but both live at the same address, since structs are stored as a list of it's fields.
    ///
    /// See <https://doc.rust-lang.org/std/ptr/fn.eq.html> for more information. This function uses the same semantics.
    fn eq(&self, other: &&'a T) -> bool {
        // Convert the other borrow to a raw pointer, then strip it and this pointer's typing, and convert any
        // DST fat pointers to thin pointers to avoid checking their v-tables (which can be transient).
        let Some(self_ptr) = self.data else { return false; };
        let other_ptr = (*other as *const T).cast::<()>();
        // Check if the data pointers point to the same location in memory.
        std::ptr::eq(self_ptr.cast::<()>(), other_ptr)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Ensure that it's valid to have multiple immutable borrows to a piece a data through
    // the pointer that owns it, and any weak pointers created from it.
    #[test]
    fn multiple_immutable_borrows_is_legal() {
        // Arrange
        let owned_ptr = OwnedPtr::new(79_u32);
        let weak_ptr1 = owned_ptr.downgrade();
        let weak_ptr2 = owned_ptr.downgrade();

        let borrow0 = owned_ptr.borrow();
        let borrow1 = weak_ptr1.borrow();
        let borrow2 = weak_ptr2.borrow();

        // Act
        // Use all the borrowed values to prevent the compiler from dropping them pre-maturely.
        let dummy = borrow0 + borrow1 + borrow2;

        // Assert
        assert_eq!(dummy, 3 * 79);
    }

    // Ensure that accessing an uninitialized pointer causes a panic.
    // It's impossible to construct an uninitialized `OwnedPtr`, so we only check `WeakPtr`.
    #[test]
    #[should_panic]
    fn accessing_uninitialized_pointer_causes_panic() {
        // Arrange
        let weak_ptr: WeakPtr<String> = WeakPtr::create_uninitialized();

        // Act
        weak_ptr.borrow();

        // Assert
        // This function should panic in the 'act' section. This is 'asserted' by the 'should_panic' attribute on it.
    }

    #[test]
    fn pointer_equality_is_reflexive() {
        // Arrange
        let owned_ptr = OwnedPtr::new("test".to_owned());
        let weak_ptr = owned_ptr.downgrade();

        // Act/Assert: asserting that they're equal is the 'act'.
        assert_eq!(owned_ptr, owned_ptr.borrow());
        assert_eq!(owned_ptr, weak_ptr.borrow());
        assert_eq!(weak_ptr, owned_ptr.borrow());
        assert_eq!(weak_ptr, weak_ptr.borrow());
    }

    // Ensure that two pointers that point to the same memory location are equal, even if they hold different types.
    #[test]
    fn pointer_equality_is_type_independent() {
        // Arrange
        let owned_ptr: OwnedPtr<String> = OwnedPtr::new("test".to_owned());

        // Create a weak pointer to the string.
        let weak_ptr: WeakPtr<String> = owned_ptr.downgrade();
        // Rip it apart and manually cast the pointer from `String` to `bool`.
        let (raw_pointer, type_id) = weak_ptr.into_inner();
        let casted_pointer = raw_pointer.map(|ptr| ptr as *const bool);
        // Re-assemble the weak pointer with the casted type.
        // This is safe and legal to do in Rust, but borrowing from this pointer in any way would be unsafe.
        let casted_weak_ptr: WeakPtr<bool> = WeakPtr::from_inner((casted_pointer, type_id));

        // Act/Assert: asserting that they're equal is the 'act'.
        assert_eq!(casted_weak_ptr, owned_ptr.borrow());
    }

    // Ensure that two pointers that point to different memory locations are unequal, even if they point to equal data.
    #[test]
    fn different_pointers_are_not_equal() {
        // Arrange
        let owned_ptr1 = OwnedPtr::new(79_i32);
        let weak_ptr1 = owned_ptr1.downgrade();

        let owned_ptr2 = OwnedPtr::new(79_i32);
        let weak_ptr2 = owned_ptr2.downgrade();

        // Act/Assert: asserting that they're not equal is the 'act'.
        assert_ne!(owned_ptr1, owned_ptr2.borrow());
        assert_ne!(owned_ptr1, weak_ptr2.borrow());

        assert_ne!(weak_ptr1, owned_ptr2.borrow());
        assert_ne!(weak_ptr1, weak_ptr2.borrow());

        assert_ne!(owned_ptr2, owned_ptr1.borrow());
        assert_ne!(owned_ptr2, weak_ptr1.borrow());

        assert_ne!(weak_ptr2, owned_ptr1.borrow());
        assert_ne!(weak_ptr2, weak_ptr1.borrow());
    }
}
