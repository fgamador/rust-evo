use smallvec::alloc::slice::{Iter, IterMut};
use std::cmp::Ordering;
use std::convert::TryInto;
use std::fmt;
use std::fmt::{Error, Formatter};
use std::marker::PhantomData;
use std::u32;

#[derive(Debug)]
pub struct ObjectsWithHandles<T: ObjectWithHandle<T>> {
    objects: Vec<T>,
}

impl<T: ObjectWithHandle<T>> ObjectsWithHandles<T> {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        ObjectsWithHandles { objects: vec![] }
    }

    pub fn add(&mut self, mut obj: T) -> Handle<T> {
        let handle = self.next_handle();
        *obj.handle_mut() = handle;
        self.objects.push(obj);
        handle
    }

    fn next_handle(&self) -> Handle<T> {
        Handle::new(self.objects.len().try_into().unwrap())
    }

    pub fn is_valid_handle(&self, handle: Handle<T>) -> bool {
        (handle.index as usize) < self.objects.len()
    }

    /// Removes the objects referenced by `handles`.
    ///
    /// Warning: this function has two big gotchas:
    ///
    /// 1) `handles` should be in ascending order of `index`. If not, the function will
    /// panic on index out-of-bounds if we're removing objects at the end of self.objects.
    ///
    /// 2) Worse, this function changes the objects referenced by some of the remaining handles.
    /// Never retain handles across a call to this function.
    pub fn remove_all<F>(&mut self, handles: &[Handle<T>], mut on_handle_change: F)
    where
        F: FnMut(&T, Handle<T>),
    {
        for &handle in handles.iter().rev() {
            self.remove(handle, &mut on_handle_change);
        }
    }

    /// Warning: invalidates handles to the last object in self.objects.
    fn remove<F>(&mut self, handle: Handle<T>, on_handle_change: &mut F)
    where
        F: FnMut(&T, Handle<T>),
    {
        self.objects.swap_remove(handle.index());
        if self.is_valid_handle(handle) {
            *self.object_mut(handle).handle_mut() = handle;
            on_handle_change(self.object(handle), self.next_handle());
        }
    }

    pub fn with_objects<F>(&mut self, handle1: Handle<T>, handle2: Handle<T>, mut f: F)
    where
        F: FnMut(&mut T, &mut T),
    {
        let obj1;
        let obj2;
        if handle1.index() < handle2.index() {
            let slices = self.objects.split_at_mut(handle2.index());
            obj1 = &mut slices.0[handle1.index()];
            obj2 = &mut slices.1[0];
        } else {
            let slices = self.objects.split_at_mut(handle1.index());
            obj2 = &mut slices.0[handle2.index()];
            obj1 = &mut slices.1[0];
        }

        f(obj1, obj2);
    }

    pub fn objects(&self) -> &[T] {
        &self.objects
    }

    pub fn objects_mut(&mut self) -> &mut [T] {
        &mut self.objects
    }

    pub fn object(&self, handle: Handle<T>) -> &T {
        &self.objects[handle.index()]
    }

    pub fn object_mut(&mut self, handle: Handle<T>) -> &mut T {
        &mut self.objects[handle.index()]
    }

    pub fn iter(&self) -> Iter<'_, T> {
        self.objects.iter()
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        self.objects.iter_mut()
    }
}

impl<T: ObjectWithHandle<T>> IntoIterator for ObjectsWithHandles<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.objects.into_iter()
    }
}

pub trait ObjectWithHandle<T: ObjectWithHandle<T>> {
    fn handle(&self) -> Handle<T>;

    fn handle_mut(&mut self) -> &mut Handle<T>;
}

pub struct Handle<T: ObjectWithHandle<T>> {
    index: u32,
    _phantom: PhantomData<T>,
}

impl<T: ObjectWithHandle<T>> Handle<T> {
    pub fn new(index: u32) -> Self {
        Handle {
            index,
            _phantom: PhantomData,
        }
    }

    pub fn unset() -> Self {
        Handle {
            index: u32::MAX,
            _phantom: PhantomData,
        }
    }

    fn index(self) -> usize {
        self.index as usize
    }
}

impl<T: ObjectWithHandle<T>> Clone for Handle<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: ObjectWithHandle<T>> Copy for Handle<T> {}

impl<T: ObjectWithHandle<T>> fmt::Debug for Handle<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Handle")
            .field("index", &self.index)
            .finish()
    }
}

impl<T: ObjectWithHandle<T>> fmt::Display for Handle<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", self.index)
    }
}

impl<T: ObjectWithHandle<T>> Eq for Handle<T> {}

impl<T: ObjectWithHandle<T>> Ord for Handle<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.index.cmp(&other.index)
    }
}

impl<T: ObjectWithHandle<T>> PartialEq for Handle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}

impl<T: ObjectWithHandle<T>> PartialOrd for Handle<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn added_object_has_correct_handle() {
        let mut objs = ObjectsWithHandles::new();

        let handle = objs.add(SimpleObjectWithHandle::new(0));
        println!("{:?}", handle);

        let obj = &objs.objects()[0];
        assert_eq!(obj.handle(), handle);
    }

    #[test]
    fn can_fetch_object_by_handle() {
        let mut objs = ObjectsWithHandles::new();

        let handle = objs.add(SimpleObjectWithHandle::new(0));

        let obj = &objs.objects()[0];
        assert_eq!(*objs.object(handle), *obj);
    }

    #[test]
    fn can_remove_last_and_non_last_objects() {
        let mut objs = ObjectsWithHandles::new();
        let handle0 = objs.add(SimpleObjectWithHandle::new(0));
        let _handle1 = objs.add(SimpleObjectWithHandle::new(1));
        let handle2 = objs.add(SimpleObjectWithHandle::new(2));

        objs.remove_all(&vec![handle0, handle2], |_, _| {});

        assert_eq!(objs.objects.len(), 1);
        let obj = &objs.objects()[0];
        assert_eq!(obj.id, 1);
        assert_eq!(obj.handle().index, 0);
    }

    #[test]
    fn gets_callback_for_swapped_object() {
        let mut objs = ObjectsWithHandles::new();
        let handle0 = objs.add(SimpleObjectWithHandle::new(0));
        objs.add(SimpleObjectWithHandle::new(1));
        let mut num = 0;

        objs.remove_all(&vec![handle0], |obj, prev_handle| {
            assert_eq!(obj.handle.index, 0);
            assert_eq!(prev_handle.index, 1);
            num = 42;
        });

        assert_eq!(num, 42);
    }

    #[derive(Clone, Debug, PartialEq)]
    pub struct SimpleObjectWithHandle {
        handle: Handle<SimpleObjectWithHandle>,
        pub id: i32,
    }

    impl SimpleObjectWithHandle {
        pub fn new(id: i32) -> Self {
            SimpleObjectWithHandle {
                handle: Handle::unset(),
                id,
            }
        }
    }

    impl ObjectWithHandle<SimpleObjectWithHandle> for SimpleObjectWithHandle {
        fn handle(&self) -> Handle<SimpleObjectWithHandle> {
            self.handle
        }

        fn handle_mut(&mut self) -> &mut Handle<SimpleObjectWithHandle> {
            &mut self.handle
        }
    }
}
