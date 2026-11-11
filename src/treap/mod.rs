//! Treaps: binary search trees on the keys that are also heaps on the priorities.
//!
//! Every node holds a key and a priority. The keys follow the binary search
//! tree order (smaller keys on the left, greater keys on the right) and the
//! priorities follow the heap order chosen with an [`Order`]: a parent is
//! always above its children. When the priorities are random the expected
//! depth of the tree is `O(log n)`.
//!
//! The module provides:
//! - [`Treap`]: the generic treap, with keys, priorities and heap order chosen
//!   by the caller, and its iterators [`Iter`] and [`IntoIter`];
//! - [`RandomTreap`]: a wrapper that draws random priorities; its iterators
//!   are in the [`random`] submodule;
//! - the heap orders [`MinHeap`] and [`MaxHeap`], through the [`Order`] trait;
//! - [`TreapError`]: the errors returned by the treap operations.
//!
//! # Layout
//!
//! - `generic`, `order`, `error`: the public items above, re-exported here;
//! - [`random`]: [`RandomTreap`] and its iterators;
//! - `node` (private): the nodes and the recursive algorithms shared by every
//!   treap.

mod error;
mod generic;
mod node;
mod order;
pub mod random;

pub use error::TreapError;
pub use generic::{IntoIter, Iter, Treap};
pub use order::{MaxHeap, MinHeap, Order};
pub use random::RandomTreap;
