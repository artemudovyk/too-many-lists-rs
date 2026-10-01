use std::cell::{Ref, RefCell};
use std::rc::Rc;

pub struct List<T> {
    head: Link<T>,
    tail: Link<T>,
}

type Link<T> = Option<Rc<RefCell<Node<T>>>>;

struct Node<T> {
    elem: T,
    prev: Link<T>,
    next: Link<T>,
}

impl<T> List<T> {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            head: None,
            tail: None,
        }
    }

    pub fn push_front(&mut self, elem: T) {
        let new_head = Rc::new(RefCell::new(Node::new(elem)));
        // Head can't be None while tail is Some, so we can check only one of them to check if the
        // list is empty or not.
        match self.head.take() {
            // Empty list
            None => {
                self.head = Some(new_head.clone());
                self.tail = Some(new_head);
            }
            Some(old_head) => {
                old_head.borrow_mut().next = Some(new_head.clone());
                new_head.borrow_mut().prev = Some(old_head);
                self.head = Some(new_head);
            }
        }
    }

    pub fn pop_front(&mut self) -> Option<T> {
        self.head.take().map(|old_head| {
            match old_head.borrow_mut().prev.take() {
                // If prev is None, then the head is the last element of the list and it's
                // pointed to by both head and tail, so we need to reset tail as well.
                None => {
                    self.tail.take();
                }
                Some(prev) => {
                    // Remove reference to the old_head as it will be popped out of the list
                    prev.borrow_mut().next.take();
                    self.head = Some(prev);
                }
            }
            Rc::try_unwrap(old_head).ok().unwrap().into_inner().elem
        })
    }

    // TODO: continue from here
    pub fn push_back(&mut self, elem: T) {
        let new_tail = Rc::new(RefCell::new(Node::new(elem)));
        // Head can't be None while tail is Some, so we can check only one of them to check if the
        // list is empty or not.
        match self.tail.take() {
            // Empty list
            None => {
                self.head = Some(new_tail.clone());
                self.tail = Some(new_tail);
            }
            Some(old_tail) => {
                old_tail.borrow_mut().prev = Some(new_tail.clone());
                new_tail.borrow_mut().next = Some(old_tail);
                self.tail = Some(new_tail);
            }
        }
    }

    pub fn pop_back(&mut self) -> Option<T> {
        self.tail.take().map(|old_tail| {
            match old_tail.borrow_mut().next.take() {
                // If prev is None, then the tail is the last element of the list and it's
                // pointed to by both head and tail, so we need to reset head as well.
                None => {
                    self.head.take();
                }
                Some(next) => {
                    // Remove reference to the old_tail as it will be popped out of the list
                    next.borrow_mut().prev.take();
                    self.tail = Some(next);
                }
            }
            Rc::try_unwrap(old_tail).ok().unwrap().into_inner().elem
        })
    }

    pub fn peek_front(&self) -> Option<Ref<'_, T>> {
        self.head
            .as_ref()
            .map(|v| Ref::map(v.borrow(), |v| &v.elem))
    }

    pub fn peek_back(&self) -> Option<Ref<'_, T>> {
        self.tail
            .as_ref()
            .map(|v| Ref::map(v.borrow(), |v| &v.elem))
    }
}

pub struct IntoIter<T>(List<T>);

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.pop_front()
    }
}

impl<T> DoubleEndedIterator for IntoIter<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.pop_back()
    }
}

impl<T> IntoIterator for List<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter(self)
    }
}

impl<T> Drop for List<T> {
    fn drop(&mut self) {
        while self.pop_front().is_some() {}
    }
}

impl<T> std::fmt::Display for List<T>
where
    T: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "HEAD")?;
        let mut cursor = self.head.clone();
        while let Some(maybe_node) = cursor {
            let node = maybe_node.borrow();
            write!(f, " <-> {:?}", node.elem)?;
            cursor = node.prev.clone();
        }
        write!(f, " <-> TAIL")
    }
}

impl<T> Node<T> {
    pub fn new(elem: T) -> Self {
        Self {
            elem,
            prev: None,
            next: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_front_adds_new_nodes() {
        let mut list = List::new();
        list.push_front(1);
        assert_eq!(format!("{}", list), "HEAD <-> 1 <-> TAIL");
        list.push_front(2);
        assert_eq!(format!("{}", list), "HEAD <-> 2 <-> 1 <-> TAIL");
        list.push_front(3);
        assert_eq!(format!("{}", list), "HEAD <-> 3 <-> 2 <-> 1 <-> TAIL");
    }

    #[test]
    fn push_front_on_empty_list_inits_head() {
        let mut list = List::new();
        list.push_front(1);
        assert_eq!(&*list.peek_front().unwrap(), &1);
    }

    #[test]
    fn push_front_on_empty_list_inits_tail() {
        let mut list = List::new();
        list.push_front(1);
        assert_eq!(&*list.peek_back().unwrap(), &1);
    }

    #[test]
    fn pop_front_returns_last_front_value() {
        let mut list = List::new();
        list.push_front(1);
        list.push_front(2);
        list.push_front(3);

        assert_eq!(list.pop_front(), Some(3));
        assert_eq!(list.pop_front(), Some(2));
        assert_eq!(list.pop_front(), Some(1));
        assert_eq!(list.pop_front(), None);
    }

    #[test]
    fn push_back_adds_new_nodes() {
        let mut list = List::new();
        list.push_back(1);
        assert_eq!(format!("{}", list), "HEAD <-> 1 <-> TAIL");
        list.push_back(2);
        assert_eq!(format!("{}", list), "HEAD <-> 1 <-> 2 <-> TAIL");
        list.push_back(3);
        assert_eq!(format!("{}", list), "HEAD <-> 1 <-> 2 <-> 3 <-> TAIL");
    }

    #[test]
    fn push_back_on_empty_list_inits_head() {
        let mut list = List::new();
        list.push_back(1);
        assert_eq!(&*list.peek_back().unwrap(), &1);
    }

    #[test]
    fn push_back_on_empty_list_inits_tail() {
        let mut list = List::new();
        list.push_back(1);
        assert_eq!(&*list.peek_back().unwrap(), &1);
    }

    #[test]
    fn pop_back_returns_last_back_value() {
        let mut list = List::new();
        list.push_back(1);
        list.push_back(2);
        list.push_back(3);

        assert_eq!(list.pop_back(), Some(3));
        assert_eq!(list.pop_back(), Some(2));
        assert_eq!(list.pop_back(), Some(1));
        assert_eq!(list.pop_back(), None);
    }

    #[test]
    fn into_iter() {
        let mut list = List::new();
        list.push_front(1);
        list.push_front(2);
        list.push_front(3);

        let mut iter = list.into_iter();
        assert_eq!(iter.next(), Some(3));
        assert_eq!(iter.next_back(), Some(1));
        assert_eq!(iter.next(), Some(2));
        assert_eq!(iter.next_back(), None);
        assert_eq!(iter.next(), None);
    }
}
