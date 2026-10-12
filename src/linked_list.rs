use core::fmt;
use std::hash::Hash;
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
        if self.len > 0 {
            self.front.map(|link| unsafe {
                self.front = (*link.as_ptr()).back;
                self.len -= 1;
                &(*link.as_ptr()).elem
            })
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<'a, T> DoubleEndedIterator for Iter<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.len > 0 {
            self.back.map(|link| unsafe {
                self.back = (*link.as_ptr()).front;
                self.len -= 1;
                &(*link.as_ptr()).elem
            })
        } else {
            None
        }
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

impl<T> ExactSizeIterator for IntoIter<T> {
    fn len(&self) -> usize {
        self.0.len
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
        if self.len > 0 {
            self.front.map(|link| unsafe {
                self.front = (*link.as_ptr()).back;
                self.len -= 1;
                &mut (*link.as_ptr()).elem
            })
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<'a, T> DoubleEndedIterator for IterMut<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.len > 0 {
            self.back.map(|link| unsafe {
                self.back = (*link.as_ptr()).front;
                self.len -= 1;
                &mut (*link.as_ptr()).elem
            })
        } else {
            None
        }
    }
}

impl<'a, T> ExactSizeIterator for IterMut<'a, T> {
    fn len(&self) -> usize {
        self.len
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
            list.push_back(item);
        }
        list
    }
}

impl<T: fmt::Debug> fmt::Debug for LinkedList<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

impl<T: PartialEq> PartialEq for LinkedList<T> {
    fn eq(&self, other: &Self) -> bool {
        self.len() == other.len() && self.iter().eq(other.iter())
    }
}

impl<T: Eq> Eq for LinkedList<T> {}

impl<T: PartialOrd> PartialOrd for LinkedList<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.iter().partial_cmp(other.iter())
    }
}

impl<T: Ord> Ord for LinkedList<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.iter().cmp(other.iter())
    }
}

impl<T: Hash> Hash for LinkedList<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.len().hash(state);
        for item in self.iter() {
            item.hash(state);
        }
    }
}

#[cfg(test)]
mod test {
    use super::LinkedList;
    use std::hash::Hash;
    use std::iter::FromIterator;

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
        assert_eq!(list.pop_front(), Some(&"a"));
        assert_eq!(list.pop_front(), Some(&"b"));
        assert_eq!(list.pop_front(), Some(&"c"));
    }

    #[test]
    fn debug() {
        let list = LinkedList::from_iter(["a", "b", "c"]);
        assert_eq!(format!("{:?}", list), "[\"a\", \"b\", \"c\"]")
    }

    #[test]
    fn two_lists_are_equal_when_all_elements_are_the_same_and_in_the_same_order() {
        let list1 = LinkedList::from_iter([1, 2, 3]);
        let list2 = LinkedList::from_iter([1, 2, 3]);
        assert!(list1 == list2);
    }

    #[test]
    fn two_lists_are_not_equal_when_at_least_one_element_is_different() {
        let list1 = LinkedList::from_iter([1, 2, 3]);
        let list2 = LinkedList::from_iter([1, 10, 3]);
        assert!(list1 != list2);
    }

    #[test]
    fn two_lists_are_not_equal_when_same_elements_are_in_different_order() {
        let list1 = LinkedList::from_iter([1, 2, 3]);
        let list2 = LinkedList::from_iter([3, 2, 1]);
        assert!(list1 != list2);
    }

    #[test]
    fn two_lists_are_not_equal_when_they_have_different_lengths() {
        let list1 = LinkedList::from_iter([1, 2, 3]);
        let list2 = LinkedList::from_iter([1, 2, 3, 4]);
        assert!(list1 != list2);
    }

    mod hash {
        use super::*;
        use std::hash::{DefaultHasher, Hasher};

        #[test]
        fn two_lists_have_the_same_hash_if_they_are_equal() {
            let list1 = LinkedList::from_iter([1, 2, 3]);
            let list2 = LinkedList::from_iter([1, 2, 3]);
            let hash1 = {
                let mut hasher = DefaultHasher::new();
                list1.hash(&mut hasher);
                hasher.finish()
            };
            let hash2 = {
                let mut hasher = DefaultHasher::new();
                list2.hash(&mut hasher);
                hasher.finish()
            };
            assert_eq!(hash1, hash2)
        }

        #[test]
        fn two_lists_have_different_hash_if_elements_are_the_same_but_order_different() {
            let list1 = LinkedList::from_iter([1, 2, 3]);
            let list2 = LinkedList::from_iter([3, 2, 1]);
            let hash1 = {
                let mut hasher = DefaultHasher::new();
                list1.hash(&mut hasher);
                hasher.finish()
            };
            let hash2 = {
                let mut hasher = DefaultHasher::new();
                list2.hash(&mut hasher);
                hasher.finish()
            };
            assert_ne!(hash1, hash2)
        }

        #[test]
        fn lists_with_nested_lists_have_different_hashes_if_they_are_not_equal() {
            let list1 =
                LinkedList::from_iter([LinkedList::from_iter([1, 2]), LinkedList::from_iter([3])]);
            let list2 =
                LinkedList::from_iter([LinkedList::from_iter([1]), LinkedList::from_iter([2, 3])]);
            let hash1 = {
                let mut hasher = DefaultHasher::new();
                list1.hash(&mut hasher);
                hasher.finish()
            };
            let hash2 = {
                let mut hasher = DefaultHasher::new();
                list2.hash(&mut hasher);
                hasher.finish()
            };
            assert_ne!(hash1, hash2)
        }
    }
}

#[cfg(test)]
mod tests_from_the_book {
    use super::LinkedList;

    fn generate_test() -> LinkedList<i32> {
        list_from(&[0, 1, 2, 3, 4, 5, 6])
    }

    fn list_from<T: Clone>(v: &[T]) -> LinkedList<T> {
        v.iter().map(|x| (*x).clone()).collect()
    }

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
    fn test_basic() {
        let mut m = LinkedList::new();
        assert_eq!(m.pop_front(), None);
        assert_eq!(m.pop_back(), None);
        assert_eq!(m.pop_front(), None);
        m.push_front(1);
        assert_eq!(m.pop_front(), Some(1));
        m.push_back(2);
        m.push_back(3);
        assert_eq!(m.len(), 2);
        assert_eq!(m.pop_front(), Some(2));
        assert_eq!(m.pop_front(), Some(3));
        assert_eq!(m.len(), 0);
        assert_eq!(m.pop_front(), None);
        m.push_back(1);
        m.push_back(3);
        m.push_back(5);
        m.push_back(7);
        assert_eq!(m.pop_front(), Some(1));

        let mut n = LinkedList::new();
        n.push_front(2);
        n.push_front(3);
        {
            assert_eq!(n.front().unwrap(), &3);
            let x = n.front_mut().unwrap();
            assert_eq!(*x, 3);
            *x = 0;
        }
        {
            assert_eq!(n.back().unwrap(), &2);
            let y = n.back_mut().unwrap();
            assert_eq!(*y, 2);
            *y = 1;
        }
        assert_eq!(n.pop_front(), Some(0));
        assert_eq!(n.pop_front(), Some(1));
    }

    #[test]
    fn test_iterator() {
        let m = generate_test();
        dbg!(&m);
        for (i, elt) in m.iter().enumerate() {
            assert_eq!(i as i32, *elt);
        }
        let mut n = LinkedList::new();
        assert_eq!(n.iter().next(), None);
        n.push_front(4);
        let mut it = n.iter();
        assert_eq!(it.size_hint(), (1, Some(1)));
        assert_eq!(it.next().unwrap(), &4);
        assert_eq!(it.size_hint(), (0, Some(0)));
        assert_eq!(it.next(), None);
    }

    #[test]
    fn test_iterator_double_end() {
        let mut n = LinkedList::new();
        assert_eq!(n.iter().next(), None);
        n.push_front(4);
        n.push_front(5);
        n.push_front(6);
        let mut it = n.iter();
        assert_eq!(it.size_hint(), (3, Some(3)));
        assert_eq!(it.next().unwrap(), &6);
        assert_eq!(it.size_hint(), (2, Some(2)));
        assert_eq!(it.next_back().unwrap(), &4);
        assert_eq!(it.size_hint(), (1, Some(1)));
        assert_eq!(it.next_back().unwrap(), &5);
        assert_eq!(it.next_back(), None);
        assert_eq!(it.next(), None);
    }

    #[test]
    #[allow(clippy::manual_next_back)]
    fn test_rev_iter() {
        let m = generate_test();
        for (i, elt) in m.iter().rev().enumerate() {
            assert_eq!(6 - i as i32, *elt);
        }
        let mut n = LinkedList::new();
        assert_eq!(n.iter().rev().next(), None);
        n.push_front(4);
        let mut it = n.iter().rev();
        assert_eq!(it.size_hint(), (1, Some(1)));
        assert_eq!(it.next().unwrap(), &4);
        assert_eq!(it.size_hint(), (0, Some(0)));
        assert_eq!(it.next(), None);
    }

    #[test]
    fn test_mut_iter() {
        let mut m = generate_test();
        let mut len = m.len();
        for (i, elt) in m.iter_mut().enumerate() {
            assert_eq!(i as i32, *elt);
            len -= 1;
        }
        assert_eq!(len, 0);
        let mut n = LinkedList::new();
        assert!(n.iter_mut().next().is_none());
        n.push_front(4);
        n.push_back(5);
        let mut it = n.iter_mut();
        assert_eq!(it.size_hint(), (2, Some(2)));
        assert!(it.next().is_some());
        assert!(it.next().is_some());
        assert_eq!(it.size_hint(), (0, Some(0)));
        assert!(it.next().is_none());
    }

    #[test]
    fn test_iterator_mut_double_end() {
        let mut n = LinkedList::new();
        assert!(n.iter_mut().next_back().is_none());
        n.push_front(4);
        n.push_front(5);
        n.push_front(6);
        let mut it = n.iter_mut();
        assert_eq!(it.size_hint(), (3, Some(3)));
        assert_eq!(*it.next().unwrap(), 6);
        assert_eq!(it.size_hint(), (2, Some(2)));
        assert_eq!(*it.next_back().unwrap(), 4);
        assert_eq!(it.size_hint(), (1, Some(1)));
        assert_eq!(*it.next_back().unwrap(), 5);
        assert!(it.next_back().is_none());
        assert!(it.next().is_none());
    }

    #[test]
    fn test_eq() {
        let mut n: LinkedList<u8> = list_from(&[]);
        let mut m = list_from(&[]);
        assert!(n == m);
        n.push_front(1);
        assert!(n != m);
        m.push_back(1);
        assert!(n == m);

        let n = list_from(&[2, 3, 4]);
        let m = list_from(&[1, 2, 3]);
        assert!(n != m);
    }

    #[test]
    fn test_ord() {
        let n = list_from(&[]);
        let m = list_from(&[1, 2, 3]);
        assert!(n < m);
        assert!(m > n);
        assert!(n <= n);
        assert!(n >= n);
    }

    #[test]
    fn test_ord_nan() {
        let nan = 0.0f64 / 0.0;
        let n = list_from(&[nan]);
        let m = list_from(&[nan]);
        assert!(!(n < m));
        assert!(!(n > m));
        assert!(!(n <= m));
        assert!(!(n >= m));

        let n = list_from(&[nan]);
        let one = list_from(&[1.0f64]);
        assert!(!(n < one));
        assert!(!(n > one));
        assert!(!(n <= one));
        assert!(!(n >= one));

        let u = list_from(&[1.0f64, 2.0, nan]);
        let v = list_from(&[1.0f64, 2.0, 3.0]);
        assert!(!(u < v));
        assert!(!(u > v));
        assert!(!(u <= v));
        assert!(!(u >= v));

        let s = list_from(&[1.0f64, 2.0, 4.0, 2.0]);
        let t = list_from(&[1.0f64, 2.0, 3.0, 2.0]);
        assert!(!(s < t));
        assert!(s > one);
        assert!(!(s <= one));
        assert!(s >= one);
    }

    #[test]
    fn test_debug() {
        let list: LinkedList<i32> = (0..10).collect();
        assert_eq!(format!("{:?}", list), "[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]");

        let list: LinkedList<&str> = ["just", "one", "test", "more"].iter().copied().collect();
        assert_eq!(format!("{:?}", list), r#"["just", "one", "test", "more"]"#);
    }

    #[test]
    fn test_hashmap() {
        // Check that HashMap works with this as a key

        let list1: LinkedList<i32> = (0..10).collect();
        let list2: LinkedList<i32> = (1..11).collect();
        let mut map = std::collections::HashMap::new();

        assert_eq!(map.insert(list1.clone(), "list1"), None);
        assert_eq!(map.insert(list2.clone(), "list2"), None);

        assert_eq!(map.len(), 2);

        assert_eq!(map.get(&list1), Some(&"list1"));
        assert_eq!(map.get(&list2), Some(&"list2"));

        assert_eq!(map.remove(&list1), Some("list1"));
        assert_eq!(map.remove(&list2), Some("list2"));

        assert!(map.is_empty());
    }
}
