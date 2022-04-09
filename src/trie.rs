/// Type for the size of the trie's alphabet
pub type AlphabetSize = usize;

/// Error type which will be returned when a key cannot be used with given alphabet size
/// E.g. the key has value 19 but alphabet size is 10
#[derive(Debug)]
pub struct KeyNotInAlphabet;

/// Type used for looking up node indices from key's elements
#[derive(Clone, Copy)]
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
    type Error = KeyNotInAlphabet;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        (value < R)
            .then(|| Self { inner: value })
            .ok_or(KeyNotInAlphabet)
    }
}

/// Key for trie operations
#[derive(Clone)]
pub struct Key<const R: AlphabetSize> {
    buf: Vec<KeyIndex<R>>,
}

impl<const R: AlphabetSize> Key<R> {
    // Convert a u8 slice (byte string) to Key
    pub fn from_bytes(slice: &[u8]) -> Result<Self, KeyNotInAlphabet> {
        let mut buf = Vec::with_capacity(slice.len());
        for value in slice {
            buf.push(usize::from(*value).try_into()?);
        }
        Ok(Self { buf })
    }
}

// Enables iterating a key with for-loop
impl<const R: AlphabetSize> IntoIterator for Key<R> {
    type Item = KeyIndex<R>;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.buf.into_iter()
    }
}

/// Type used for indirect pointing to other nodes from nodes
type NodeIndex = usize;

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

    fn get(&self) -> &Option<T> {
        &self.value
    }

    fn set(&mut self, value: Option<T>) {
        self.value = value;
    }
}

/// A zero-sized type for just tracking if a key was inserted or not
struct Inserted;
type SetNode<const R: usize> = Node<R, Inserted>;

/// A set as a trie, where R is the cardinality of the alphabet in use.
///
/// Supports insertion and lookups.
pub struct Set<const R: usize> {
    nodes: Vec<SetNode<R>>,
}

impl<const R: usize> Set<R> {
    /// Initialize an empty `Set<R>`
    pub fn new() -> Self {
        Self {
            nodes: vec![Node::new()],
        }
    }

    /// Creates a new node and returns it's index
    fn create(&mut self) -> NodeIndex {
        self.nodes.push(Node::new());
        self.nodes.len() - 1
    }

    /// Immutable node reference
    fn node(&self, idx: NodeIndex) -> &Node<R, Inserted> {
        &self.nodes[idx]
    }

    /// Mutable node reference
    fn node_mut(&mut self, idx: NodeIndex) -> &mut SetNode<R> {
        &mut self.nodes[idx]
    }

    /// Insert a key into the set.
    ///
    /// `key`'s elements need to be able to convert to usize.
    pub fn insert(&mut self, key: Key<R>) {
        let mut node = 0; // Root node index

        // Walk through key elements
        for key in key {
            // Look up next node's index by key
            node = match self.node(node).get_idx(key) {
                // Go to next if it already exists
                Some(next) => *next,
                // Create a new node and go to it if not preexisting
                None => {
                    let new_node = self.create();
                    self.node_mut(node).set_idx(key, new_node);
                    new_node
                }
            }
        }

        self.node_mut(node).set(Some(Inserted));
    }

    pub fn contains(&self, key: Key<R>) -> bool {
        let mut node = 0; // Root node index

        for key in key {
            if let Some(next) = self.node(node).get_idx(key) {
                node = *next;
            } else {
                return false;
            }
        }

        self.node(node).get().is_some()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn no_insertion_not_contained() {
        const R: usize = 128;
        let key = Key::<R>::from_bytes(b"hello").unwrap();
        let set = Set::<R>::new();
        assert!(!set.contains(key))
    }

    #[test]
    fn insertion_contained() {
        const R: usize = 128;
        let key = Key::<R>::from_bytes(b"hello").unwrap();
        let mut set = Set::<R>::new();
        set.insert(key.clone());
        assert!(set.contains(key))
    }
}
