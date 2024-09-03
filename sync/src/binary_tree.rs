use std::{cell::RefCell, rc::Rc};

type NodeRef<V> = Box<RefCell<Node<V>>>;

#[derive(Debug)]
struct Node<V> {
    left: Option<NodeRef<V>>,
    right: Option<NodeRef<V>>,
    value: V,
}

impl<V: std::cmp::PartialOrd> Node<V> {
    pub fn new(value: V) -> Node<V> {
        Self {
            left: None,
            right: None,
            value,
        }
    }

    fn insert(&mut self, value: V) {
        if value < self.value {
            match &self.left {
                Some(left_node) => left_node.borrow_mut().insert(value),
                None => self.left = Some(Box::new(RefCell::new(Node::new(value)))),
            }
        } else if value > self.value {
            match &self.right {
                Some(right_node) => right_node.borrow_mut().insert(value),
                None => self.right = Some(Box::new(RefCell::new(Node::new(value)))),
            }
        } else {
            println!("Same value... Ignoring")
        }
    }
}

#[derive(Debug)]
struct Tree<V> {
    root: Option<Node<V>>,
}

impl<V: std::cmp::PartialOrd> Tree<V> {
    fn new() -> Tree<V> {
        Tree { root: None }
    }

    fn insert(&mut self, value: V) {
        match &mut self.root {
            Some(root) => root.insert(value),
            None => self.root = Some(Node::new(value)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Node, Tree};

    #[test]
    fn create_nodes() {
        let mut tree = Tree::<i32>::new();
        tree.insert(1);
        tree.insert(2);
        tree.insert(3);
        println!("{:#?}", tree);
    }
}
