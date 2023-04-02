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
type PotentialNode<T> = Option<Rc<RefCell<DLLNode<T>>>>;
#[allow(dead_code)]
struct DLLNode<T> {
    value: T,
    next: PotentialNode<T>,
    prev: PotentialNode<T>,
}

#[allow(dead_code)]
impl<T> DLLNode<T> {
    fn new(value: T, next: PotentialNode<T>, prev: PotentialNode<T>) -> Self {
        Self { value, next, prev }
    }
}

#[allow(dead_code)]
struct DoublyLinkedList<T> {
    pub head: PotentialNode<T>,
    pub tail: PotentialNode<T>,
}

#[allow(dead_code)]
impl<T> DoublyLinkedList<T> {
    fn build() -> Self {
        Self {
            head: None,
            tail: None,
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    mod doubly_linked_list_tests {

        use super::*;

        #[test]
        fn dll_build_empty_dll() {
            let list: DoublyLinkedList<usize> = DoublyLinkedList::build();

            assert!(list.head.is_none());
            assert!(list.tail.is_none());
        }
    }
}
