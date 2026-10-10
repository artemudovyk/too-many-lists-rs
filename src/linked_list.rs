use std::{iter::FromIterator, marker::PhantomData, ptr::NonNull};

pub struct LinkedList<T> {
    front: Link<T>,
    back: Link<T>,
    len: usize,
    _ghost: PhantomData<T>,
}

type Link<T> = Option<NonNull<Node<T>>>;

struct Node<T> {
    elem: T,
    front: Link<T>,
    back: Link<T>,
}

impl<T> LinkedList<T> {
    pub fn new() -> Self {
        Self {
            front: None,
            back: None,
            len: 0,
            _ghost: PhantomData,
        }
    }

    pub fn push_front(&mut self, elem: T) {
        unsafe {
            let new_front = NonNull::new_unchecked(Node::new(elem).into_ptr());

            if let Some(old_front) = self.front {
                (*new_front.as_ptr()).back = Some(old_front);
                (*old_front.as_ptr()).front = Some(new_front);
            } else {
                // If no front, then the list is empty and we must set a back as well
                self.back = Some(new_front);
            }
            // Move front to the new node
            self.front = Some(new_front);
            self.len += 1;
        }
    }

    pub fn pop_front(&mut self) -> Option<T> {
        unsafe {
            self.front.map(|old_front| {
                // Bring box back to life
                let old_front = Node::box_from_ptr(old_front.as_ptr());

                self.front = old_front.back;
                match self.front {
                    Some(front) => {
                        (*front.as_ptr()).front = None;
                    }
                    None => {
                        self.back = None;
                    }
                }

                self.len -= 1;
                old_front.elem
            })
        }
    }

    pub fn push_back(&mut self, elem: T) {
        unsafe {
            let new_back = NonNull::new_unchecked(Node::new(elem).into_ptr());

            if let Some(old_back) = self.back {
                (*new_back.as_ptr()).front = Some(old_back);
                (*old_back.as_ptr()).back = Some(new_back);
            } else {
                // If no back, then the list is empty and we must set a back as well
                self.front = Some(new_back);
            }
            // Move back to the new node
            self.back = Some(new_back);
            self.len += 1;
        }
    }

    pub fn pop_back(&mut self) -> Option<T> {
        unsafe {
            self.back.map(|old_back| {
                // Bring box back to life
                let old_back = Node::box_from_ptr(old_back.as_ptr());

                self.back = old_back.front;
                match self.back {
                    Some(back) => {
                        (*back.as_ptr()).back = None;
                    }
                    None => {
                        self.front = None;
                    }
                }

                self.len -= 1;
                old_back.elem
            })
        }
    }

    pub fn front(&self) -> Option<&T> {
        unsafe { self.front.map(|v| &(*v.as_ptr()).elem) }
    }

    pub fn front_mut(&mut self) -> Option<&mut T> {
        unsafe { self.front.map(|v| &mut (*v.as_ptr()).elem) }
    }

    pub fn back(&self) -> Option<&T> {
        unsafe { self.back.map(|v| &(*v.as_ptr()).elem) }
    }

    pub fn back_mut(&mut self) -> Option<&mut T> {
        unsafe { self.back.map(|v| &mut (*v.as_ptr()).elem) }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            front: self.front,
            back: self.back,
            len: self.len,
            _phantom: PhantomData,
        }
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        IterMut {
            front: self.front,
            back: self.back,
            len: self.len,
            _phantom: PhantomData,
        }
    }

    pub fn clear(&mut self) {
        while self.pop_front().is_some() {}
    }
}

impl<T> Default for LinkedList<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Drop for LinkedList<T> {
    fn drop(&mut self) {
        while self.pop_front().is_some() {}
    }
}

pub struct Iter<'a, T> {
    front: Link<T>,
    back: Link<T>,
    len: usize,
    _phantom: PhantomData<&'a T>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.front.map(|link| unsafe {
            self.front = (*link.as_ptr()).back;
            self.len -= 1;
            &(*link.as_ptr()).elem
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<'a, T> DoubleEndedIterator for Iter<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.back.map(|link| unsafe {
            self.back = (*link.as_ptr()).front;
            self.len -= 1;
            &(*link.as_ptr()).elem
        })
    }
}

impl<'a, T> ExactSizeIterator for Iter<'a, T> {
    fn len(&self) -> usize {
        self.len
    }
}

pub struct IntoIter<T>(LinkedList<T>);

impl<T> IntoIterator for LinkedList<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter(self)
    }
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.pop_front()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.0.len, Some(self.0.len))
    }
}

impl<T> DoubleEndedIterator for IntoIter<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.pop_back()
    }
}

impl<T> Node<T> {
    pub fn new(elem: T) -> Self {
        Self {
            elem,
            front: None,
            back: None,
        }
    }

    pub fn into_ptr(self) -> *mut Node<T> {
        Box::into_raw(Box::new(self))
    }

    pub fn box_from_ptr(ptr: *mut Node<T>) -> Box<Node<T>> {
        unsafe { Box::from_raw(ptr) }
    }
}

pub struct IterMut<'a, T> {
    front: Link<T>,
    back: Link<T>,
    len: usize,
    _phantom: PhantomData<&'a mut T>,
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        self.front.map(|link| unsafe {
            self.front = (*link.as_ptr()).back;
            self.len -= 1;
            &mut (*link.as_ptr()).elem
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<'a, T> DoubleEndedIterator for IterMut<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.back.map(|link| unsafe {
            self.back = (*link.as_ptr()).front;
            self.len -= 1;
            &mut (*link.as_ptr()).elem
        })
    }
}

impl<T: Clone> Clone for LinkedList<T> {
    fn clone(&self) -> Self {
        let mut cloned = LinkedList::new();
        for item in self.iter() {
            cloned.push_back(item.clone());
        }
        cloned
    }
}

impl<T> Extend<T> for LinkedList<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for item in iter {
            self.push_front(item);
        }
    }
}

impl<T> FromIterator<T> for LinkedList<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut list = Self::new();
        for item in iter {
            list.push_front(item);
        }
        list
    }
}

#[cfg(test)]
mod test {
    use std::iter::FromIterator;

    use super::LinkedList;

    #[test]
    fn test_basic_front() {
        let mut list = LinkedList::new();

        // Try to break an empty list
        assert_eq!(list.len(), 0);
        assert_eq!(list.pop_front(), None);
        assert_eq!(list.len(), 0);

        // Try to break a one item list
        list.push_front(10);
        assert_eq!(list.len(), 1);
        assert_eq!(list.pop_front(), Some(10));
        assert_eq!(list.len(), 0);
        assert_eq!(list.pop_front(), None);
        assert_eq!(list.len(), 0);

        // Mess around
        list.push_front(10);
        assert_eq!(list.len(), 1);
        list.push_front(20);
        assert_eq!(list.len(), 2);
        list.push_front(30);
        assert_eq!(list.len(), 3);
        assert_eq!(list.pop_front(), Some(30));
        assert_eq!(list.len(), 2);
        list.push_front(40);
        assert_eq!(list.len(), 3);
        assert_eq!(list.pop_front(), Some(40));
        assert_eq!(list.len(), 2);
        assert_eq!(list.pop_front(), Some(20));
        assert_eq!(list.len(), 1);
        assert_eq!(list.pop_front(), Some(10));
        assert_eq!(list.len(), 0);
        assert_eq!(list.pop_front(), None);
        assert_eq!(list.len(), 0);
        assert_eq!(list.pop_front(), None);
        assert_eq!(list.len(), 0);
    }

    #[test]
    fn test_basic_back() {
        let mut list = LinkedList::new();

        // Try to break an empty list
        assert_eq!(list.len(), 0);
        assert_eq!(list.pop_back(), None);
        assert_eq!(list.len(), 0);

        // Try to break a one item list
        list.push_back(10);
        assert_eq!(list.len(), 1);
        assert_eq!(list.pop_back(), Some(10));
        assert_eq!(list.len(), 0);
        assert_eq!(list.pop_back(), None);
        assert_eq!(list.len(), 0);

        // Mess around
        list.push_back(10);
        assert_eq!(list.len(), 1);
        list.push_back(20);
        assert_eq!(list.len(), 2);
        list.push_back(30);
        assert_eq!(list.len(), 3);
        assert_eq!(list.pop_back(), Some(30));
        assert_eq!(list.len(), 2);
        list.push_back(40);
        assert_eq!(list.len(), 3);
        assert_eq!(list.pop_back(), Some(40));
        assert_eq!(list.len(), 2);
        assert_eq!(list.pop_back(), Some(20));
        assert_eq!(list.len(), 1);
        assert_eq!(list.pop_back(), Some(10));
        assert_eq!(list.len(), 0);
        assert_eq!(list.pop_back(), None);
        assert_eq!(list.len(), 0);
        assert_eq!(list.pop_back(), None);
        assert_eq!(list.len(), 0);
    }

    #[test]
    fn test_iter_front() {
        let mut list = LinkedList::new();
        list.push_front(1);
        list.push_front(2);
        list.push_front(3);

        let mut iter = list.iter();
        assert_eq!(iter.next(), Some(&3));
        assert_eq!(iter.next(), Some(&2));
        assert_eq!(iter.next(), Some(&1));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_iter_back() {
        let mut list = LinkedList::new();
        list.push_back(3);
        list.push_back(2);
        list.push_back(1);

        let mut iter = list.iter();
        assert_eq!(iter.next_back(), Some(&1));
        assert_eq!(iter.next_back(), Some(&2));
        assert_eq!(iter.next_back(), Some(&3));
        assert_eq!(iter.next_back(), None);
    }

    #[test]
    fn test_into_iter() {
        let mut list = LinkedList::new();
        list.push_front(3);
        list.push_front(2);
        list.push_front(1);

        let mut iter = list.into_iter();
        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.next(), Some(2));
        assert_eq!(iter.next(), Some(3));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_into_iter_back() {
        let mut list = LinkedList::new();
        list.push_back(1);
        list.push_back(2);
        list.push_back(3);

        let mut iter = list.into_iter();
        assert_eq!(iter.next_back(), Some(3));
        assert_eq!(iter.next_back(), Some(2));
        assert_eq!(iter.next_back(), Some(1));
        assert_eq!(iter.next_back(), None);
    }

    #[test]
    fn test_iter_mut() {
        let mut list = LinkedList::new();
        list.push_front(1);
        list.push_front(2);
        list.push_front(3);

        let mut iter = list.iter_mut();
        assert_eq!(iter.next(), Some(&mut 3));
        assert_eq!(iter.next(), Some(&mut 2));
        assert_eq!(iter.next(), Some(&mut 1));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_iter_mut_back() {
        let mut list = LinkedList::new();
        list.push_back(1);
        list.push_back(2);
        list.push_back(3);

        let mut iter = list.iter_mut();
        assert_eq!(iter.next_back(), Some(&mut 3));
        assert_eq!(iter.next_back(), Some(&mut 2));
        assert_eq!(iter.next_back(), Some(&mut 1));
        assert_eq!(iter.next_back(), None);
    }

    #[test]
    fn is_empty_true_when_list_has_no_nodes() {
        let list: LinkedList<i32> = LinkedList::new();
        assert!(list.is_empty())
    }

    #[test]
    fn is_empty_false_when_list_has_nodes() {
        let mut list = LinkedList::new();
        list.push_front(10);
        assert!(!list.is_empty())
    }

    #[test]
    fn clone_creates_a_deep_copy() {
        let mut list1 = LinkedList::new();
        list1.push_front("a");
        list1.push_front("b");
        list1.push_front("c");

        let list2 = list1.clone();

        assert_eq!(list1.len(), list2.len());
        for (item1, item2) in list1.iter().zip(list2.iter()) {
            // Values are the same
            assert_eq!(item1, item2);
            // But memory addresses are different
            assert!(!std::ptr::eq(item1, item2));
        }
    }

    #[test]
    fn from_iter() {
        let items = ["a", "b", "c"];

        let mut list = LinkedList::from_iter(items.iter());

        assert_eq!(list.len(), 3);
        assert_eq!(list.pop_front(), Some(&"c"));
        assert_eq!(list.pop_front(), Some(&"b"));
        assert_eq!(list.pop_front(), Some(&"a"));
    }
}
