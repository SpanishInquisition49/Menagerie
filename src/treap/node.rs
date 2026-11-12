//! Treap nodes and the recursive algorithms on them.
//!
//! Everything here is internal to the crate: the public treaps own a
//! [`Link`] to the root and delegate every
//! operation to the functions of [`TreapNode`].

use std::borrow::Borrow;
use std::fmt::{self, Debug};
use std::iter;
use std::mem::{swap, take};

use super::error::TreapError;
use super::order::Order;

/// An owning pointer to a subtree; `None` is the empty subtree.
pub(crate) type Link<T, P> = Option<Box<TreapNode<T, P>>>;

/// A node of a treap with keys of type `T` and priorities of type `P`.
///
/// The subtree rooted here is a binary search tree on the keys and a heap on
/// the priorities, in the [`Order`] passed to each operation. The order is not
/// stored in the node: every operation on the same tree must receive the same
/// order.
///
/// Duplicate keys are not supported.
#[derive(Clone)]
pub(crate) struct TreapNode<T, P> {
    key: T,
    /// The priority: the node is above its children in the heap order.
    priority: P,
    /// Subtree with the keys smaller than `key`.
    left: Link<T, P>,
    /// Subtree with the keys greater than `key`.
    right: Link<T, P>,
}

/// Formats the items of a cloneable iterator as a list, without consuming
/// it: the `Debug` implementations of the iterators print what they have
/// left, as the iterators of the standard collections do.
pub(crate) struct DebugList<I>(pub(crate) I);

impl<I> Debug for DebugList<I>
where
    I: Iterator + Clone,
    I::Item: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.0.clone()).finish()
    }
}

/// In-order iterator over the keys and priorities of a subtree, by reference.
///
/// The stack holds the nodes whose key has not been returned yet, along the
/// path to the next one: its size is at most the depth of the tree, so
/// `O(log n)` expected with random priorities.
pub(crate) struct Iter<'a, T, P> {
    stack: Vec<&'a TreapNode<T, P>>,
}

impl<'a, T, P> Iter<'a, T, P> {
    /// An iterator over the subtree behind `root`, from the smallest key.
    pub(crate) fn new(root: &'a Link<T, P>) -> Self {
        let mut iter = Self { stack: Vec::new() };
        iter.push_left_spine(root.as_deref());
        iter
    }

    /// Push `node` and all its left descendants: the last one pushed holds
    /// the smallest key of the subtree.
    fn push_left_spine(&mut self, mut node: Option<&'a TreapNode<T, P>>) {
        while let Some(n) = node {
            self.stack.push(n);
            node = n.left.as_deref();
        }
    }
}

// Not derived: the derive would require `T: Clone` and `P: Clone`, but
// cloning only copies the stack of references.
impl<T, P> Clone for Iter<'_, T, P> {
    fn clone(&self) -> Self {
        Self {
            stack: self.stack.clone(),
        }
    }
}

impl<T: Debug, P: Debug> Debug for Iter<'_, T, P> {
    /// The pairs not returned yet.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        DebugList(self.clone()).fmt(f)
    }
}

impl<'a, T, P> Iterator for Iter<'a, T, P> {
    type Item = (&'a T, &'a P);

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.stack.pop()?;
        // The keys after `node` start with the smallest key of its right subtree.
        self.push_left_spine(node.right.as_deref());
        Some((&node.key, &node.priority))
    }
}

/// In-order iterator over the keys and priorities of a subtree, by value.
///
/// It owns the nodes and frees each one when its key is returned.
pub(crate) struct IntoIter<T, P> {
    stack: Vec<Box<TreapNode<T, P>>>,
}

impl<T, P> IntoIter<T, P> {
    /// An iterator that consumes the subtree behind `root`, from the smallest key.
    pub(crate) fn new(root: Link<T, P>) -> Self {
        let mut iter = Self { stack: Vec::new() };
        iter.push_left_spine(root);
        iter
    }

    /// The pairs not returned yet, by reference and in order, without
    /// consuming them: each node on the stack comes before its right subtree,
    /// and the stack is popped from the end.
    pub(crate) fn remaining(&self) -> impl Iterator<Item = (&T, &P)> + Clone + '_ {
        self.stack
            .iter()
            .rev()
            .flat_map(|node| iter::once((&node.key, &node.priority)).chain(Iter::new(&node.right)))
    }

    /// Push `link` and all its left descendants, detaching each from its
    /// parent so that the stack owns it.
    fn push_left_spine(&mut self, mut link: Link<T, P>) {
        while let Some(mut node) = link {
            link = node.left.take();
            self.stack.push(node);
        }
    }
}

impl<T: Debug, P: Debug> Debug for IntoIter<T, P> {
    /// The pairs not returned yet.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        DebugList(self.remaining()).fmt(f)
    }
}

impl<T, P> Iterator for IntoIter<T, P> {
    type Item = (T, P);

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.stack.pop()?;
        // The left subtree was already detached by `push_left_spine`.
        let TreapNode {
            key,
            priority,
            right,
            ..
        } = *node;
        self.push_left_spine(right);
        Some((key, priority))
    }
}

impl<T, P> TreapNode<T, P>
where
    T: Ord,
    P: Ord,
{
    /// Create a leaf with the given key and priority.
    pub fn new(key: T, priority: P) -> Self {
        Self {
            key,
            priority,
            left: None,
            right: None,
        }
    }

    /// Insert `node` in the subtree rooted at `self`.
    ///
    /// The node is attached as a leaf in its binary search tree position, then
    /// rotated up while it is above its parent in `order`. Thanks to the
    /// content swap in the rotations, `self` may end up holding a different
    /// node, so the caller's link never needs updating.
    ///
    /// # Errors
    ///
    /// [`TreapError::DuplicateKey`] if the key is already in the subtree. The
    /// duplicate is found on the way down, before any change, so the subtree
    /// is left untouched.
    pub fn insert<O: Order>(&mut self, node: TreapNode<T, P>, order: O) -> Result<(), TreapError> {
        if self.key == node.key {
            drop(node);
            return Err(TreapError::DuplicateKey);
        }
        if self.key > node.key {
            match &mut self.left {
                Some(sx) => {
                    sx.insert(node, order)?;
                }
                None => {
                    let leaf = Box::new(node);
                    self.left = Some(leaf);
                }
            }
            // The left subtree is a valid treap: only its root can be above `self`.
            if self
                .left
                .as_ref()
                .is_some_and(|l| order.is_above(&l.priority, &self.priority))
            {
                self.rotate_right();
            }
        } else {
            match &mut self.right {
                Some(dx) => {
                    dx.insert(node, order)?;
                }
                None => {
                    let leaf = Box::new(node);
                    self.right = Some(leaf);
                }
            }
            // The right subtree is a valid treap: only its root can be above `self`.
            if self
                .right
                .as_ref()
                .is_some_and(|r| order.is_above(&r.priority, &self.priority))
            {
                self.rotate_left();
            }
        }
        Ok(())
    }

    /// Remove the key equal to `key` from the subtree behind `node` and
    /// return it, or `None` if the key is not there (the subtree is then
    /// unchanged).
    ///
    /// `key` may be any borrowed form of the key type, as in
    /// [`TreapNode::get`].
    ///
    /// The node with the key is rotated down, each time lifting the child
    /// that is above the other in `order`, until it is a leaf; then the leaf
    /// is detached from its link.
    pub fn remove<Q, O>(node: &mut Link<T, P>, key: &Q, order: O) -> Option<T>
    where
        T: Borrow<Q>,
        Q: Ord + ?Sized,
        O: Order,
    {
        let n = node.as_mut()?;
        let here = n.key.borrow();
        match (&n.left, &n.right) {
            // Found, and a leaf: detach it.
            (None, None) if here == key => take(node).map(|n| n.key),
            // Found, two children: lift the higher one. After the rotation
            // the node to remove is on the opposite side.
            (Some(l), Some(r)) if here == key => {
                if order.is_above(&l.priority, &r.priority) {
                    n.rotate_right();
                    TreapNode::remove(&mut n.right, key, order)
                } else {
                    n.rotate_left();
                    TreapNode::remove(&mut n.left, key, order)
                }
            }
            // Found, one child: lift it.
            (Some(_), None) if here == key => {
                n.rotate_right();
                TreapNode::remove(&mut n.right, key, order)
            }
            (None, Some(_)) if here == key => {
                n.rotate_left();
                TreapNode::remove(&mut n.left, key, order)
            }
            // Not here: search the subtree that can hold the key.
            (Some(_), _) if here > key => TreapNode::remove(&mut n.left, key, order),
            (_, Some(_)) if here < key => TreapNode::remove(&mut n.right, key, order),
            // The subtree that could hold the key is empty.
            (_, _) => None,
        }
    }

    /// Split the subtree `node` into the keys smaller than `key` and the
    /// keys greater than or equal to `key`, returned in this order: `key`
    /// itself, if present, goes to the second part, as in
    /// `BTreeSet::split_off`.
    ///
    /// If the root is smaller than `key`, the root and its left subtree
    /// belong to the first part, and its right subtree is split recursively:
    /// the smaller half becomes the new right child of the root. The other
    /// case is the mirror image. A node only ever receives as a child a part
    /// of its own subtree, so both the key order and the heap order are kept
    /// without comparing priorities. Each call goes one level down: `O(log n)`
    /// expected time with random priorities.
    pub fn split_off<Q>(node: Link<T, P>, key: &Q) -> (Link<T, P>, Link<T, P>)
    where
        T: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        match node {
            Some(mut n) if n.key.borrow() < key => {
                let (smaller, greater) = TreapNode::split_off(n.right.take(), key);
                n.right = smaller;
                (Some(n), greater)
            }
            Some(mut n) => {
                let (smaller, greater) = TreapNode::split_off(n.left.take(), key);
                n.left = greater;
                (smaller, Some(n))
            }
            None => (None, None),
        }
    }

    /// Join the subtrees `t1` and `t2` into one, if every key of `t1` is
    /// smaller than every key of `t2`.
    ///
    /// The condition is checked once, comparing the greatest key of `t1` with
    /// the smallest key of `t2`, and then [`TreapNode::join`] does the work:
    /// `O(log n)` expected time overall with random priorities.
    ///
    /// # Errors
    ///
    /// [`TreapError::UnjoinableTreaps`] if some key of `t1` is not smaller
    /// than some key of `t2`. Both subtrees are then dropped.
    pub fn try_join<O: Order>(
        t1: Link<T, P>,
        t2: Link<T, P>,
        order: O,
    ) -> Result<Link<T, P>, TreapError> {
        Ok(match (&t1, &t2) {
            // If the greatest key of T1 is not smaller than the smallest key
            // of T2, the trees cannot be joined.
            (Some(t1), Some(t2)) if t1.last() >= t2.first() => Err(TreapError::UnjoinableTreaps)?,
            (_, _) => TreapNode::join(t1, t2, order),
        })
    }

    /// Join `t1` and `t2`, assuming every key of `t1` is smaller than every
    /// key of `t2` (not checked).
    ///
    /// The root that is above the other in `order` becomes the root of the
    /// result. If it is the root of `t1`, its right subtree is joined with
    /// `t2`; otherwise `t1` is joined with the left subtree of `t2`. Each call
    /// goes one level down in one of the trees, along the right spine of `t1`
    /// or the left spine of `t2`, so no rotation is needed.
    /// With distinct priorities the result is the only treap with these keys
    /// and priorities, so joining the two halves of a `split_off` rebuilds the
    /// original tree.
    fn join<O: Order>(t1: Link<T, P>, t2: Link<T, P>, order: O) -> Link<T, P> {
        match (t1, t2) {
            // Nothing to do
            (None, None) => None,
            // Only one of the treaps is something
            (None, Some(t2)) => Some(t2),
            (Some(t1), None) => Some(t1),
            // Both T1 and T2 are something
            (Some(mut t1), Some(mut t2)) => match order.is_above(&t1.priority, &t2.priority) {
                true => {
                    t1.right = TreapNode::join(t1.right, Some(t2), order);
                    Some(t1)
                }
                false => {
                    t2.left = TreapNode::join(Some(t1), t2.left, order);
                    Some(t2)
                }
            },
        }
    }

    /// Return a reference to the key equal to `key` in the subtree, if any.
    ///
    /// `key` may be any borrowed form of the key type, as in the standard
    /// collections: a treap of `String` can be searched with a `&str`. The
    /// borrowed form must be ordered like the key itself.
    pub fn get<Q>(&self, key: &Q) -> Option<&T>
    where
        T: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        let mut node = self;
        loop {
            let here = node.key.borrow();
            let next = if here == key {
                return Some(&node.key);
            } else if here > key {
                &node.left
            } else {
                &node.right
            };
            node = next.as_deref()?;
        }
    }

    /// Return a reference to the smallest key of the subtree.
    ///
    /// It is the key of the leftmost node, found by following the left
    /// children: `O(log n)` expected time with random priorities.
    pub fn first(&self) -> &T {
        let mut node = self;
        while let Some(left) = node.left.as_deref() {
            node = left;
        }
        &node.key
    }

    /// Return a reference to the greatest key of the subtree.
    ///
    /// It is the key of the rightmost node, found by following the right
    /// children: `O(log n)` expected time with random priorities.
    pub fn last(&self) -> &T {
        let mut node = self;
        while let Some(right) = node.right.as_deref() {
            node = right;
        }
        &node.key
    }

    /// Rotate the subtree to the right: the left child becomes its root.
    ///
    /// ```text
    ///       y              x
    ///      / \            / \
    ///     x   C   -->    A   y
    ///    / \                / \
    ///   A   B              B   C
    /// ```
    ///
    /// The contents of `self` and of the child are swapped, so after the call
    /// `self` holds `x` and the old root `y` is its right child.
    ///
    /// # Panics
    ///
    /// If there is no left child.
    fn rotate_right(&mut self) {
        let mut x = self.left.take().expect("Rotate right without a left child");
        self.left = x.right.take();
        swap(self, &mut *x);
        self.right = Some(x);
    }

    /// Rotate the subtree to the left: the right child becomes its root.
    ///
    /// The mirror image of [`TreapNode::rotate_right`]: after the call `self`
    /// holds the old right child and the old root is its left child.
    ///
    /// # Panics
    ///
    /// If there is no right child.
    fn rotate_left(&mut self) {
        let mut y = self
            .right
            .take()
            .expect("Rotate left without a right child");
        self.right = y.left.take();
        swap(self, &mut *y);
        self.left = Some(y);
    }
}

#[cfg(test)]
mod tests;
