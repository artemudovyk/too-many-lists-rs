use std::rc::Rc;

// A persistent singly-linked stack
pub struct List<T> {
    head: Link<T>,
}

type Link<T> = Option<Rc<Node<T>>>;

struct Node<T> {
    elem: T,
    prev: Link<T>,
}

impl<T> List<T> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn prepend(&self, elem: T) -> List<T> {
        List {
            head: Some(Rc::new(Node {
                elem,
                prev: self.head.clone(),
            })),
        }
    }

    pub fn tail(&self) -> List<T> {
        List {
            head: self.head.as_ref().and_then(|v| v.prev.clone()),
        }
    }

    pub fn head(&self) -> Option<&T> {
        self.head.as_ref().map(|v| &v.elem)
    }
}

impl<T> Default for List<T> {
    fn default() -> Self {
        Self { head: None }
    }
}

impl<T> std::fmt::Display for List<T>
where
    T: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut cursor = self.head.as_ref();
        while let Some(node) = cursor {
            write!(f, "{:?} -> ", node.elem)?;
            cursor = node.prev.as_ref();
        }
        write!(f, "$")
    }
}

pub struct Iter<'a, T> {
    cursor: Option<&'a Node<T>>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        self.cursor.map(|node| {
            self.cursor = node.prev.as_deref();
            &node.elem
        })
    }
}

// Drop long lists iteratively instead of recursively.
impl<T> Drop for List<T> {
    fn drop(&mut self) {
        let mut cursor = self.head.take();
        while let Some(node) = cursor {
            // If we are the sole owner of this Rc, we can hoist this node out of it and drop it.
            // If not, then there are other owners and we must stop dropping.
            if let Ok(node) = Rc::try_unwrap(node) {
                cursor = node.prev;
            } else {
                break;
            }
        }
    }
}

impl<T> List<T> {
    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            cursor: self.head.as_deref(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basics() {
        let list = List::new();
        assert_eq!(list.head(), None);

        let list = list.prepend(1).prepend(2).prepend(3);
        assert_eq!(list.head(), Some(&3));

        let list = list.tail();
        assert_eq!(list.head(), Some(&2));

        let list = list.tail();
        assert_eq!(list.head(), Some(&1));

        let list = list.tail();
        assert_eq!(list.head(), None);

        // Make sure empty tail works
        let list = list.tail();
        assert_eq!(list.head(), None);
    }

    #[test]
    fn prepend_returns_new_list_with_element_as_last_node() {
        let empty_list: List<i32> = List::new();
        let list_with_1 = empty_list.prepend(1);
        let list_with_2 = empty_list.prepend(2);
        let list_with_3_and_1 = list_with_1.prepend(3);

        assert_eq!(format!("{}", empty_list), "$");
        assert_eq!(format!("{}", list_with_1), "1 -> $");
        assert_eq!(format!("{}", list_with_2), "2 -> $");
        assert_eq!(format!("{}", list_with_3_and_1), "3 -> 1 -> $");
    }

    #[test]
    fn tail_returns_list_without_its_head() {
        let list: List<i32> = List::new();
        let list = list.prepend(1);
        let list = list.prepend(2);
        let list = list.prepend(3);
        assert_eq!(format!("{}", list), "3 -> 2 -> 1 -> $");

        let tail = list.tail();
        assert_eq!(format!("{}", tail), "2 -> 1 -> $");
    }

    #[test]
    fn head_returns_reference_to_first_elem() {
        let list: List<i32> = List::new();
        let list = list.prepend(1);
        let list = list.prepend(2);
        assert_eq!(list.head(), Some(&2));
    }

    #[test]
    fn iter() {
        let list = List::new().prepend(1).prepend(2).prepend(3);

        let mut iter = list.iter();
        assert_eq!(iter.next(), Some(&3));
        assert_eq!(iter.next(), Some(&2));
        assert_eq!(iter.next(), Some(&1));
    }
}
