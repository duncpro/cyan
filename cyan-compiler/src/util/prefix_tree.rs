pub struct PrefixTree<K, V> {
    map: std::collections::HashMap<K, PrefixTree<K, V>>,
    value: Option<V>
}

impl<K, V> Default for PrefixTree<K, V> {
    fn default() -> Self {
        Self {
            map: std::collections::HashMap::new(),
            value: None 
        }
    }
}

impl<K: std::hash::Hash + Eq, V> PrefixTree<K, V> {
    pub fn insert_iter(&mut self, mut iter: impl Iterator<Item = K>, value: V) -> Option<V> {
        match iter.next() {
            Some(key) => {
                let child_node = self.map.entry(key).or_default();
                return child_node.insert_iter(iter, value);
            },
            None => {
                let prev = self.value.take();
                self.value = Some(value);
                return prev;
            },
        }
    }
}

impl<'a, K: std::hash::Hash + Eq + Clone + 'a, V> PrefixTree<K, V> {
    pub fn insert_seq(&mut self, seq: impl IntoIterator<Item = &'a K>, value: V) -> Option<V> {
        return self.insert_iter(seq.into_iter().cloned(), value);
    }

    pub fn get(&self, mut seq: impl Iterator<Item = &'a K>) -> Option<&V> {
        if let Some(next_key) = seq.next() {
            if let Some(next_child) = self.map.get(next_key) {
                if let Some(longer_match) = next_child.get(seq) {
                    return Some(longer_match);
                }
            }
        }
        return self.value.as_ref();
    }
}

