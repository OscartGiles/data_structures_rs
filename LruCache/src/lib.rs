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

pub struct LruCacheIter<'a, V> {
    current_idx: Option<usize>,
    buffer: &'a [Option<Node<V>>],
}

impl<'a, V> Iterator for LruCacheIter<'a, V> {
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

pub struct LruCache<V> {
    map: HashMap<String, Index>,
    buffer: Box<[Option<Node<V>>]>,
    head_index: usize,
    tail_index: usize,
    len: usize,
    capacity: usize,
    free: Vec<Index>,
}

impl<V> LruCache<V> {
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

    /// Create an iterator over values in the cache.
    pub fn iter(&self) -> LruCacheIter<'_, V> {
        LruCacheIter {
            current_idx: Some(self.head_index),
            buffer: &self.buffer,
        }
    }

    /// Replace the vaue at a specific buffer index.
    fn replace_at_index(&mut self, index: Index, value: V) {
        if let Some(node) = &mut self.buffer[index] {
            node.value = value
        }
    }

    // Create a new node to insert at the head of the list.
    fn new_head_node(&self, key: String, value: V) -> Node<V> {
        if self.len == 0 {
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
        }
    }

    /// Push a new value to the front of the list.
    /// If the list is already full the last item is popped from the back of the list to make space.
    fn push_front(&mut self, key: String, value: V) -> (Index, Option<String>) {
        let node = self.new_head_node(key, value);

        let old_key = if self.len < self.capacity {
            // The new head of the list is the next free space in the buffer (i.e. the current value of self.len)
            self.head_index = self.len;
            self.len += 1;
            self.buffer[self.head_index] = Some(node);
            None
        } else if let Some(idx) = self.free.pop() {
            // Otherwise check for any space that has been freed (i.e contained in the free list).
            self.head_index = idx;
            self.buffer[self.head_index] = Some(node);
            None
        } else {
            // If the list is already filled then swap the item at the tail for the new head.
            self.head_index = self.tail_index;

            // The new tail is the node before the old tail. Update this nodes `next` node to None (i.e. make it the tail node).
            let old_node = self.buffer[self.head_index]
                .as_ref()
                .expect("Tail node should be a valid node and not None.");

            if let Some(new_tail_node) = old_node.previous {
                self.tail_index = new_tail_node;

                let tail_node = self.buffer[self.tail_index]
                    .as_mut()
                    .expect("The tail nodes previous node should be a valid Node and not None.");

                tail_node.next = None;
            }

            let update_node = &mut self.buffer[self.head_index];
            let old_node = update_node.replace(node);
            old_node.map(|node| node.key)
        };

        // Get second list item and update its previous node to the new head node.
        let head_node = &self.buffer[self.head_index]
            .as_mut()
            .expect("The head node should be a valid node and not None.");

        if let Some(next_node_index) = head_node.next {
            let next_node = &mut self.buffer[next_node_index]
                .as_mut()
                .expect("The next node should be a valid Node and not None.");

            next_node.previous = Some(self.head_index)
        }

        (self.head_index, old_key)
    }

    pub fn set(&mut self, key: impl Into<String>, value: V) {
        let key = key.into();
        match !self.map.contains_key(&key) {
            true => {
                let (new_index, removed_key) = self.push_front(key.clone(), value);
                self.map.insert(key, new_index);
                if let Some(old_key) = removed_key {
                    self.map.remove(&old_key);
                }
            }
            false => {
                let existing_index = *self.map.get(&key).expect("The expected node was not found");
                self.replace_at_index(existing_index, value);
            }
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
        let index = self.map.get(key).copied();

        match index {
            Some(index) => {
                let (key, value) = self.remove(index).unwrap();
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
    use crate::LruCache;

    #[test]
    fn get_makes_most_recent() {
        let mut cache = LruCache::new(3);

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
        let mut cache = LruCache::new(3);

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
        let mut cache = LruCache::new(1000);

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
