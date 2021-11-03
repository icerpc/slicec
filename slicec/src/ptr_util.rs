// Copyright (c) ZeroC, Inc. All rights reserved.

use std::any::TypeId;

// `ThreadSafe` is a transparent wrapper for marking data as thread-safe, even if it isn't.
// The Rust compiler automatically infers thread-safety at compile time, but data can be
// explicitily marked as thread-safe by implementing the `Sync` trait on it, like here.
//
// We use this as a hack to satisfy the Rust compiler. Only thread-safe data can be stored in
// static variables, since it MIGHT be accessed from other threads. But since the slice
// compiler is single threaded, this isn't a concern.
//
// If we ever make the slice compiler multi-threaded we'd have to make the data thread-safe
// anyways, and then could drop this hack.
pub struct ThreadSafe<T>(pub T);

unsafe impl<T> Sync for ThreadSafe<T> {}

impl<T> std::ops::Deref for ThreadSafe<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug)]
pub struct OwnedPtr<T: ?Sized> {
    data: Box<T>,
    concrete_type_id: TypeId, // TODO for downcasting support
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
    pub fn borrow(&self) -> &T {
        &*self.data
    }

    // This function doesn't use unsafe Rust, but is marked unsafe because the invoker must
    // GUARANTEE there are no other references to the underlying data when calling this function.
    //
    // The borrow checker can ensure that there are no other references through this `OwnedPtr`,
    // but it's possible to obtain a reference via `WeakPtr::borrow` instead. Because `WeakPtr`
    // uses raw pointers, the borrow checker can't reason about these accesses. So we have to
    // enforce Rust's borrow policy manually by ensuring this mutable borrow is the ONLY borrow.
    //
    // Mutating the underlying data while another reference to it still exists, is undefined
    // behavior. So ONLY call this function if you are CERTAIN that NO other references exist.
    pub unsafe fn borrow_mut(&mut self) -> &mut T {
        &mut *self.data
    }

    pub fn downgrade(&self) -> WeakPtr<T> {
        WeakPtr {
            data: Some(&*self.data),
            concrete_type_id: self.concrete_type_id,
        }
    }

    pub fn downcast<U: 'static>(self) -> Result<OwnedPtr<U>, OwnedPtr<T>> {
        // Make sure that the original concrete type of this `OwnedPtr` matches the requested type.
        // If it doesn't return an error holding the uncasted `OwnedPtr`.
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

#[derive(Debug)]
pub struct WeakPtr<T: ?Sized> {
    data: Option<*const T>,
    concrete_type_id: TypeId, // TODO for downcasting support
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
    pub fn borrow(&self) -> &T {
        unsafe { &*self.data.unwrap() }
    }

    pub fn downcast<U: 'static>(self) -> Result<WeakPtr<U>, WeakPtr<T>> {
        // Make sure that the original concrete type of this `WeakPtr` matches the requested type.
        // If it doesn't return an error holding the uncasted `WeakPtr`.
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
//impl<T: ?Sized + Unsize<U>, U: ?Sized> CoerceUnsized<OwnedPtr<U>> for OwnedPtr<T> {}
//impl<T: ?Sized + Unsize<U>, U: ?Sized> CoerceUnsized<WeakPtr<U>> for WeakPtr<T> {}

#[macro_export]
macro_rules! downgrade_as {
    ($owned:expr, $new_type:ty) => {
        crate::upcast_weak_as!($owned.downgrade(), $new_type)
    };
}

#[macro_export]
macro_rules! upcast_owned_as {
    ($owned:expr, $new_type:ty) => {{
        let (data, type_id) = $owned.into_inner();
        OwnedPtr::from_inner((
            data as Box<$new_type>,
            type_id,
        ))
    }};
}

#[macro_export]
macro_rules! upcast_weak_as {
    ($weak:expr, $new_type:ty) => {{
        let (data, type_id) = $weak.into_inner();
        WeakPtr::from_inner((
            data.map(|ptr| ptr as *const $new_type),
            type_id,
        ))
    }};
}
