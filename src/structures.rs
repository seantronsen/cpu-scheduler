use std::{cell::RefCell, ops::Deref, rc::Rc, result};

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
pub struct NodeValue<T> {
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
pub type ReferenceNode<T> = Rc<RefCell<NodeValue<T>>>;

#[derive(Debug)]
pub struct Node<T>(ReferenceNode<T>);

impl<T> Deref for Node<T> {
    type Target = ReferenceNode<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> Node<T> {
    fn new(value: T, next: Option<Node<T>>, prev: Option<Node<T>>) -> Self {
        let node_value: NodeValue<T> = NodeValue::new(value, next, prev);
        let reference = Rc::new(RefCell::new(node_value));
        Self(reference)
    }

    // might need to think about weak pointer references here
    // othewise we might run into a 'memory leak'
    pub fn clone_reference(&self) -> Self {
        Self(Rc::clone(&self))
    }

    fn set_next(&self, next: PotentialNode<T>) {
        let mut value_ref = self.borrow_mut();
        value_ref.next = next;
    }

    fn set_prev(&self, prev: PotentialNode<T>) {
        let mut value_ref = self.borrow_mut();
        value_ref.prev = prev;
    }

    pub fn mutate_value(&mut self, mut fn_mut: impl FnMut(&mut T)) {
        fn_mut(&mut (self.borrow_mut()).value);
    }

    pub fn clone_next_reference(&mut self) -> PotentialNode<T> {
        let mut value_ref = self.borrow_mut();
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

    pub fn clone_prev_reference(&mut self) -> PotentialNode<T> {
        let mut value_ref = self.borrow_mut();
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

impl<T> DoubleEndedIterator for Node<T> {
    fn next_back(&mut self) -> Option<Node<T>> {
        self.clone_prev_reference()
    }
}

pub struct DoublyLinkedList<T> {
    head: PotentialNode<T>,
    tail: PotentialNode<T>,
}

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
        let holder = self.head.take().expect("head didn't have a node");
        let mut current_node = holder.clone_reference();
        self.head.replace(holder);

        while let Some(node_ref) = current_node.clone_next_reference() {
            counter += 1;
            current_node = node_ref;
        }

        counter
    }

    pub fn clone_head_reference(&mut self) -> Result<Node<T>> {
        let head = match self.head.take() {
            Some(node_ref) => node_ref,
            None => return Err(DataStructureError::InvalidActionEmpty),
        };
        let reference = head.clone_reference();
        self.head.replace(head);
        Ok(reference)
    }

    pub fn clone_tail_reference(&mut self) -> Result<Node<T>> {
        let tail = match self.tail.take() {
            Some(node_ref) => node_ref,
            None => return Err(DataStructureError::InvalidActionEmpty),
        };
        let reference = tail.clone_reference();
        self.tail.replace(tail);
        Ok(reference)
    }

    pub fn push(&mut self, item: T) {
        let node: Node<T> = Node::new(item, None, None);

        // start new head
        if self.head.is_none() {
            let node_clone = node.clone_reference();
            self.head = Some(node);
            self.tail = Some(node_clone);
            return;
        }

        // append to tail and reassign original tail
        let tail = self.tail.take().expect("list tail didn't have a node");
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
            None => self.head = None,
        };

        Ok(Rc::<RefCell<NodeValue<T>>>::try_unwrap(old_tail.0)?
            .into_inner()
            .value)
    }
    pub fn unshift(&mut self, item: T) {
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

    pub fn shift(&mut self) -> Result<T> {
        let mut old_head = match self.head.take() {
            Some(node_ref) => node_ref,
            None => return Err(DataStructureError::InvalidActionEmpty),
        };

        match old_head.clone_next_reference() {
            Some(node_ref) => {
                node_ref.set_prev(None);
                self.head = Some(node_ref);
            }
            None => self.tail = None,
        }

        Ok(Rc::<RefCell<NodeValue<T>>>::try_unwrap(old_head.0)?
            .into_inner()
            .value)
    }

    pub fn insert(&mut self, index: usize, item: T) -> Result<()> {
        let length = self.length();
        if index > length {
            return Err(DataStructureError::InvalidIndex);
        } else if index == 0 {
            return Ok(self.unshift(item));
        } else if index == length {
            return Ok(self.push(item));
        }

        let mut counter: usize = 0;
        let holder = self.head.take().expect("head didn't have a node");
        let mut current_node = holder.clone_reference();
        self.head.replace(holder);

        while counter != index {
            current_node = current_node.clone_next_reference().unwrap();
            counter += 1;
        }

        let new_node = Node::new(item, None, None);
        let previous_reference = current_node
            .clone_prev_reference()
            .expect("no previous node");
        let next_reference = current_node;
        previous_reference.set_next(Some(new_node.clone_reference()));
        next_reference.set_prev(Some(new_node.clone_reference()));
        new_node.set_prev(Some(previous_reference));
        new_node.set_next(Some(next_reference));
        Ok(())
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
            let node = list.clone_head_reference()?.clone_next_reference().unwrap();
            assert_eq!(3, Rc::strong_count(&node));
            let clone = list.clone_head_reference()?.clone_next_reference().unwrap();
            assert_eq!(4, Rc::strong_count(&clone));
            Ok(())
        }

        #[test]
        fn clone_reference_prev_increases_prev_strong_count() -> Result<()> {
            let mut list = arrange_test_list();
            let node = list.clone_tail_reference()?.clone_prev_reference().unwrap();
            assert_eq!(3, Rc::strong_count(&node));
            let clone = list.clone_tail_reference()?.clone_prev_reference().unwrap();
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
            let head = list.clone_head_reference().unwrap();
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
