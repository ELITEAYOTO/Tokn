use std::collections::HashSet;

#[derive(Debug, Default)]
pub struct DedupSet {
    seen: HashSet<String>,
}

impl DedupSet {
    pub fn insert(&mut self, key: impl Into<String>) -> bool {
        self.seen.insert(key.into())
    }

    pub fn len(&self) -> usize {
        self.seen.len()
    }

    pub fn is_empty(&self) -> bool {
        self.seen.is_empty()
    }
}
