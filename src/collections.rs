use indexmap::IndexSet;

/// Update a IndexSet "a" with the values from "b"
pub fn append<T>(a: &mut IndexSet<T>, b: &IndexSet<T>)
where T: Clone + Eq + Ord + std::hash::Hash
{

    for value in b {
        a.insert(value.clone());
    }
}
