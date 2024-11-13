/// A prefix tree whose nodes are backed by a sorted array.
///
/// # Worst-Case Runtime Complexity
/// - Lookup: `log_2(256) * m = O(m)` where `m` is the length in bytes of the prefix to match.
/// - Insertion: `(log_2(256) + 256) * m = O(m)` where `m` is the length in bytes of the prefix to
///   insert.
///
/// So, this implementation is memory-effecient, offering fast lookups but slower insertions,
/// making it well suited for cases where insertions occur only during initialization.
pub struct PrefixTree<V> { table: Table<V>, value: Option<V> }

struct Table<V> { table: Vec<TableEntry<V>> }

struct TableEntry<V> { key: u8, value: PrefixTree<V> }

impl<V> Default for PrefixTree<V> {
    fn default() -> Self {
        Self { table: Table::default(), value: None }
    }
}

impl<V> Default for Table<V> {
    fn default() -> Self {
        Self { table: Vec::default() }
    }
}

impl<V> Table<V> {
    fn get(&self, key: u8) -> Option<&PrefixTree<V>> {
        let idx = self.table.binary_search_by_key(&key, |entry| entry.key).ok()?;
        return Some(&self.table[idx].value);
    }

    fn entry(&mut self, key: u8) -> &mut PrefixTree<V> {
        let result = self.table.binary_search_by_key(&key, |entry| entry.key);
        match result {
            Ok(idx) => {
                return &mut self.table[idx].value;    
            },
            Err(idx) => {
                let new_node: PrefixTree<V> = PrefixTree::default();
                let new_entry: TableEntry<V> = TableEntry { key, value: new_node };
                self.table.insert(idx, new_entry);
                return &mut self.table[idx].value;
            },
        }        
    }
}

impl<V> PrefixTree<V> {
    pub fn get(&self, mut seq: impl Iterator<Item = u8>) -> Option<&V> {
        if let Some(next_key) = seq.next() {
            if let Some(next_child) = self.table.get(next_key) {
                if let Some(longer_match) = next_child.get(seq) {
                    return Some(longer_match);
                }
            }
        }
        return self.value.as_ref();
    }
    
    pub fn insert_iter(&mut self, mut iter: impl Iterator<Item = u8>, value: V) -> Option<V> {
        match iter.next() {
            Some(key) => {
                let child_node = self.table.entry(key);
                return child_node.insert_iter(iter, value);
            },
            None => {
                let prev = self.value.take();
                self.value = Some(value);
                return prev;
            },
        }
    }

    pub fn insert_seq(&mut self, seq: &[u8], value: V) -> Option<V> {
        return self.insert_iter(seq.iter().copied(), value);
    }
}
