use std::hash::Hash;

use indexmap::IndexMap;

pub struct MultiMap<K : Hash + Eq, V> {
    map: IndexMap<K, Vec<V>>
}

impl<K : Hash + Eq,V> MultiMap<K,V> {
    pub fn new() -> Self {
        let map = IndexMap::new();
        Self { map }
    }

    pub fn insert(&mut self, key: K, value: V) {
        let values = self.map.entry(key).or_insert(Vec::new());
        values.push(value);
    }

    pub fn insert_all(&mut self, key: K, values: Vec<V>) {
        if self.map.contains_key(&key) {
            self.map.get_mut(&key).unwrap().extend(values);
        }
        else{
            self.map.insert(key, values);
        }
    }

    pub fn get(&self, key: &K) -> Option<&Vec<V>> {
        self.map.get(key)
    }

    pub fn contains_key(&self, key: &K) -> bool {
        self.map.contains_key(key)
    }
}