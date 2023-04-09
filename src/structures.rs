/*
 *
 * todos:
 * - convert the dll implementations to return the result type: Ok(T) | Err(T)
 *
 *
 */

use std::{cell::RefCell, rc::Rc, result};

// data structures required for RR related algorithms

#[derive(Debug)]
pub enum DataStructureError {
    InvalidState,
    InvalidReference,
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
struct NodeValue<T> {
    value: T,
    next: PotentialNode<T>,
    prev: PotentialNode<T>,
}
impl<T> NodeValue<T> {
    fn new(value: T, next: PotentialNode<T>, prev: PotentialNode<T>) -> Self {
        Self { value, next, prev }
    }
}

// shorthand type
type PotentialNode<T> = Option<Node<T>>;
type ReferenceNode<T> = Rc<RefCell<NodeValue<T>>>;

struct Node<T> {
    reference: ReferenceNode<T>,
}

impl<T> Node<T> {
    fn new(value: T, next: Option<Node<T>>, prev: Option<Node<T>>) -> Self {
        let node_value: NodeValue<T> = NodeValue::new(value, next, prev);
        let reference = Rc::new(RefCell::new(node_value));

        Self { reference }
    }

    // might need to think about weak pointer references here
    // othewise we might run into a 'memory leak'
    fn clone_reference(&self) -> Self {
        Self {
            reference: Rc::clone(&self.reference),
        }
    }

    fn set_next(&self, next: PotentialNode<T>) {
        let mut value_ref = self.reference.borrow_mut();
        value_ref.next = next;
    }

    fn set_prev(&self, prev: PotentialNode<T>) {
        let mut value_ref = self.reference.borrow_mut();
        value_ref.prev = prev;
    }

    fn clone_next_reference(&mut self) -> PotentialNode<T> {
        let mut value_ref = self.reference.borrow_mut();
        let next = value_ref.next.take();
        match next {
            Some(node) => {
                let clone = node.clone_reference();
                value_ref.next.replace(node);
                Some(clone)
            }
            None => None,
        }
    }

    fn clone_prev_reference(&mut self) -> PotentialNode<T> {
        let mut value_ref = self.reference.borrow_mut();
        let prev = value_ref.prev.take();

        match prev {
            Some(node) => {
                let clone = node.clone_reference();
                value_ref.prev.replace(node);
                Some(clone)
            }

            None => None,
        }
    }
}

impl<T> Iterator for Node<T> {
    fn next(&mut self) -> Option<Self::Item> {
        self.clone_next_reference()
    }

    type Item = Node<T>;
}

#[allow(dead_code)]
struct DoublyLinkedList<T> {
    head: PotentialNode<T>,
    tail: PotentialNode<T>,
}

type DLL<T> = DoublyLinkedList<T>;

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
        let holder = self.head.take().expect("head didn't have a node");
        let mut current_node = holder.clone_reference();
        self.head.replace(holder);

        while let Some(node_ref) = current_node.clone_next_reference() {
            counter += 1;
            current_node = node_ref;
        }

        counter
    }

    fn push(&mut self, item: T) {
        let node: Node<T> = Node::new(item, None, None);

        // start new head
        if self.head.is_none() {
            let node_clone = node.clone_reference();
            self.head = Some(node);
            self.tail = Some(node_clone);
            return;
        }

        // append to tail and reassign original tail
        let tail = self.tail.take().expect("tail didn't have a node");
        node.set_prev(Some(tail.clone_reference()));
        tail.set_next(Some(node.clone_reference()));
        self.tail = Some(node);
    }

    pub fn pop(&mut self) -> Result<T> {
        let mut old_tail = match self.tail.take() {
            Some(node_ref) => node_ref,
            None => return Err(DataStructureError::InvalidActionEmpty),
        };
        match old_tail.clone_prev_reference() {
            Some(node_ref) => {
                node_ref.set_next(None);
                self.tail = Some(node_ref);
            }
            None => {
                self.head = None;
            }
        };

        Ok(Rc::<RefCell<NodeValue<T>>>::try_unwrap(old_tail.reference)?
            .into_inner()
            .value)
    }
    fn unshift(&mut self, item: T) {
        let node: Node<T> = Node::new(item, None, None);

        // start new tail
        if self.tail.is_none() {
            let node_clone = node.clone_reference();
            self.tail = Some(node);
            self.head = Some(node_clone);
            return;
        }
        // append to head and reassign original head
        let head = self.head.take().expect("head didn't have a node");
        node.set_next(Some(head.clone_reference()));
        head.set_prev(Some(node.clone_reference()));
        self.head = Some(node);
    }

    pub fn shift(&mut self) -> T {
        todo!();
    }
}

impl<T> From<Vec<T>> for DoublyLinkedList<T> {
    fn from(collection: Vec<T>) -> Self {
        let mut list: DLL<T> = DLL::build();
        collection.into_iter().for_each(|item| list.push(item));
        list
    }
}

impl<T> Into<Vec<T>> for DoublyLinkedList<T> {
    fn into(self) -> Vec<T> {
        todo!()
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    mod doubly_linked_list_tests {

        use super::*;

        fn build_test_list() -> DLL<usize> {
            DLL::from(vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10])
        }

        #[test]
        fn dll_build_empty_dll() {
            let list: DLL<usize> = DLL::build();
            assert!(list.is_empty());
            assert!(list.head.is_none());
            assert!(list.tail.is_none());
        }

        #[test]
        fn dll_push_node_valid_length() {
            todo!()
        }

        #[test]
        fn dll_push_node_valid_order() {
            todo!()
        }

        #[test]
        fn dll_unshift_node_valid_length() {
            todo!()
        }

        #[test]
        fn dll_unshift_node_valid_order() {
            todo!()
        }

        #[test]
        fn dll_build_valid_length() {
            let mut list = build_test_list();
            assert_eq!(10, list.length());
        }

        #[test]
        fn dll_from_vector_valid_length() {
            let mut list = DLL::from(vec![0; 20]);
            assert_eq!(20, list.length());
        }

        // todo: ensure you test the length
        #[test]
        fn dll_populated_pop_valid_result() {
            todo!();
        }

        #[test]
        fn dll_unpopulated_pop_err_empty() {
            todo!();
        }

        #[test]
        fn dll_populated_pop_decreases_old_tail_strong_count() {
            todo!()
        }

        #[test]
        fn dll_populated_shift_valid_result() {
            todo!();
        }

        #[test]
        fn dll_unpopulated_shift_err_empty() {
            todo!();
        }
        #[test]
        fn dll_populated_shift_decreases_old_head_strong_count() {
            todo!()
        }
    }
    mod node_tests {
        use super::*;

        fn arrange_test_node() -> Node<usize> {
            Node::new(100, None, None)
        }

        #[test]
        fn reminder_make_tests() {
            todo!()
        }

        #[test]
        fn clone_reference_self_increases_strong_count() {
            todo!()
        }

        #[test]
        fn clone_reference_next_increases_next_strong_count() {
            todo!()
        }

        #[test]
        fn clone_reference_prev_increases_prev_strong_count() {
            todo!()
        }
    }
}
