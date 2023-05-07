use std::{
    cell::RefCell,
    ops::Deref,
    rc::{Rc, Weak},
    result,
};

#[derive(Debug)]
pub enum DataStructureError {
    InvalidState,
    InvalidReference,
    InvalidIndex,
    NonZeroStrongCount(usize),
    InvalidActionEmpty,
}
pub type Result<T> = result::Result<T, DataStructureError>;

impl<T> From<Rc<T>> for DataStructureError {
    fn from(value: Rc<T>) -> Self {
        DataStructureError::NonZeroStrongCount(Rc::strong_count(&value))
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct Node<T> {
    value: T,
    next: Option<NextNode<T>>,
    prev: Option<PrevNode<T>>,
}

impl<T> Node<T> {
    fn new(value: T, next: Option<NextNode<T>>, prev: Option<PrevNode<T>>) -> Self {
        Self { value, next, prev }
    }
}

pub type NextNode<T> = Rc<RefCell<Node<T>>>;
pub type PrevNode<T> = Weak<RefCell<Node<T>>>;

impl<T> Deref for Node<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}





impl<T> Node<T> {
    fn set_next(&mut self, next: Option<NextNode<T>>) {
        self.next = next;
    }

    fn set_prev(&mut self, prev: Option<PrevNode<T>>) {
        self.prev = prev;
    }

    pub fn next_as_reference(&mut self) -> Option<NextNode<T>> {
        let next = self.next.take();
        match next {
            Some(node) => {
                let clone = Rc::clone(&node);
                self.set_next(Some(node));
                Some(clone)
            }
            None => None,
        }
    }

    pub fn prev_as_reference(&mut self) -> Option<PrevNode<T>> {
        let prev = self.prev.take();
        match prev {
            Some(node) => {
                let clone = Weak::clone(&node);
                self.set_prev(Some(node));
                Some(clone)
            }
            None => None,
        }
    }

    fn destruct(self) -> T {
        let Self { value, next, prev } = self;
        value
    }
}

/*
 * For DLL, implement
 * - IntoIterator
 * - Create types Iter and IterMut
 *  - types must implement iterator and double ended iterator
 *
 *
 *
 */

pub struct DoublyLinkedList<T> {
    head: Option<NextNode<T>>,
    tail: Option<PrevNode<T>>,
}

// todo: make Iter<T> and IterMut<T>

pub type DLL<T> = DoublyLinkedList<T>;

#[allow(dead_code)]
impl<T> DoublyLinkedList<T> {
    fn build() -> Self {
        Self {
            head: None,
            tail: None,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.head.is_none() && self.tail.is_none()
    }

    pub fn length(&mut self) -> usize {
        if self.head.is_none() {
            return 0;
        }

        let mut counter: usize = 1;
        let holder = self.head.take().expect("invalid head node");
        let mut current_node = Rc::clone(&holder);
        self.head.replace(holder);

        while let Some(node_ref) = current_node.borrow_mut().next_as_reference() {
            counter += 1;
            current_node = node_ref;
        }

        counter
    }

    pub fn head_as_reference(&mut self) -> Result<NextNode<T>> {
        let head = match self.head.take() {
            Some(node_ref) => node_ref,
            None => return Err(DataStructureError::InvalidActionEmpty),
        };
        let reference = Rc::clone(&head);
        self.head.replace(head);
        Ok(reference)
    }

    pub fn tail_as_reference(&mut self) -> Result<PrevNode<T>> {
        let tail = match self.tail.take() {
            Some(node_ref) => node_ref,
            None => return Err(DataStructureError::InvalidActionEmpty),
        };
        let reference = Weak::clone(&tail);
        self.tail.replace(tail);
        Ok(reference)
    }

    pub fn push(&mut self, item: T) -> Result<()> {
        let node: Node<T> = Node::new(item, None, None);
        let node_strong = Rc::new(RefCell::new(node));
        let node_weak = Rc::downgrade(&node_strong);
        self.tail = Some(node_weak);

        // start new head
        if self.head.is_none() {
            self.head = Some(node_strong);
            return Ok(());
        }

        // append to tail and reassign original tail
        let tail = match self.tail.take() {
            Some(node_ref) => node_ref,
            None => return Err(DataStructureError::InvalidReference),
        };
        let tail_strong =
            Weak::upgrade(&tail).ok_or_else(|| DataStructureError::InvalidReference)?;
        tail_strong
            .borrow_mut()
            .set_next(Some(Rc::clone(&node_strong)));
        node_strong.borrow_mut().set_prev(Some(Weak::clone(&tail)));
        Ok(())
    }

    pub fn pop(&mut self) -> Result<T> {
        let mut old_tail = match self.tail.take() {
            Some(node_ref) => match Weak::upgrade(&node_ref) {
                Some(value) => value,
                None => return Err(DataStructureError::InvalidReference),
            },
            None => return Err(DataStructureError::InvalidActionEmpty),
        };

        let old_tail_mut = old_tail.borrow_mut();
        match old_tail_mut.prev {
            Some(ref prev_node_weak) => {
                let mut prev_node_strong = Weak::upgrade(prev_node_weak).unwrap();
                prev_node_strong.borrow_mut().set_next(None);
                self.tail = Some(prev_node_weak)
            }
            None => self.head = None,
        };
        Ok(Rc::<RefCell<Node<T>>>::try_unwrap(old_tail)?
            .into_inner()
            .destruct())
    }
    pub fn unshift(&mut self, item: T) -> Result<()> {
        // start new tail
        if self.tail.is_none() {
            return self.push(item);
        }
        let node: Node<T> = Node::new(item, None, None);
        let node_strong = Rc::new(RefCell::new(node));
        let node_weak = Rc::downgrade(&node_strong);

        // append to head and reassign original head
        let head = self
            .head
            .take()
            .ok_or_else(|| DataStructureError::InvalidActionEmpty)?;
        head.borrow_mut().set_prev(Some(node_weak));
        node_strong.borrow_mut().set_next(Some(head));
        self.head = Some(node_strong);
        Ok(())
    }

    pub fn shift(&mut self) -> Result<T> {
        let mut old_head = self
            .head
            .take()
            .ok_or_else(|| DataStructureError::InvalidActionEmpty)?;

        match old_head.borrow_mut().next {
            Some(next_node) => {
                next_node.borrow_mut().set_prev(None);
                self.head = Some(next_node);
            }
            None => self.tail = None,
        };

        Ok(Rc::<RefCell<Node<T>>>::try_unwrap(old_head)?
            .into_inner()
            .destruct())
    }

    pub fn insert(&mut self, index: usize, item: T) -> Result<()> {
        let length = self.length();
        if index > length {
            return Err(DataStructureError::InvalidIndex);
        } else if index == 0 {
            return self.unshift(item);
        } else if index == length {
            return self.push(item);
        }

        let mut counter: usize = 0;
        let node_head = self
            .head
            .take()
            .ok_or_else(|| DataStructureError::InvalidReference)?;
        let mut current_node = Rc::clone(&node_head);
        self.head.replace(node_head);

        while counter != index {
            current_node = current_node.borrow_mut().next_as_reference().unwrap();
            counter += 1;
        }

        let node_new = Node::new(item, None, None);
        let node_new_strong = Rc::new(RefCell::new(node_new));
        let previous_reference =
            Weak::upgrade(&current_node.borrow_mut().prev_as_reference().unwrap()).unwrap();
        let next_reference = current_node.borrow_mut().next_as_reference().unwrap();

        previous_reference
            .borrow_mut()
            .set_next(Some(Rc::clone(&node_new_strong)));
        next_reference
            .borrow_mut()
            .set_prev(Some(Rc::downgrade(&node_new_strong)));
        let mut_ref_node_new = node_new_strong.borrow_mut();
        mut_ref_node_new.set_next(Some(next_reference));
        mut_ref_node_new.set_prev(Some(Rc::downgrade(&previous_reference)));
        Ok(())
    }
}

impl<T> From<Vec<T>> for DoublyLinkedList<T> {
    fn from(collection: Vec<T>) -> Self {
        let mut list: DLL<T> = DLL::build();
        collection.into_iter().for_each(|item| list.push(item).unwrap());
        list
    }
}

impl<T> Into<Vec<T>> for DoublyLinkedList<T> {
    fn into(mut self) -> Vec<T> {
        let mut holder: Vec<T> = vec![];
        while !self.is_empty() {
            holder.push(self.shift().expect("expected a value"));
        }
        holder
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    fn arrange_reference_vector() -> Vec<usize> {
        vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
    }

    fn arrange_test_list() -> DLL<usize> {
        DLL::from(arrange_reference_vector())
    }
    mod doubly_linked_list_tests {

        use super::*;

        #[test]
        fn dll_build_empty_dll() {
            let list: DLL<usize> = DLL::build();
            assert!(list.is_empty());
            assert!(list.head.is_none());
            assert!(list.tail.is_none());
        }

        #[test]
        fn dll_push_node_valid_length() {
            let mut list = arrange_test_list();
            let value_vector = arrange_reference_vector();
            list.push(11);
            assert_eq!(value_vector.len() + 1, list.length());
        }

        #[test]
        fn dll_push_node_valid_order() -> Result<()> {
            let mut list = arrange_test_list();
            let mut value_vector = arrange_reference_vector();
            let value_test = 11;
            list.push(value_test);
            value_vector.push(value_test);
            let list_vector: Vec<usize> = list.into();
            assert_eq!(value_vector, list_vector);
            Ok(())
        }

        #[test]
        fn dll_unshift_node_valid_length() {
            let mut list = arrange_test_list();
            list.unshift(0);
            assert_eq!(11, list.length());
        }

        #[test]
        fn dll_unshift_node_valid_order() {
            let mut list = arrange_test_list();
            let mut value_vector = arrange_reference_vector();
            let value_test = 0;
            list.unshift(value_test);
            value_vector.insert(0, value_test);
            let list_vector: Vec<usize> = list.into();
            assert_eq!(list_vector, value_vector)
        }

        #[test]
        fn dll_build_valid_length() {
            let mut list = arrange_test_list();
            let value_vector = arrange_reference_vector();
            assert_eq!(value_vector.len(), list.length());
        }

        #[test]
        fn dll_from_vector_valid_length() {
            let count = 20;
            let mut list = DLL::from(vec![0; count]);
            assert_eq!(count, list.length());
        }

        #[test]
        fn dll_populated_pop_valid_result() {
            let mut list = arrange_test_list();
            let value_vector = arrange_reference_vector();
            let pop_value = list.pop();
            assert!(pop_value.is_ok());
            assert_eq!(list.length(), value_vector.len() - 1);
            assert_eq!(pop_value.unwrap(), value_vector[value_vector.len() - 1]);
        }

        #[test]
        fn dll_unpopulated_pop_err_empty() {
            let mut list = DLL::<usize>::build();
            assert!(list.is_empty());
            assert!(list.pop().is_err());
        }

        #[test]
        fn dll_populated_pop_fails_from_invalid_strong_count() {
            let mut list = arrange_test_list();
            let tail = list.tail.take().unwrap();
            assert_eq!(2, Rc::strong_count(&tail));
            let _tail_clone = tail.clone_reference();
            list.tail = Some(tail);
            match list.pop() {
                Err(DataStructureError::NonZeroStrongCount(2)) => (),
                val => panic!("should not have received {:?}", val),
            }
        }

        #[test]
        fn dll_populated_shift_valid_result() {
            let mut list = arrange_test_list();
            let value_vector = arrange_reference_vector();
            let shift_value = list.shift();
            assert!(shift_value.is_ok());
            assert_eq!(shift_value.unwrap(), value_vector[0]);
            assert_eq!(list.length(), value_vector.len() - 1);
        }

        #[test]
        fn dll_unpopulated_shift_err_empty() {
            let mut list = DLL::<usize>::build();
            assert!(list.is_empty());
            assert!(list.shift().is_err());
        }

        #[test]
        fn dll_populated_shift_fails_from_invalid_strong_count() -> Result<()> {
            let mut list = arrange_test_list();
            let head = list.head.take().unwrap();
            assert_eq!(2, Rc::strong_count(&head));

            let _head_clone = head.clone_reference();
            list.head = Some(head);
            match list.shift() {
                Ok(_) => panic!("should not have returned Ok"),
                Err(DataStructureError::NonZeroStrongCount(2)) => Ok(()),
                Err(val) => Err(val),
            }
        }

        #[test]
        fn dll_into_vector_valid() {
            let list = arrange_test_list();
            let value_vector = arrange_reference_vector();
            assert_eq!(value_vector, Into::<Vec<usize>>::into(list));
        }

        #[test]
        fn dll_insert_invalid_index() {
            let mut list = arrange_test_list();
            match list.insert(999, 999) {
                Err(DataStructureError::InvalidIndex) => (),
                val => panic!("should not have received {:?}", val),
            }
        }

        #[test]
        fn dll_insert_valid_index() {
            let mut list = arrange_test_list();
            let mut vector = arrange_reference_vector();
            let value = 999;
            list.insert(1, value).unwrap();
            vector.insert(1, value);
            assert_eq!(vector, Into::<Vec<usize>>::into(list));
        }
    }
    mod node_tests {
        use super::*;

        fn arrange_test_node() -> Node<usize> {
            Node::new(100, None, None)
        }

        #[test]
        fn clone_reference_self_increases_strong_count() {
            let node = arrange_test_node();
            assert_eq!(Rc::strong_count(&node), 1);
            let _reference = node.clone_reference();
            assert_eq!(Rc::strong_count(&node), 2);
        }

        #[test]
        fn clone_reference_next_increases_next_strong_count() -> Result<()> {
            let mut list = arrange_test_list();
            let node = list.head_as_reference()?.next_as_reference().unwrap();
            assert_eq!(3, Rc::strong_count(&node));
            let clone = list.head_as_reference()?.next_as_reference().unwrap();
            assert_eq!(4, Rc::strong_count(&clone));
            Ok(())
        }

        #[test]
        fn clone_reference_prev_increases_prev_strong_count() -> Result<()> {
            let mut list = arrange_test_list();
            let node = list.tail_as_reference()?.prev_as_reference().unwrap();
            assert_eq!(3, Rc::strong_count(&node));
            let clone = list.tail_as_reference()?.prev_as_reference().unwrap();
            assert_eq!(4, Rc::strong_count(&clone));
            Ok(())
        }

        #[test]
        fn mutate_value() -> Result<()> {
            let mut node = Node::new(0, None, None);
            assert_eq!(node.borrow().value, 0);
            node.mutate_value(|x| *x += 1);
            assert_eq!(node.borrow().value, 1);
            Ok(())
        }
    }

    #[cfg(test)]
    mod node_iterator_tests {
        use super::*;

        #[test]
        fn node_iter_has_next() {
            let mut list = arrange_test_list();
            let head = list.head_as_reference().unwrap();
            let iter = head.into_iter();
            let mut counter = 0;
            // for _value in iter {
            //     counter += 1;
            //     println!("counter: {:?}\t node: {:?}", counter, _value);
            //     if counter == 20 {
            //         break;
            //     }
            // }
            // panic!("should definitely panic");
        }
    }
}
