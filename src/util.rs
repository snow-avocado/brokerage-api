use std::collections::HashSet;

pub(crate) fn dedup_ordered<T>(v: Vec<T>) -> Vec<T>
where
    T: Eq + std::hash::Hash + Clone,
{
    let mut seen = HashSet::new(); // Create an empty HashSet to track seen elements
    v.into_iter()
        .filter(|item| seen.insert(item.clone())) // Filter out elements that have already been seen
        .collect() // Collect the unique elements into a new vector
}