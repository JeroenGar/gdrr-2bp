use std::hash::Hash;

use indexmap::{IndexMap, IndexSet};

pub struct MultiMap<K : Hash + Eq, V> {
    map: IndexMap<K, IndexSet<V>>
}

impl<K : Hash + Eq,V : Hash + Eq> MultiMap<K,V> {
    pub fn new() -> Self {
        let map = IndexMap::new();
        Self { map }
    }

    pub fn insert(&mut self, key: K, value: V) {
        let values = self.map.entry(key).or_insert(IndexSet::new());
        values.insert(value);
    }

    pub fn insert_all(&mut self, key: K, values: IndexSet<V>) {
        if self.map.contains_key(&key) {
            self.map.get_mut(&key).unwrap().extend(values);
        }
        else{
            self.map.insert(key, values);
        }
    }

    pub fn get(&self, key: &K) -> Option<&IndexSet<V>> {
        self.map.get(key)
    }

    pub fn contains_key(&self, key: &K) -> bool {
        self.map.contains_key(key)
    }

    pub fn remove_all(&mut self, key: &K) -> Option<IndexSet<V>> {
        self.map.remove(key)
    }

    pub fn remove(&mut self, key : &K, value: &V) -> bool {
        match self.map.get_mut(key){
            Some(values) => {
                values.remove(value)
            }
            None => false
        }
    }
}