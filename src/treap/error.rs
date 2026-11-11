//! Errors returned by the treap operations.

use std::fmt::Display;

/// An error returned by a treap operation.
#[derive(Debug, Clone)]
pub enum TreapError {
    /// The key is already in the treap. Treaps do not store duplicate keys.
    ///
    /// Returned by `insert` when the key is present.
    DuplicateKey,
    /// All the keys of T1 must be smaller than the keys of T2.
    ///
    /// Returned by `join` when the given treaps cannot be joined.
    UnjoinableTreaps,
}

impl Display for TreapError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TreapError::DuplicateKey => write!(f, "cannot insert duplicate keys"),
            TreapError::UnjoinableTreaps => {
                write!(f, "keys in T1 must be strictly smaller than keys in T2")
            }
        }
    }
}

impl std::error::Error for TreapError {}
