use indexmap::IndexSet;

use std::collections::HashMap;

/// Update a IndexSet "a" with the values from "b"
#[inline]
pub(crate) fn append_indexset<T>(a: &mut IndexSet<T>, b: &IndexSet<T>)
where
    T: Clone + Eq + Ord + std::hash::Hash,
{
    for value in b {
        a.insert(value.clone());
    }
}

/// Update a Hashmap "a" with the values from "b".
#[inline]
pub(crate) fn append_hashmap<K, V>(a: &mut HashMap<K, V>, b: &HashMap<K, V>)
where
    K: Clone + Eq + Ord + std::hash::Hash,
    V: Clone,
{
    for (key, value) in b {
        a.insert(key.clone(), value.clone());
    }
}
