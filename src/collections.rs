use crate::model::{IndexMap, IndexSet};

/// Update a IndexSet "a" with the values from "b"
#[inline]
pub(crate) fn append_set<T>(a: &mut IndexSet<T>, b: &IndexSet<T>)
where
    T: Clone + Eq + Ord + std::hash::Hash,
{
    for value in b {
        a.insert(value.clone());
    }
}

/// Update a map "a" with the values from "b".
#[inline]
pub(crate) fn append_map<K, V>(a: &mut IndexMap<K, V>, b: &IndexMap<K, V>)
where
    K: Clone + Eq + Ord + std::hash::Hash,
    V: Clone,
{
    for (key, value) in b {
        a.insert(key.clone(), value.clone());
    }
}
