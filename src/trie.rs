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
}

/// Trie's key's elements need to convert to usize and be small, automatically copied
pub trait KeyElement: Into<usize> + Copy {}
impl<E: Into<usize> + Copy> KeyElement for E {}

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
    fn get_idx(&self, key: usize) -> Option<NodeIndex> {
        self.children[key]
    }

    /// Set the index for the next node for key
    fn set_idx(&mut self, key: usize, idx: NodeIndex) {
        self.children[key] = Some(idx);
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

/// Trie, where R is the cardinality of the alphabet in use and B is the index base.
///
/// Supports insertion and retrieval.
pub struct Trie<const R: AlphabetSize, const B: usize, T> {
    nodes: Vec<Node<R, T>>,
}

impl<const R: AlphabetSize, const B: usize, T> Trie<R, B, T> {
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

    /// Under the hood explicit bounds check
    fn check(key: usize) -> Result<(), Error> {
        if key >= R {
            Err(Error::KeyNotInAlphabet {
                value: key,
                size: R,
            })
        } else {
            Ok(())
        }
    }

    /// Insert a value into the trie
    pub fn insert<E: KeyElement>(&mut self, key: &[E], value: T) -> Result<(), Error> {
        let mut node = 0; // Root node index

        // Walk through key elements
        for key in key.iter().map(|e| (*e).into() - B) {
            // Explicit bounds check
            Self::check(key)?;

            // Look up next node's index by key
            node = match self.nodes[node].get_idx(key) {
                // Go to next if it already exists
                Some(next) => next.get(),
                // Create a new node and go to it if not preexisting
                None => {
                    let new_node = self.create();
                    self.nodes[node].set_idx(key, new_node);
                    new_node.get()
                }
            }
        }

        *self.nodes[node].as_mut() = Some(value);
        Ok(())
    }

    /// Retrieve value for given key and tell how long prefix is contained in trie
    pub fn prefix<E: KeyElement>(&self, key: &[E]) -> Result<(usize, &Option<T>), Error> {
        let mut node = 0; // Root node index
        let mut depth = 0;

        for key in key.iter().map(|e| (*e).into() - B) {
            // Explicit bounds check
            Self::check(key)?;

            if let Some(next) = self.nodes[node].get_idx(key) {
                node = next.get();
                depth += 1;
            } else {
                return Ok((depth, &None));
            }
        }

        Ok((depth, self.nodes[node].as_ref()))
    }
}

/// Set based on trie
pub struct Set<const R: AlphabetSize, const B: usize> {
    trie: Trie<R, B, ()>,
}

impl<const R: AlphabetSize, const B: usize> Set<R, B> {
    /// Initialize an empty set
    pub fn new() -> Self {
        Self { trie: Trie::new() }
    }

    /// Insert a value (key) into the set
    pub fn insert<E: KeyElement>(&mut self, key: &[E]) -> Result<(), Error> {
        self.trie.insert(key, ())
    }

    /// Returns true if the value (key) has been inserted, otherwise false
    pub fn contains<E: KeyElement>(&self, key: &[E]) -> Result<bool, Error> {
        Ok(self.trie.prefix(key)?.1.is_some())
    }

    /// Returns `key.len() + 1` if the value (key) has been inserted, otherwise found prefix length
    pub fn prefix_score<E: KeyElement>(&self, key: &[E]) -> Result<usize, Error> {
        let (len, ins) = self.trie.prefix(key)?;
        Ok(len + usize::from(ins.is_some()))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn no_insertion_not_contained() {
        const R: AlphabetSize = 128;
        let set = Set::<R, 0>::new();
        assert!(!set.contains(b"hello").unwrap())
    }

    #[test]
    fn insertion_contained() {
        const R: AlphabetSize = 128;
        let mut set = Set::<R, 0>::new();
        set.insert(b"hello").unwrap();
        assert!(set.contains(b"hello").unwrap())
    }

    #[test]
    fn insertion_prefix_not_contained() {
        const R: AlphabetSize = 128;
        let mut set = Set::<R, 0>::new();
        set.insert(b"hello").unwrap();
        assert!(!set.contains(b"hell").unwrap())
    }

    #[test]
    fn insertion_prefix_score() {
        const R: AlphabetSize = 128;
        let mut set = Set::<R, 0>::new();
        set.insert(b"hello").unwrap();
        assert_eq!(set.prefix_score(b"hell").unwrap(), 4);
        assert_eq!(set.prefix_score(b"hell0").unwrap(), 4);
        assert_eq!(set.prefix_score(b"hello").unwrap(), 6);
    }

    #[test]
    fn multiple_insertions() {
        const R: AlphabetSize = 128;
        let keys = [
            b"apples".as_slice(),
            b"oranges".as_slice(),
            b"bananas".as_slice(),
        ];
        let false_keys = [
            b"apple".as_slice(),
            b"orangee".as_slice(),
            b"bananasplit".as_slice(),
            b"".as_slice(),
        ];

        let mut set = Set::<R, 0>::new();
        for key in &keys {
            set.insert(key).unwrap();
        }
        for key in &false_keys {
            assert!(!set.contains(key).unwrap())
        }
        for key in &keys {
            assert!(set.contains(key).unwrap())
        }
    }

    #[test]
    fn key_error() {
        const R: AlphabetSize = 96;
        let mut set = Set::<R, 0>::new();
        assert!(matches!(
            set.insert(b"Hello!"),
            Err(Error::KeyNotInAlphabet {
                value: 101,
                size: 96
            })
        ));
    }
}
