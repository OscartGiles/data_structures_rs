use std::fmt::Debug;

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

    fn fix_height(&mut self) {
        let left_height = Node::height(&self.left);
        let right_height = Node::height(&self.right);
        self.height = 1 + left_height.max(right_height);
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

    /// Balance the tree using AVL rotations
    ///
    /// Algorithm:
    /// 1. First calculate the height of the node.
    /// 2. Calculate the balance factor balance_factor = (height(node.left) - height(node.right))
    /// 3. If balance_factor > 1 this is a left case
    ///     3.i) Compare the heights of the left node and the right node.
    ///         - If height(left) > height(right) this is a left-left case.
    ///         - otherwise this is left-right case.
    ///     3.ii) if left-left case:
    ///             - rotate_right(node)
    ///           else:
    ///             1. rotate_left(node.left) (rotate the left node left)
    ///             2. rotate_right(node)
    /// 4. If balance_factor < 1 this is right case. Do opposite of left case.
    fn balance(mut self: NodeRef<V>) -> NodeRef<V> {
        // 1. Calculate heights
        self.fix_height();

        // 2. Calculate balance factor
        let balance_factor = Node::height(&self.left) as i32 - Node::height(&self.right) as i32;

        // 3. Rebalance if required
        if balance_factor > 1 {
            let l = self.left.as_ref().map(|n| Node::height(&n.left));
            let r = self.left.as_ref().map(|n| Node::height(&n.right));

            if l > r {
                // Left left case
                self.rotate_right()
            } else {
                // Left right case
                self.left = Some(self.left.expect("Should have left node").rotate_left());
                self.rotate_right()
            }
        } else if balance_factor < -1 {
            let l = self.right.as_ref().map(|n| Node::height(&n.left));
            let r = self.right.as_ref().map(|n| Node::height(&n.right));
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
pub struct Tree<V> {
    root: Option<NodeRef<V>>,
}

impl<V: std::cmp::PartialOrd> Default for Tree<V> {
    fn default() -> Self {
        Tree::new()
    }
}

impl<V: std::cmp::PartialOrd> Tree<V> {
    pub fn new() -> Tree<V> {
        Tree { root: None }
    }

    pub fn insert(&mut self, value: V) {
        if let Some(root) = self.root.take() {
            self.root = Some(root.insert(value));
        } else {
            self.root = Some(Box::new(Node::new(value)))
        }
    }

    pub fn iter(&self) -> AvlTreeIter<V> {
        AvlTreeIter::new(&self.root)
    }
}

pub struct AvlTreeIter<'a, V> {
    stack: Vec<&'a NodeRef<V>>,
}

impl<'a, V> AvlTreeIter<'a, V> {
    fn new(root: &Option<NodeRef<V>>) -> AvlTreeIter<'_, V> {
        let mut avl_iter = AvlTreeIter { stack: vec![] };
        avl_iter.push_left_branch(root);
        avl_iter
    }

    fn push_left_branch(&mut self, mut node: &'a Option<NodeRef<V>>) {
        while let Some(ref n) = node {
            self.stack.push(n);
            node = &n.left;
        }
    }
}

impl<'a, V> Iterator for AvlTreeIter<'a, V> {
    type Item = &'a V;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(next) = self.stack.pop() {
            let value = &next.value;

            if next.right.is_some() {
                self.push_left_branch(&next.right);
            }
            Some(value)
        } else {
            None
        }
    }
}
#[cfg(test)]
mod tests {
    use super::Tree;

    #[test]
    fn create_nodes() {
        let mut tree = Tree::<i32>::new();
        tree.insert(6);
        tree.insert(4);
        tree.insert(3);
        tree.insert(5);
        tree.insert(-100);

        for v in tree.iter() {
            println!("{}", v);
        }
    }
}
