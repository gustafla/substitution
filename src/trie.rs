use thiserror::Error;

/// Type for the size of the trie's alphabet
pub type AlphabetSize = usize;

/// Errors that trie and key operations may return
#[derive(Error, Debug)]
pub enum Error {
    /// Error which will be returned when a key cannot be used with given alphabet size
    /// E.g. the key has value 19 but alphabet size is 10
    #[error("value {value} in key does not fit in alphabet size {size}")]
    KeyNotInAlphabet { value: usize, size: usize },
    /// char does not fit in an usize, but Key's impl FromStr is used
    #[error("char {value} does not fit in an usize, thus cannot be used in trie")]
    CharCannotIndex {
        value: char,
        source: std::num::TryFromIntError,
    },
}

/// Trie's key's elements need to convert to usize and be small, automatically copied
pub trait KeyElement: Into<usize> + Copy {}
impl<E: Into<usize> + Copy> KeyElement for E {}

/// Key for trie operations
#[derive(Clone, Debug)]
pub struct Key<const R: AlphabetSize, E: KeyElement> {
    buf: Vec<E>,
}

impl<const R: AlphabetSize, E: KeyElement> Key<R, E> {
    /// Iterate over references to the key's elements
    fn iter(&self) -> std::slice::Iter<E> {
        self.buf.iter()
    }
}

impl<const R: AlphabetSize, E: KeyElement> TryFrom<Vec<E>> for Key<R, E> {
    type Error = Error;

    fn try_from(buf: Vec<E>) -> Result<Self, Self::Error> {
        for value in &buf {
            if (*value).into() >= R {
                return Err(Error::KeyNotInAlphabet {
                    value: (*value).into(),
                    size: R,
                });
            }
        }
        Ok(Self { buf })
    }
}

// Convert &str (such as "this" <- literal) to a key
impl<const R: AlphabetSize> std::str::FromStr for Key<R, usize> {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut buf = Vec::with_capacity(s.len());
        for c in s.chars() {
            let u: u32 = c.into();
            let us: usize = u.try_into().map_err(|e| Error::CharCannotIndex {
                value: c,
                source: e,
            })?;
            if us >= R {
                return Err(Error::KeyNotInAlphabet { value: us, size: R });
            }
            buf.push(us);
        }
        Ok(Self { buf })
    }
}

/// Type used for indirect pointing to other nodes from nodes
type NodeIndex = std::num::NonZeroUsize;

/// A node of trie, which holds indices to other nodes
#[derive(Clone)]
struct Node<const R: AlphabetSize, T> {
    children: [Option<NodeIndex>; R],
    value: Option<T>,
}

impl<const R: AlphabetSize, T> Node<R, T> {
    /// Create a new empty node
    fn new() -> Self {
        Self {
            children: [None; R],
            value: None,
        }
    }

    /// Get the index for the next node for key
    fn get_idx(&self, key: usize) -> &Option<NodeIndex> {
        self.children.get(key).unwrap()
    }

    /// Set the index for the next node for key
    fn set_idx(&mut self, key: usize, idx: NodeIndex) {
        let node = self.children.get_mut(key).unwrap();
        *node = Some(idx);
    }
}

// Allow accessing the value in the node
impl<const R: AlphabetSize, T> AsRef<Option<T>> for Node<R, T> {
    fn as_ref(&self) -> &Option<T> {
        &self.value
    }
}

// Allow accessing the value in the node
impl<const R: AlphabetSize, T> AsMut<Option<T>> for Node<R, T> {
    fn as_mut(&mut self) -> &mut Option<T> {
        &mut self.value
    }
}

/// Trie, where R is the cardinality of the alphabet in use.
///
/// Supports insertion and retrieval.
pub struct Trie<const R: AlphabetSize, T> {
    nodes: Vec<Node<R, T>>,
}

impl<const R: AlphabetSize, T> Trie<R, T> {
    /// Initialize an empty trie
    pub fn new() -> Self {
        Self {
            nodes: vec![Node::new()],
        }
    }

    /// Create a new node and return it's index
    fn create(&mut self) -> NodeIndex {
        self.nodes.push(Node::new());
        NodeIndex::new(self.nodes.len() - 1).unwrap()
    }

    /// Insert a value into the trie
    pub fn insert<E: KeyElement>(&mut self, key: &Key<R, E>, value: T) {
        let mut node = 0; // Root node index

        // Walk through key elements
        for key in key.iter() {
            // Look up next node's index by key
            node = match self.nodes[node].get_idx((*key).into()) {
                // Go to next if it already exists
                Some(next) => next.get(),
                // Create a new node and go to it if not preexisting
                None => {
                    let new_node = self.create();
                    self.nodes[node].set_idx((*key).into(), new_node);
                    new_node.get()
                }
            }
        }

        *self.nodes[node].as_mut() = Some(value);
    }

    /// Retrieve value for given key and tell how long prefix is contained in trie
    pub fn prefix<E: KeyElement>(&self, key: &Key<R, E>) -> (usize, &Option<T>) {
        let mut node = 0; // Root node index
        let mut depth = 0;

        for key in key.iter() {
            if let Some(next) = self.nodes[node].get_idx((*key).into()) {
                node = next.get();
                depth += 1;
            } else {
                return (depth, &None);
            }
        }

        (depth, self.nodes[node].as_ref())
    }
}

/// Set based on trie
pub struct Set<const R: AlphabetSize> {
    trie: Trie<R, ()>,
}

impl<const R: AlphabetSize> Set<R> {
    /// Initialize an empty set
    pub fn new() -> Self {
        Self { trie: Trie::new() }
    }

    /// Insert a value (key) into the set
    pub fn insert<E: KeyElement>(&mut self, key: &Key<R, E>) {
        self.trie.insert(key, ());
    }

    /// Returns true if the value (key) has been inserted, otherwise false
    pub fn contains<E: KeyElement>(&self, key: &Key<R, E>) -> bool {
        self.trie.prefix(key).1.is_some()
    }

    /// Returns `key.len() + 1` if the value (key) has been inserted, otherwise found prefix length
    pub fn prefix_score<E: KeyElement>(&self, key: &Key<R, E>) -> usize {
        let (len, ins) = self.trie.prefix(key);
        len + if ins.is_some() { 1 } else { 0 }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn no_insertion_not_contained() {
        const R: AlphabetSize = 128;
        let key = Key::<R, u8>::try_from(b"hello".to_vec()).unwrap();
        let set = Set::<R>::new();
        assert!(!set.contains(&key))
    }

    #[test]
    fn insertion_contained() {
        const R: AlphabetSize = 128;
        let key = Key::<R, u8>::try_from(b"hello".to_vec()).unwrap();
        let mut set = Set::<R>::new();
        set.insert(&key);
        assert!(set.contains(&key))
    }

    #[test]
    fn insertion_subkey_not_contained() {
        const R: AlphabetSize = 128;
        let key = Key::<R, u8>::try_from(b"hello".to_vec()).unwrap();
        let mut set = Set::<R>::new();
        set.insert(&key);
        let key = Key::<R, u8>::try_from(b"hell".to_vec()).unwrap();
        assert!(!set.contains(&key))
    }

    #[test]
    fn multiple_insertions() {
        const R: AlphabetSize = 128;
        let keys: Vec<Key<R, usize>> = ["apples", "oranges", "bananas"]
            .iter()
            .map(|str| str.parse().unwrap())
            .collect();
        let false_keys: Vec<Key<R, usize>> = ["apple", "orangee", "bananasplit", ""]
            .iter()
            .map(|str| str.parse().unwrap())
            .collect();

        let mut set = Set::<R>::new();
        for key in &keys {
            set.insert(key);
        }
        for key in &false_keys {
            assert!(!set.contains(key))
        }
        for key in &keys {
            assert!(set.contains(key))
        }
    }

    #[test]
    fn key_from_str_error() {
        use std::str::FromStr;
        assert!(matches!(
            Key::<96, usize>::from_str("Hello!"),
            Err(Error::KeyNotInAlphabet {
                value: 101,
                size: 96,
            })
        ))
    }
}
