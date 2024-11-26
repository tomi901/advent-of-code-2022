
#[derive(Debug, Clone)]
pub struct KeyedOrd<T, K> {
    pub value: T,
    pub key: K,
}

impl<T, K> KeyedOrd<T, K> {
    pub fn new(value: T, key: K) -> Self {
        Self { value, key }
    }
}

impl<T, K: PartialEq> PartialEq for KeyedOrd<T, K> {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

impl<T, K: Eq> Eq for KeyedOrd<T, K> {}

impl<T, K: PartialOrd> PartialOrd for KeyedOrd<T, K> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.key.partial_cmp(&other.key)
    }
}

impl<T, K: Ord> Ord for KeyedOrd<T, K> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.key.cmp(&other.key)
    }
}
