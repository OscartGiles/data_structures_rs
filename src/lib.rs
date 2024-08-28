use std::collections::HashMap;

type Index = usize;

#[allow(unused)]
struct Node<V> {
    previous: Option<Index>,
    next: Option<Index>,
    key: String,
    value: V,
}

impl<V> std::fmt::Debug for Node<V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Node")
            .field("key", &self.key)
            .field("previous", &self.previous)
            .field("next", &self.next)
            .finish()
    }
}

pub struct LRUCacheIter<'a, V> {
    current_idx: Option<usize>,
    buffer: &'a [Option<Node<V>>],
}

impl<'a, V> Iterator for LRUCacheIter<'a, V> {
    type Item = &'a V;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(index) = self.current_idx {
            if let Some(current_node) = self.buffer[index].as_ref() {
                let value = Some(&current_node.value);
                self.current_idx = current_node.next;
                value
            } else {
                // Catch the case where the Cache is empty
                None
            }
        } else {
            None
        }
    }
}

pub struct LRUCache<V> {
    map: HashMap<String, Index>,
    buffer: Box<[Option<Node<V>>]>,
    head_index: usize,
    tail_index: usize,
    len: usize,
    capacity: usize,
    free: Vec<Index>,
}

impl<V> LRUCache<V> {
    pub fn new(size: usize) -> Self {
        let mut buf: Vec<Option<Node<V>>> = Vec::with_capacity(size);

        // ToDo: Figure out how to init this faster!
        for _ in 0..size {
            buf.push(None);
        }

        Self {
            map: HashMap::new(),
            buffer: buf.into_boxed_slice(),
            head_index: 0,
            tail_index: 0,
            len: 0,
            capacity: size,
            free: Vec::new(),
        }
    }

    pub fn iter(&self) -> LRUCacheIter<'_, V> {
        LRUCacheIter {
            current_idx: Some(self.head_index),
            buffer: &self.buffer,
        }
    }

    fn replace_at_index(&mut self, index: Index, value: V) {
        if let Some(node) = &mut self.buffer[index] {
            node.value = value
        }
    }

    /// Push a new value to the front of the list.
    /// If the list is already full the last item is popped from the back of the list to make space.
    fn push_front(&mut self, key: String, value: V) -> (Index, Option<String>) {
        let node = if self.len == 0 {
            Node {
                previous: None,
                next: None,
                key,
                value,
            }
        } else {
            Node {
                previous: None,
                next: Some(self.head_index),
                key,
                value,
            }
        };

        let old_key = if self.len < self.capacity {
            // If the list has not been filled then keep track of space to the right.
            // The new head of the list is the next free space to the right (i.e. the current value of self.len)
            self.head_index = self.len;
            self.len += 1;
            self.buffer[self.head_index] = Some(node);
            None
        } else if let Some(idx) = self.free.pop() {
            self.head_index = idx;
            self.buffer[self.head_index] = Some(node);
            None
        } else {
            // If the list is already filled then swap the item at the tail for the new head.
            self.head_index = self.tail_index;

            // The new tail is the node before the old tail. Update this nodes `next` node to None (i.e. make it the tail node).
            if let Some(node) = &mut self.buffer[self.head_index] {
                if let Some(new_tail_node) = node.previous {
                    self.tail_index = new_tail_node;
                    if let Some(tail_node) = &mut self.buffer[self.tail_index] {
                        tail_node.next = None;
                    }
                }
            }

            let update_node = &mut self.buffer[self.head_index];
            let old_node = update_node.replace(node);
            old_node.map(|node| node.key)
        };

        // Get second list item and update its previous node to the new head node.
        if let Some(node) = &self.buffer[self.head_index] {
            if let Some(second_node_index) = node.next {
                if let Some(second_node) = &mut self.buffer[second_node_index] {
                    second_node.previous = Some(self.head_index)
                }
            }
        }
        (self.head_index, old_key)
    }

    pub fn set(&mut self, key: impl Into<String>, value: V) {
        let key = key.into();
        if !self.map.contains_key(&key) {
            let (new_index, removed_key) = self.push_front(key.clone(), value);
            self.map.insert(key, new_index);
            if let Some(old_key) = removed_key {
                self.map.remove(&old_key);
            }
        } else {
            let existing_index = *self.map.get(&key).expect("The expected node was not found");
            self.replace_at_index(existing_index, value);
        }
    }

    fn get_node_mut(&mut self, index: Option<Index>) -> Option<&mut Node<V>> {
        match index {
            Some(idx) => self.buffer[idx].as_mut(),
            None => None,
        }
    }

    fn get_node(&self, index: Option<Index>) -> Option<&Node<V>> {
        match index {
            Some(idx) => self.buffer[idx].as_ref(),
            None => None,
        }
    }

    /// Remove the item at the index returning the key and value
    fn remove(&mut self, index: Index) -> Option<(String, V)> {
        let remove_node = self.buffer[index].take();

        // Track index as free
        self.free.push(index);

        if let Some(ref node) = remove_node {
            match (self.get_node(node.previous), self.get_node(node.next)) {
                (None, None) => {}
                (None, Some(_)) => {
                    let next = self.get_node_mut(node.next).unwrap();
                    next.previous = None
                }
                (Some(_), None) => {
                    let prev = self.get_node_mut(node.previous).unwrap();
                    prev.next = None
                }
                (Some(_), Some(_)) => {
                    let prev = self.get_node_mut(node.previous).unwrap();
                    prev.next = node.next;

                    {
                        let next = self.get_node_mut(node.next).unwrap();
                        next.previous = node.previous
                    }
                }
            }
        }

        remove_node.map(|node| (node.key, node.value))
    }

    pub fn get(&mut self, key: &str) -> Option<&V> {
        println!("{:?}", self.map);
        let index = self.map.get(key).copied();

        match index {
            Some(index) => {
                let (key, value) = self.remove(index).unwrap();
                println!("{}", key);
                let (new_index, _old_key) = self.push_front(key.clone(), value);
                let _ = self.map.insert(key, new_index);
                let v = self.buffer[new_index].as_ref().map(|node| &node.value);
                v
            }
            None => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::LRUCache;

    #[test]
    fn get_makes_most_recent() {
        let mut cache = LRUCache::new(3);

        cache.set("a", 1);
        cache.set("b", 2);
        cache.set("c", 3);
        assert_eq!(cache.get("a"), Some(&1));
        assert_eq!(cache.get("b"), Some(&2));
        assert_eq!(cache.get("c"), Some(&3));

        cache.set("a", 4);
        assert_eq!(cache.get("a"), Some(&4));
        assert_eq!(cache.get("b"), Some(&2));
        assert_eq!(cache.get("c"), Some(&3));

        cache.set("b", 5);
        assert_eq!(cache.get("a"), Some(&4));
        assert_eq!(cache.get("b"), Some(&5));
        assert_eq!(cache.get("c"), Some(&3));

        cache.set("c", 6);
        assert_eq!(cache.get("a"), Some(&4));
        assert_eq!(cache.get("b"), Some(&5));
        assert_eq!(cache.get("c"), Some(&6));
    }

    #[test]
    fn test_set_get() {
        let mut cache = LRUCache::new(3);

        cache.set("a", 1);
        cache.set("b", 2);
        cache.set("c", 3);

        assert_eq!(cache.get("a"), Some(&1));
        assert_eq!(cache.get("b"), Some(&2));
        assert_eq!(cache.get("c"), Some(&3));

        cache.set("d", 4);

        assert_eq!(cache.get("a"), None);
        assert_eq!(cache.get("b"), Some(&2));
        assert_eq!(cache.get("c"), Some(&3));
        assert_eq!(cache.get("d"), Some(&4));
    }

    #[test]
    fn iter_by_recency() {
        let mut cache = LRUCache::new(1000);

        cache.set("a", 1);
        cache.set("b", 2);
        cache.set("c", 3);
        cache.set("d", 4);
        cache.set("e", 5);
        cache.set("f", 6);
        cache.set("g", 7);

        cache.get("a");

        for node in cache.iter() {
            println!("{:?}", node);
        }
    }
}
