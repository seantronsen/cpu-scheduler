use std::{cell::RefCell, rc::Rc};

use crate::sim::SimProcess;
pub fn fcfs(mut incoming: Vec<SimProcess>) -> Vec<SimProcess> {
    let mut finished: Vec<SimProcess> = vec![];
    incoming.reverse();

    while incoming.len() != 0 {
        let mut process_current = incoming.pop().unwrap();
        for process_next in incoming.iter_mut() {
            process_next.wait += process_current.burst;
        }
        process_current.burst = 0;
        finished.push(process_current);
    }

    finished
}

// sorting for sjf and so on
fn mergesort(collection: Vec<SimProcess>) -> Vec<SimProcess> {
    let length = collection.len();
    if length == 0 {
        return collection;
    } else {
        let mut collection: Vec<Option<SimProcess>> =
            collection.into_iter().map(|x| Some(x)).collect();
        thunk_mergesort(&mut collection[..], length)
            .into_iter()
            .map(|x| x.unwrap())
            .collect::<Vec<SimProcess>>()
    }
}

fn thunk_mergesort(
    collection: &mut [Option<SimProcess>],
    length: usize,
) -> Vec<Option<SimProcess>> {
    let mut result = Vec::with_capacity(length);
    // do nothing, already sorted
    if length == 1 {
        result.push(collection[0].take());
        result
    } else {
        let len_a = length / 2;
        let len_b = length - len_a;
        let (collection_a, collection_b) = collection.split_at_mut(len_a);
        let mut collection_a = thunk_mergesort(collection_a, len_a);
        let mut collection_b = thunk_mergesort(collection_b, len_b);

        // merge
        let mut index_a = 0;
        let mut index_b = 0;

        for _ in 0..length {
            if index_a < len_a
                && (index_b == len_b || &collection_a[index_a] <= &collection_b[index_b])
            {
                result.push(collection_a[index_a].take());
                index_a += 1;
            } else if index_b < len_b
                && (index_a == len_a || &collection_b[index_b] <= &collection_a[index_a])
            {
                result.push(collection_b[index_b].take());
                index_b += 1;
            }
        }

        result
    }
}

// the following two functions are defined explicitly on purpose, despite having identical content.
// since simple priority and shortest-job-first scheduling involve only sorting, there's little
// else to do. the above implementation of mergesort relies on the traits implemented on the types.
// as such, the job was already completed before these functions were written.
pub fn sjf(incoming: Vec<SimProcess>) -> Vec<SimProcess> {
    fcfs(mergesort(incoming))
}

pub fn priority(incoming: Vec<SimProcess>) -> Vec<SimProcess> {
    fcfs(mergesort(incoming))
}

// data structures required for RR related algorithms

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

    fn append(&mut self, item: T) {
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

    fn prepend(&mut self, item: T) {
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
}

impl<T> From<Vec<T>> for DoublyLinkedList<T> {
    fn from(collection: Vec<T>) -> Self {
        let mut list: DLL<T> = DLL::build();
        collection.into_iter().for_each(|item| list.append(item));

        list
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    mod doubly_linked_list_tests {

        use super::*;

        #[test]
        fn dll_build_empty_dll() {
            let list: DLL<usize> = DLL::build();

            assert!(list.head.is_none());
            assert!(list.tail.is_none());
        }

        #[test]
        fn dll_build_valid_length() {
            let mut list: DLL<usize> = DLL::build();
            list.append(0);
            assert_eq!(1, list.length());
            list.append(1);
            assert_eq!(2, list.length());
            list.append(2);
            assert_eq!(3, list.length());
        }

        #[test]
        fn dll_from_vector_valid_length() {
            let mut list = DLL::from(vec![0, 0, 0]);
            assert_eq!(3, list.length());
        }
    }
}
