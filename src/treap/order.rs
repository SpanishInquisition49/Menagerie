//! Heap orders for the priorities of a treap.
//!
//! An [`Order`] decides which of two priorities stays higher in the tree.
//! The orders are zero-sized types, so choosing one costs no memory and no
//! runtime check: the comparison is fixed at compile time.

/// The heap order of the priorities in a treap.
///
/// Implementors are expected to be zero-sized types: the order is passed by
/// value through the recursive operations, which is free for a zero-sized
/// type. `Default` lets a treap build its order without being given one.
pub trait Order: Copy + Default {
    /// Returns `true` if a node with priority `a` must stay above a node with
    /// priority `b`.
    ///
    /// The comparison must be strict: equal priorities are never above each
    /// other.
    fn is_above<P: Ord>(self, a: &P, b: &P) -> bool;
}

/// Min-heap order: the smallest priority is at the root.
///
/// # Examples
///
/// ```
/// use menagerie::treap::{MinHeap, Order};
///
/// assert!(MinHeap.is_above(&1, &2));
/// assert!(!MinHeap.is_above(&2, &2));
/// ```
#[derive(Clone, Copy, Default)]
pub struct MinHeap;

impl Order for MinHeap {
    fn is_above<P: Ord>(self, a: &P, b: &P) -> bool {
        a < b
    }
}

/// Max-heap order: the greatest priority is at the root.
///
/// # Examples
///
/// ```
/// use menagerie::treap::{MaxHeap, Order};
///
/// assert!(MaxHeap.is_above(&2, &1));
/// assert!(!MaxHeap.is_above(&2, &2));
/// ```
#[derive(Clone, Copy, Default)]
pub struct MaxHeap;

impl Order for MaxHeap {
    fn is_above<P: Ord>(self, a: &P, b: &P) -> bool {
        a > b
    }
}
