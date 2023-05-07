use std::{
    cell::RefCell,
    ops::{Deref, DerefMut},
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

#[derive(Debug)]
struct WeakNode<T>(Weak<RefCell<NodeValue<T>>>);

impl<T> WeakNode<T> {
    fn to_strong(&self) -> Result<StrongNode<T>> {
        let strong =
            StrongNode(Weak::upgrade(&self.0).ok_or_else(|| DataStructureError::InvalidReference)?);
        Ok(strong)
    }
}

#[derive(Debug)]
struct StrongNode<T>(Rc<RefCell<NodeValue<T>>>);

impl<T> StrongNode<T> {
    fn to_weak(&self) -> WeakNode<T> {
        WeakNode(Rc::downgrade(&self.0))
    }

    fn destruct(self) -> Result<NodeValue<T>> {
        Ok(Rc::<RefCell<NodeValue<T>>>::try_unwrap(self.0)?.into_inner())
    }

}

impl<T> Deref for StrongNode<T> {
    type Target = NodeValue<T>;

    fn deref(&self) -> &Self::Target {
        &self.0.borrow()
    }
}

impl<T> DerefMut for StrongNode<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0.borrow_mut()
    }
}

#[allow(dead_code)]
#[derive(Debug)]
struct NodeValue<T> {
    value: T,
    next: Option<StrongNode<T>>,
    prev: Option<WeakNode<T>>,
}

impl<T> NodeValue<T> {
    fn new(value: T, next: Option<StrongNode<T>>, prev: Option<WeakNode<T>>) -> Self {
        Self { value, next, prev }
    }
}

struct NodeValueIter<T> {
    current: Option<NodeValue<T>>,
}

impl<T> Iterator for NodeValueIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        match self.current.take() {
            Some(node_value) => {
                let NodeValue { value, next, prev } = node_value;
                self.current = match next {
                    Some(node) => Some(node.destruct().unwrap()),
                    None => None,
                };
                Some(value)
            }

            None => None,
        }
    }
}
