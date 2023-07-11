use std::{
    marker::{self, PhantomData},
    ptr::NonNull,
    result,
};

#[derive(Debug)]
pub enum DataStructureError {
    InvalidState,
    InvalidReference,
    InvalidIndex,
    InvalidActionEmpty,
}
pub type Result<T> = result::Result<T, DataStructureError>;
type Link<T> = Option<NonNull<DLLNode<T>>>;
#[allow(dead_code)]
struct DLLNode<T> {
    next: Link<T>,
    prev: Link<T>,
    value: T,
}

impl<T> DLLNode<T> {
    fn new(value: T) -> Self {
        Self {
            next: None,
            prev: None,
            value,
        }
    }

    unsafe fn enchain(a: NonNull<DLLNode<T>>, b: NonNull<DLLNode<T>>) {
        (*a.as_ptr()).next = Some(b);
        (*b.as_ptr()).prev = Some(a);
    }
}

pub struct DLL<T> {
    head: Link<T>,
    tail: Link<T>,
    length: usize,
    _marker: marker::PhantomData<T>,
}

impl<T> DLL<T> {
    pub fn new() -> Self {
        Self {
            head: None,
            tail: None,
            length: 0,
            _marker: PhantomData,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.length == 0
    }

    pub fn append(&mut self, value: T) {
        unsafe {
            let new_node = NonNull::new_unchecked(Box::into_raw(Box::new(DLLNode::new(value))));
            if self.length == 0 {
                self.head = Some(new_node);
                self.tail = Some(new_node);
            } else if let Some(old_tail) = self.tail {
                DLLNode::enchain(old_tail, new_node);
                self.tail = Some(new_node);
            }
        }
        self.length += 1;
    }

    pub fn prepend(&mut self, value: T) {
        unsafe {
            let new_node = NonNull::new_unchecked(Box::into_raw(Box::new(DLLNode::new(value))));

            if self.length == 0 {
                self.head = Some(new_node);
                self.tail = Some(new_node);
            } else {
                self.head.map(|head| {
                    DLLNode::enchain(new_node, head);
                    self.head = Some(new_node);
                });
            }
        }
        self.length += 1;
    }

    pub fn pop_back(&mut self) -> Option<T> {
        unsafe {
            self.tail.map(|tail_node| {
                let tail_node = Box::from_raw(tail_node.as_ptr());
                if let Some(prev) = tail_node.prev {
                    (*prev.as_ptr()).next = None;
                    self.tail = Some(prev);
                } else {
                    self.head = None;
                    self.tail = None;
                }

                self.length -= 1;
                tail_node.value
            })
        }
    }

    pub fn pop_front(&mut self) -> Option<T> {
        unsafe {
            self.head.map(|old_head| {
                let old_head = Box::from_raw(old_head.as_ptr());
                if let Some(next_node) = old_head.next {
                    (*next_node.as_ptr()).prev = None;
                    self.head = Some(next_node);
                } else {
                    self.head = None;
                    self.tail = None;
                }

                self.length -= 1;
                old_head.value
            })
        }
    }

    pub fn insert(&mut self, index: usize, value: T) {
        if index >= self.length {
            panic!(
                "invalid index '{}' for list with length '{}'",
                index, self.length
            );
        }
        unsafe {
            let mut counter: usize = 0;
            let new_node = NonNull::new_unchecked(Box::into_raw(Box::new(DLLNode::new(value))));
            let mut current_node = self.head.take().unwrap();
            self.head.replace(current_node);

            if counter == index {
                (*new_node.as_ptr()).prev = None;
                self.head = Some(new_node);
            } else {
                while counter != index {
                    let next_node = (*current_node.as_ptr()).next.take().unwrap();
                    (*current_node.as_ptr()).next.replace(next_node);
                    current_node = next_node;
                    counter += 1;
                }
                DLLNode::enchain((*current_node.as_ptr()).prev.take().unwrap(), new_node);
            }
            DLLNode::enchain(new_node, current_node);
            self.length += 1;
        }
    }

    pub fn into_iter(self) -> IntoIter<T> {
        IntoIter { list: self }
    }

    pub fn iter(&self) -> Iter<T> {
        Iter {
            front: self.head,
            back: self.tail,
            length: self.length,
            _marker: PhantomData,
        }
    }

    pub fn iter_mut(&mut self) -> IterMut<T> {
        IterMut {
            front: self.head,
            back: self.tail,
            length: self.length,
            _marker: PhantomData,
        }
    }
}

pub struct IntoIter<T> {
    list: DLL<T>,
}

impl<T> From<Vec<T>> for DLL<T> {
    fn from(collection: Vec<T>) -> Self {
        let mut list: DLL<T> = DLL::new();
        collection.into_iter().for_each(|item| list.append(item));
        list
    }
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.list.pop_front()
    }
}

pub struct Iter<'a, T> {
    front: Link<T>,
    back: Link<T>,
    length: usize,
    _marker: PhantomData<&'a T>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.length == 0 {
            return None;
        }
        self.front.map(|node| unsafe {
            self.length -= 1;
            self.front = (*node.as_ptr()).next;
            &(*node.as_ptr()).value
        })
    }
}
impl<'a, T> DoubleEndedIterator for Iter<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.length == 0 {
            return None;
        }
        self.back.map(|node| unsafe {
            self.length -= 1;
            self.back = (*node.as_ptr()).prev;
            &(*node.as_ptr()).value
        })
    }
}

impl<'a, T> ExactSizeIterator for Iter<'a, T> {
    fn len(&self) -> usize {
        self.length
    }
}

pub struct IterMut<'a, T> {
    front: Link<T>,
    back: Link<T>,
    length: usize,
    _marker: marker::PhantomData<&'a mut T>,
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.length == 0 {
            return None;
        }
        self.front.map(|node| unsafe {
            self.length -= 1;
            self.front = (*node.as_ptr()).next;
            &mut (*node.as_ptr()).value
        })
    }
}

impl<'a, T> DoubleEndedIterator for IterMut<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.length == 0 {
            return None;
        }
        self.back.map(|node| unsafe {
            self.length -= 1;
            self.back = (*node.as_ptr()).prev;
            &mut (*node.as_ptr()).value
        })
    }
}

impl<'a, T> ExactSizeIterator for IterMut<'a, T> {
    fn len(&self) -> usize {
        self.length
    }
}

impl<T> Drop for DLL<T> {
    fn drop(&mut self) {
        while let Some(_) = self.pop_back() {}
    }
}

impl<T> Into<Vec<T>> for DLL<T> {
    fn into(self) -> Vec<T> {
        Vec::from_iter(self.into_iter())
    }
}

impl<T: PartialEq> PartialEq for DLL<T> {
    fn eq(&self, other: &Self) -> bool {
        if self.length != other.length {
            return false;
        }
        let mut flag = true;
        let mut zipped = self.iter().zip(other.iter());

        while let Some((x, y)) = zipped.next() {
            if x != y {
                flag = false;
            }
        }

        flag
    }
}
impl<T> FromIterator<T> for DLL<T> {
    fn from_iter<A: IntoIterator<Item = T>>(iter: A) -> Self {
        let mut list = DLL::<T>::new();

        for element in iter {
            list.append(element);
        }

        list
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(test)]
    mod dll_node_tests {

        use super::*;

        #[test]
        fn new() {
            let value = 1;
            let node = DLLNode::<usize>::new(value);
            assert_eq!(node.value, 1);
            assert_eq!(node.next, None);
            assert_eq!(node.prev, None);
        }
    }

    #[cfg(test)]
    mod dll_tests {

        use std::ops::Range;

        use super::*;

        #[test]
        fn new() {
            let list: DLL<usize> = DLL::new();
            assert_eq!(list.length, 0);
            assert_eq!(list.head, None);
            assert_eq!(list.tail, None);
        }

        #[test]
        fn append() {
            let mut list: DLL<u8> = DLL::new();
            (0..3).enumerate().for_each(|(i, x)| {
                assert_eq!(list.length, i);
                list.append(x);
                assert_eq!(list.length, i + 1);
                let tail_value = list.tail.map(|tail| unsafe { (*tail.as_ptr()).value });
                assert_eq!(tail_value, Some(x));
            });
        }

        #[test]
        fn prepend() {
            let mut list = DLL::new();
            (0..5).for_each(|x| {
                assert_eq!(list.length, x);
                list.prepend(x);
                let head_value = list.head.map(|y| unsafe { (*y.as_ptr()).value });
                assert_eq!(head_value, Some(x));
            });
        }

        fn obtain_range() -> Range<u8> {
            0..5
        }

        fn obtain_list() -> DLL<u8> {
            let mut list = DLL::new();
            obtain_range().for_each(|x| list.append(x));
            return list;
        }

        #[test]
        fn pop_back() {
            let mut list = obtain_list();
            assert_eq!(list.length, 5);
            obtain_range().rev().for_each(|x| {
                assert_eq!(list.pop_back(), Some(x));
                assert_eq!(list.length as u8, x);
            });
        }

        #[test]
        fn pop_front() {
            let mut list = obtain_list();
            let range = obtain_range();
            let end = range.end;
            assert_eq!(list.length, 5);
            range.for_each(|x| {
                assert_eq!(list.pop_front(), Some(x));
                assert_eq!(list.length as u8, end - (x + 1));
            });
        }

        #[test]
        fn into_iter() {
            let list = obtain_list();
            let range = obtain_range();
            assert_eq!(list.into_iter().count(), range.count());
        }

        #[test]
        fn exact_sized_iterator() {
            let list = obtain_list();
            let range = obtain_range();
            let iter = list.iter();

            assert_eq!(iter.len(), range.len());
            assert_eq!(iter.len(), iter.zip(range).count());
        }

        #[test]
        fn double_ended_iter() {
            let list = obtain_list();
            let length = list.length;
            let range = obtain_range();
            let iter = list.iter();
            let mut zipped = iter.zip(range);

            let mut counter = 0;
            loop {
                counter += 2;
                if let Some((x, y)) = zipped.next() {
                    assert_eq!(x, &y);
                }
                if let Some((x, y)) = zipped.next_back() {
                    assert_eq!(x, &y);
                }

                if counter >= length {
                    break;
                }
            }
        }

        #[test]
        fn exact_sized_iterator_mut() {
            let mut list = obtain_list();
            let range = obtain_range();
            let iter = list.iter_mut();

            assert_eq!(iter.len(), range.len());
            assert_eq!(iter.len(), iter.zip(range).count());
        }

        #[test]
        fn double_ended_iter_mut() {
            let mut list = obtain_list();
            let length = list.length;
            let range = obtain_range();
            let iter = list.iter_mut();
            let mut zipped = iter.zip(range);

            let mut counter = 0;
            loop {
                counter += 2;
                if let Some((x, y)) = zipped.next() {
                    assert_eq!(x, &y);
                }
                if let Some((x, y)) = zipped.next_back() {
                    assert_eq!(x, &y);
                }

                if counter >= length {
                    break;
                }
            }
        }

        fn obtain_vector() -> Vec<u8> {
            let range = obtain_range();
            let mut vector = vec![];

            range.for_each(|x| vector.push(x));
            vector
        }

        #[test]
        fn equal() {
            assert!(obtain_list() == obtain_list());
        }

        #[test]
        fn not_equal() {
            let list_a = obtain_list();
            let mut list_b = obtain_list();
            list_b.append(100);
            assert!(list_a != list_b);
        }

        #[test]
        fn from_iterator() {
            let list = obtain_list();
            assert_eq!(Vec::from_iter(list.into_iter()), obtain_vector())
        }

        #[test]
        fn insert() {
            let value = 100;
            let index = 3;
            let mut list_a = obtain_list();
            let mut list_b = obtain_list();

            list_a.insert(index, value);
            list_b.insert(index, value);
            assert!(list_a == list_b);
            list_a.append(value);
            list_b.append(value);
            assert!(list_a == list_b);

            // check indices
            let mut vector_a: Vec<_> = list_a.into();
            assert_eq!(vector_a[index], value);
            assert_eq!(vector_a.last(), Some(&value));

            //check insert at front
            vector_a.insert(0, value);
            list_b.insert(0, value);
            assert_eq!(vector_a.len(), list_b.length);

            let vector_b: Vec<_> = list_b.into();
            assert_eq!(vector_a, vector_b);
        }
    }
}
