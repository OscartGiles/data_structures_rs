use std::{f64::consts::PI, fmt::Debug};

type NodeRef<V> = Box<Node<V>>;

#[derive(Debug)]
struct Node<V> {
    left: Option<NodeRef<V>>,
    right: Option<NodeRef<V>>,
    height: usize,
    value: V,
}

impl<V: std::cmp::PartialOrd> Node<V> {
    pub fn new(value: V) -> Node<V> {
        Node {
            left: None,
            right: None,
            height: 1,
            value,
        }
    }

    fn height(node: &Option<NodeRef<V>>) -> usize {
        node.as_ref().map_or(0, |n| n.height)
    }
    fn right_height(&self) -> usize {
        self.right.as_ref().map_or(0, |n| n.height)
    }
}

impl<V: std::cmp::PartialOrd> Node<V> {
    fn rotate_right(mut self: NodeRef<V>) -> NodeRef<V> {
        let mut left_child = self.left.take().unwrap();
        let left_child_right = left_child.right.take();

        self.fix_height();
        left_child.right = Some(self);

        if let Some(node) = &mut left_child.right {
            node.left = left_child_right;
        }
        left_child.fix_height();
        left_child
    }

    fn rotate_left(mut self: NodeRef<V>) -> NodeRef<V> {
        let mut right_child = self.right.take().unwrap();
        let right_child_left = right_child.left.take();

        self.fix_height();
        right_child.left = Some(self);

        if let Some(node) = &mut right_child.left {
            node.right = right_child_left;
        }
        right_child.fix_height();
        right_child
    }

    fn fix_height(&mut self) {
        let left_height = Node::height(&self.left);
        let right_height = Node::height(&self.right);
        self.height = 1 + left_height.max(right_height);
    }

    fn balance(mut self: NodeRef<V>) -> NodeRef<V> {
        // 1. Calculate heights
        self.fix_height();

        // 2. Calculate balance factor
        let balance_factor = Node::height(&self.left) as i32 - Node::height(&self.right) as i32;
        println!("Balance = {}", balance_factor);

        // 3. Rebalance if required
        if balance_factor > 1 {
            let l = self.left.as_ref().map_or(0, |n| Node::height(&n.left));
            let r = self.left.as_ref().map_or(0, |n| Node::height(&n.right));

            if l > r {
                // Left left case
                self.rotate_right()
            } else {
                // Left right case
                self.left = Some(self.left.expect("Should have left node").rotate_left());
                self.rotate_right()
            }
        } else if balance_factor < -1 {
            let l = self.right.as_ref().map_or(0, |n| Node::height(&n.left));
            let r = self.right.as_ref().map_or(0, |n| Node::height(&n.right));
            if r > l {
                // Right right case
                self.rotate_left()
            } else {
                // Right left case
                self.right = Some(self.right.expect("Should have right node").rotate_right());
                self.rotate_left()
            }
        } else {
            self
        }
    }

    fn insert(mut self: NodeRef<V>, value: V) -> NodeRef<V> {
        if value < self.value {
            if let Some(left) = self.left {
                self.left = Some(left.insert(value));
            } else {
                self.left = Some(Box::new(Node::new(value)))
            }
        } else if value > self.value {
            if let Some(right) = self.right {
                self.right = Some(right.insert(value));
            } else {
                self.right = Some(Box::new(Node::new(value)))
            }
        } else {
            self.value = value;
        }
        self.balance()
    }
}

#[derive(Debug)]
struct Tree<V> {
    root: Option<NodeRef<V>>,
}

impl<V: std::cmp::PartialOrd> Tree<V> {
    fn new() -> Tree<V> {
        Tree { root: None }
    }

    fn insert(&mut self, value: V) {
        if let Some(root) = self.root.take() {
            self.root = Some(root.insert(value));
        } else {
            self.root = Some(Box::new(Node::new(value)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Tree;

    #[test]
    fn create_nodes() {
        let mut tree = Tree::<i32>::new();
        tree.insert(1);
        tree.insert(3);
        tree.insert(2);

        println!("{:#?}", tree);
    }
}
