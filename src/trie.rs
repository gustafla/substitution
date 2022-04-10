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

/// Type used for looking up node indices from key's elements
#[derive(Clone, Copy, Debug)]
pub struct KeyIndex<const R: AlphabetSize> {
    inner: usize,
}

// Enables dereferencing back to usize. (`*key_index`)
impl<const R: AlphabetSize> std::ops::Deref for KeyIndex<R> {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

// The only way to create KeyIndex. Checks the invariant that the index is lower than alphabet size
impl<const R: AlphabetSize> TryFrom<usize> for KeyIndex<R> {
    type Error = Error;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        (value < R)
            .then(|| Self { inner: value })
            .ok_or(Error::KeyNotInAlphabet { value, size: R })
    }
}

/// Key for trie operations
#[derive(Clone, Debug)]
pub struct Key<const R: AlphabetSize> {
    buf: Vec<KeyIndex<R>>,
}

impl<const R: AlphabetSize> Key<R> {
    /// Iterate over references to the key's elements
    fn iter(&self) -> std::slice::Iter<KeyIndex<R>> {
        self.buf.iter()
    }
}

impl<const R: AlphabetSize> Key<R> {
    // Convert a u8 slice (byte string) to Key
    // This cannot be (for now) TryFrom<AsRef<[u8]>> because of conflicting blanket impl in core
    pub fn from_bytes(slice: &[u8]) -> Result<Self, Error> {
        let mut buf = Vec::with_capacity(slice.len());
        for value in slice {
            buf.push(usize::from(*value).try_into()?);
        }
        Ok(Self { buf })
    }
}

impl<const R: AlphabetSize> std::str::FromStr for Key<R> {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut buf = Vec::with_capacity(s.len());
        for c in s.chars() {
            let u: u32 = c.into();
            let us: usize = u.try_into().map_err(|e| Error::CharCannotIndex {
                value: c,
                source: e,
            })?;
            buf.push(us.try_into()?);
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
    fn get_idx(&self, key: KeyIndex<R>) -> &Option<NodeIndex> {
        self.children.get(*key).unwrap()
    }

    /// Set the index for the next node for key
    fn set_idx(&mut self, key: KeyIndex<R>, idx: NodeIndex) {
        let node = self.children.get_mut(*key).unwrap();
        *node = Some(idx);
    }
}

// Allow accessing the value in the node by dereferencing
impl<const R: AlphabetSize, T> std::ops::Deref for Node<R, T> {
    type Target = Option<T>;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

// Allow accessing the value in the node by dereferencing
impl<const R: AlphabetSize, T> std::ops::DerefMut for Node<R, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

/// A zero-sized type for just tracking if a key was inserted or not
struct Inserted;

/// A Node for a Set
type SetNode<const R: AlphabetSize> = Node<R, Inserted>;

/// A set as a trie, where R is the cardinality of the alphabet in use.
///
/// Supports insertion and lookups.
pub struct Set<const R: AlphabetSize> {
    nodes: Vec<SetNode<R>>,
}

impl<const R: AlphabetSize> Set<R> {
    /// Initialize an empty `Set<R>`
    pub fn new() -> Self {
        Self {
            nodes: vec![Node::new()],
        }
    }

    /// Creates a new node and returns it's index
    fn create(&mut self) -> NodeIndex {
        self.nodes.push(Node::new());
        NodeIndex::new(self.nodes.len() - 1).unwrap()
    }

    /// Insert a key into the set.
    pub fn insert(&mut self, key: &Key<R>) {
        let mut node = 0; // Root node index

        // Walk through key elements
        for key in key.iter() {
            // Look up next node's index by key
            node = match self.nodes[node].get_idx(*key) {
                // Go to next if it already exists
                Some(next) => next.get(),
                // Create a new node and go to it if not preexisting
                None => {
                    let new_node = self.create();
                    self.nodes[node].set_idx(*key, new_node);
                    new_node.get()
                }
            }
        }

        *self.nodes[node] = Some(Inserted);
    }

    pub fn contains(&self, key: &Key<R>) -> bool {
        let mut node = 0; // Root node index

        for key in key.iter() {
            if let Some(next) = self.nodes[node].get_idx(*key) {
                node = next.get();
            } else {
                return false;
            }
        }

        self.nodes[node].is_some()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn no_insertion_not_contained() {
        const R: AlphabetSize = 128;
        let key = Key::<R>::from_bytes(b"hello").unwrap();
        let set = Set::<R>::new();
        assert!(!set.contains(&key))
    }

    #[test]
    fn insertion_contained() {
        const R: AlphabetSize = 128;
        let key = Key::<R>::from_bytes(b"hello").unwrap();
        let mut set = Set::<R>::new();
        set.insert(&key);
        assert!(set.contains(&key))
    }

    #[test]
    fn insertion_subkey_not_contained() {
        const R: AlphabetSize = 128;
        let key = Key::<R>::from_bytes(b"hello").unwrap();
        let mut set = Set::<R>::new();
        set.insert(&key);
        let key = Key::<R>::from_bytes(b"hell").unwrap();
        assert!(!set.contains(&key))
    }

    #[test]
    fn multiple_insertions() {
        const R: AlphabetSize = 128;
        let keys: Vec<Key<R>> = ["apples", "oranges", "bananas"]
            .iter()
            .map(|str| str.parse().unwrap())
            .collect();
        let false_keys: Vec<Key<R>> = ["apple", "orangee", "bananasplit", ""]
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
            Key::<96>::from_str("Hello!"),
            Err(Error::KeyNotInAlphabet {
                value: 101,
                size: 96,
            })
        ))
    }
}
