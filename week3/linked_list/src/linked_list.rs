use std::fmt::{self, Display};
use std::option::Option;

pub struct LinkedList<T> {
    head: Option<Box<Node<T>>>,
    size: usize,
}

struct Node<T>{
    value: T,
    next: Option<Box<Node<T>>>,
}

impl<T> Node<T> {
    pub fn new(value: T, next: Option<Box<Node<T>>>) -> Node<T> {
        Node {value: value, next: next}
    }
}

impl<T> LinkedList<T> {
    pub fn new() -> LinkedList<T> {
        LinkedList {head: None, size: 0}
    }
    
    pub fn get_size(&self) -> usize {
        self.size
    }
    
    pub fn is_empty(&self) -> bool {
        self.get_size() == 0
    }
    
    pub fn push_front(&mut self, value: T) {
        let new_node: Box<Node<T>> = Box::new(Node::new(value, self.head.take()));
        self.head = Some(new_node);
        self.size += 1;
    }
    
    pub fn pop_front(&mut self) -> Option<T> {
        let node: Box<Node<T>> = self.head.take()?;
        self.head = node.next;
        self.size -= 1;
        Some(node.value)
    }
}


impl<T: fmt::Display> fmt::Display for LinkedList<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut current: &Option<Box<Node<T>>> = &self.head;
        let mut result = String::new();
        loop {
            match current {
                Some(node) => {
                    result = format!("{} {}", result, node.value);
                    current = &node.next;
                },
                None => break,
            }
        }
        write!(f, "{}", result)
    }
}

impl<T> Drop for LinkedList<T> {
    fn drop(&mut self) {
        let mut current = self.head.take();
        while let Some(mut node) = current {
            current = node.next.take();
        }
    }
}

impl<T> Clone for Node<T>
    where T: Copy
{
    fn clone(&self) -> Self {
        Node { value: self.value, next: self.next.clone() } // will clone all
    }
}

impl<T> PartialEq for Node<T>
    where T: PartialEq
{
    fn eq(&self, other: &Self) -> bool {
        return self.value == other.value
    }
}

impl<T> Clone for LinkedList<T>
    where T:Copy
{
    fn clone(&self) -> Self {
        LinkedList {    
            head: self.head.clone(),
            size: self.size,
        }
    }
}

impl<T> PartialEq for LinkedList<T>
    where T: PartialEq
{
    fn eq(&self, other: &Self) -> bool {
        if self.get_size() != other.get_size(){
            return false;
        }
        let mut current = &self.head;
        let mut other_current = &other.head;
        while let Some(node) = current{
            if current != other_current{
                return false;
            }
            if let Some(other_node) = other_current{
                current = &node.next;
                other_current = &other_node.next;
            }
            else {
                return false;
            }
        }
        true
    }
}

