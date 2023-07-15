use std::cell::RefCell;
use std::iter::FusedIterator;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;

/// An implementation of a doubly linked-list. Not thread-safe. Note that the data items contained
/// within nodes cannot be changed after they have been added to the linked-list.
pub struct LinkedList<T> {
    head_and_tail: Option<[Link<T>; 2]>,
    len: usize,
}

#[derive(Clone, Copy)]
enum Where {
    Head,
    Tail,
}

use Where::*;

impl From<Where> for usize {
    fn from(w: Where) -> Self {
        match w {
            Head => 0,
            Tail => 1,
        }
    }
}

impl<T> LinkedList<T> {
    /// Creates an empty LinkedList.
    pub fn new() -> LinkedList<T> {
        LinkedList {
            head_and_tail: None,
            len: 0,
        }
    }

    fn push_helper(&mut self, data: T, where_to_update: Where) {
        #[allow(clippy::type_complexity)]
        let (setter1, setter2): (
            fn(&mut Link<T>, other: Option<Link<T>>),
            fn(&mut Link<T>, other: Option<Link<T>>),
        ) = match where_to_update {
            Head => (set_prev, set_next),
            Tail => (set_next, set_prev),
        };

        let mut new_node = Node::new_link(data);
        self.len += 1;

        match self.head_and_tail.as_mut() {
            None => self.head_and_tail = Some([new_node.clone(), new_node]),
            Some(head_and_tail) => {
                let update_index: usize = where_to_update.into();
                setter1(&mut head_and_tail[update_index], Some(new_node.clone()));
                setter2(&mut new_node, Some(head_and_tail[update_index].clone()));
                head_and_tail[update_index] = new_node;
            }
        }
    }

    /// Pushes the data item to the end of the LinkedList.
    pub fn push(&mut self, data: T) {
        self.push_helper(data, Tail)
    }

    /// Pushes the data item to the front of the LinkedList.
    pub fn push_front(&mut self, data: T) {
        self.push_helper(data, Head)
    }

    fn pop_helper(&mut self, where_to_update: Where) -> Option<Rc<T>> {
        #[allow(clippy::type_complexity)]
        let (getter, setter): (
            fn(&Node<T>) -> Option<Link<T>>,
            fn(&mut Link<T>, other: Option<Link<T>>),
        ) = match where_to_update {
            Head => (Node::<T>::get_next, set_prev),
            Tail => (Node::<T>::get_prev, set_next),
        };
        let mut need_to_zero_head_and_tail = false;
        let popped = self.head_and_tail.as_mut().map(|head_and_tail| {
            let update_index: usize = where_to_update.into();
            let old = head_and_tail[update_index].clone();
            let old = old.borrow();
            match getter(old.deref()) {
                None => need_to_zero_head_and_tail = true,
                Some(link) => {
                    head_and_tail[update_index] = link;
                    setter(&mut head_and_tail[update_index], None);
                }
            }
            let old_data = old.get_data();
            self.len -= 1;
            old_data
        });
        if need_to_zero_head_and_tail {
            self.head_and_tail = None;
        }
        popped
    }

    /// Removes the last node from the LinkedList. Returns Some containing the value from the
    /// removed node, otherwise None.
    pub fn pop(&mut self) -> Option<Rc<T>> {
        self.pop_helper(Tail)
    }

    /// Removes the first node from the LinkedList. Returns Some containing the value from the
    /// removed node, otherwise None.
    pub fn pop_front(&mut self) -> Option<Rc<T>> {
        self.pop_helper(Head)
    }

    /// Returns the number of items contained in the LinkedList.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Checks if the LinkedList is empty.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Creates an iterator over the LinkedList.
    pub fn iter(&self) -> LinkedListIter<T> {
        LinkedListIter::new(
            self.head_and_tail
                .as_ref()
                .map(|head_and_tail| head_and_tail[0].clone()),
        )
    }
}

impl<T> Default for LinkedList<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> IntoIterator for LinkedList<T> {
    type Item = Rc<T>;

    type IntoIter = LinkedListIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// Represents a link from one node to another before or after it.
type Link<T> = Rc<RefCell<Box<Node<T>>>>;

/// Updates the previous node.
fn set_helper<T>(
    s: &mut Link<T>,
    other: Option<Link<T>>,
    setter: fn(&mut Node<T>, other: Option<Link<T>>),
) {
    setter(s.borrow_mut().deref_mut(), other)
}

/// Updates the previous node.
fn set_prev<T>(s: &mut Link<T>, other: Option<Link<T>>) {
    set_helper(s, other, Node::<T>::set_prev)
}

/// Updates the next node.
fn set_next<T>(s: &mut Link<T>, other: Option<Link<T>>) {
    set_helper(s, other, Node::<T>::set_next)
}

/// A node containing a data item and links to previous and next nodes.
struct Node<T> {
    data: Rc<T>,
    prev: Option<Link<T>>,
    next: Option<Link<T>>,
}

impl<T> Node<T> {
    /// Creates a new Node containing the given data item. The previous and next node links are set
    /// to None.
    fn new(data: T) -> Node<T> {
        Node {
            data: Rc::new(data),
            prev: None,
            next: None,
        }
    }

    /// Updates the previous node.
    fn set_prev(&mut self, other: Option<Link<T>>) {
        self.prev = other;
    }

    /// Updates the next node.
    fn set_next(&mut self, other: Option<Link<T>>) {
        self.next = other;
    }

    /// Gets the previous link from the Node via cloning.
    fn get_prev(&self) -> Option<Link<T>> {
        self.prev.clone()
    }

    /// Gets the next link from the Node via cloning.
    fn get_next(&self) -> Option<Link<T>> {
        self.next.clone()
    }

    /// Gets the data item contained within the Node via cloning.
    fn get_data(&self) -> Rc<T> {
        self.data.clone()
    }

    /// Creates a new Link containing the given data item.
    fn new_link(data: T) -> Link<T> {
        Rc::new(RefCell::new(Box::new(Node::new(data))))
    }
}

/// Wrapper struct for LinkedList to implement the Iterator trait. Yields cloned values contained in
/// the nodes of the LinkedList.
pub struct LinkedListIter<T> {
    cursor: Option<Link<T>>,
}

impl<T> LinkedListIter<T> {
    fn new(cursor: Option<Link<T>>) -> LinkedListIter<T> {
        LinkedListIter { cursor }
    }
}

impl<T> Iterator for LinkedListIter<T> {
    type Item = Rc<T>;

    fn next(&mut self) -> Option<Self::Item> {
        let next_node;
        let yield_data;
        match self.cursor.as_ref() {
            None => return None,
            Some(cursor) => {
                let cursor = cursor.borrow();
                yield_data = cursor.get_data();
                next_node = cursor.get_next();
            }
        }
        self.cursor = next_node;
        Some(yield_data)
    }
}

impl<T> FusedIterator for LinkedListIter<T> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_and_pop() {
        let mut new_list = LinkedList::<i32>::new();
        new_list.push(1);
        new_list.pop();
        assert_eq!(new_list.len(), 0);
    }

    #[test]
    fn test_push_back_length() {
        let mut new_list = LinkedList::<i32>::new();
        let values = (0..10).collect::<Vec<i32>>();
        for i in values {
            new_list.push(i);
        }
        assert_eq!(new_list.len(), 10);
    }

    #[test]
    fn test_push_front_length() {
        let mut new_list = LinkedList::<i32>::new();
        let values = (0..10).collect::<Vec<i32>>();
        for i in values {
            new_list.push_front(i)
        }
        assert_eq!(new_list.len(), 10);
    }

    #[test]
    fn test_push_back_values() {
        let mut new_list = LinkedList::<i32>::new();
        let values = (0..10).collect::<Vec<i32>>();
        for &i in values.iter() {
            new_list.push(i);
        }
        let values_from_list = new_list
            .iter()
            .map(|val| *val.as_ref())
            .collect::<Vec<i32>>();
        assert_eq!(values, values_from_list);
    }

    #[test]
    fn test_push_front_values() {
        let mut new_list = LinkedList::<i32>::new();
        let values = (0..10).collect::<Vec<i32>>();
        for &i in values.iter() {
            new_list.push_front(i)
        }
        let values_from_list = new_list
            .iter()
            .map(|val| *val.as_ref())
            .collect::<Vec<i32>>();
        let values = values.iter().rev().copied().collect::<Vec<i32>>();
        assert_eq!(values, values_from_list);
    }

    #[test]
    fn test_empty_list_length() {
        let new_list = LinkedList::<i32>::new();
        assert_eq!(new_list.len(), 0);
    }

    #[test]
    fn test_list_length_single() {
        let mut new_list = LinkedList::<i32>::new();
        new_list.push(1);
        assert_eq!(new_list.len(), 1);
    }

    #[test]
    fn test_list_str_push_back() {
        let mut new_list = LinkedList::<&str>::new();
        let strings = ["10", "20", "30", "40", "50"].to_vec();
        for s in &strings {
            new_list.push(s);
        }
        let strings_from_list = new_list
            .iter()
            .map(|val| *val.as_ref())
            .collect::<Vec<&str>>();
        assert_eq!(strings, strings_from_list);
    }

    #[test]
    fn test_iter_values() {
        let mut new_list = LinkedList::<i32>::new();
        let values = (0..10).collect::<Vec<i32>>();
        for &i in values.iter() {
            new_list.push(i);
        }
        let mut values_from_list: Vec<i32> = vec![];
        for i in new_list {
            values_from_list.push(*i);
        }
        assert_eq!(values, values_from_list);
    }

    #[test]
    fn test_big_push() {
        let mut new_list = LinkedList::<i32>::new();
        for i in 0..100000 {
            new_list.push(i);
        }
        assert_eq!(new_list.len(), 100000);
    }

    #[test]
    fn test_bigger_push() {
        let mut new_list = LinkedList::<i32>::new();
        for i in 0..10000000 {
            new_list.push(i);
        }
        assert_eq!(new_list.len(), 10000000);
    }

    #[test]
    fn test_array_push() {
        let mut new_list = LinkedList::<[i32; 3]>::new();
        let arrays: Vec<[i32; 3]> = vec![[1, 2, 3], [4, 5, 6], [7, 8, 9]];
        for &a in arrays.iter() {
            new_list.push(a);
        }
        let arrays_from_list = new_list
            .iter()
            .map(|a| *a.as_ref())
            .collect::<Vec<[i32; 3]>>();
        assert_eq!(arrays, arrays_from_list);
    }
}
