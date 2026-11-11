//! Generic treap: the caller chooses the key, the priority and the heap order.

use std::borrow::Borrow;

use super::error::TreapError;
use super::node::{self, Link, TreapNode};
use super::order::{MinHeap, Order};

/// A treap with keys of type `T`, priorities of type `P` and heap order `O`.
///
/// The keys are kept in binary search tree order and the priorities in heap
/// order, so the shape of the tree depends only on the keys and the
/// priorities, not on the order of the insertions. With random priorities the
/// expected depth is `O(log n)`; see [`RandomTreap`] for a treap that draws
/// them itself.
///
/// `O` defaults to [`MinHeap`], so `Treap<T, P>` is `Treap<T, P, MinHeap>`.
/// Duplicate keys are not supported.
///
/// [`RandomTreap`]: super::RandomTreap
///
/// # Examples
///
/// ```
/// use menagerie::treap::{MaxHeap, Treap};
///
/// let mut treap = Treap::with_order(MaxHeap);
/// treap.insert("b", 10).unwrap();
/// treap.insert("a", 20).unwrap();
///
/// assert_eq!(treap.get("a"), Some(&"a"));
/// assert!(treap.insert("a", 30).is_err());
/// ```
pub struct Treap<T, P, O = MinHeap> {
    root: Link<T, P>,
    order: O,
}

impl<T, P, O> Default for Treap<T, P, O>
where
    T: Ord,
    P: Ord,
    O: Order,
{
    /// An empty treap with the default order.
    fn default() -> Self {
        Self::with_order(O::default())
    }
}

impl<T, P, O> Treap<T, P, O>
where
    T: Ord,
    P: Ord,
    O: Order,
{
    /// Create an empty treap with the default order.
    ///
    /// The order comes from the type, so it may need an annotation:
    ///
    /// ```
    /// use menagerie::treap::{MaxHeap, Treap};
    ///
    /// let treap: Treap<i32, u32, MaxHeap> = Treap::new();
    /// assert_eq!(treap.get(&1), None);
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Create an empty treap with the given order.
    pub fn with_order(order: O) -> Self {
        Self { root: None, order }
    }

    /// Return a reference to the key equal to `key`, or `None` if it is not
    /// in the treap.
    ///
    /// `key` may be any borrowed form of the key type, as in the standard
    /// collections, but it must be ordered like the key itself. Expected time
    /// `O(log n)` with random priorities.
    ///
    /// # Examples
    ///
    /// ```
    /// use menagerie::treap::Treap;
    ///
    /// let mut treap: Treap<String, u32> = Treap::new();
    /// treap.insert("rust".to_string(), 1).unwrap();
    ///
    /// assert_eq!(treap.get("rust").map(String::as_str), Some("rust"));
    /// assert_eq!(treap.get("go"), None);
    /// ```
    pub fn get<Q>(&self, key: &Q) -> Option<&T>
    where
        T: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.root.as_ref()?.get(key)
    }

    /// Insert `key` with the given `priority`.
    ///
    /// Expected time `O(log n)` with random priorities.
    ///
    /// # Errors
    ///
    /// [`TreapError::DuplicateKey`] if `key` is already in the treap, even
    /// with a different priority. The treap is then left unchanged.
    pub fn insert(&mut self, key: T, priority: P) -> Result<(), TreapError> {
        let node = TreapNode::new(key, priority);
        match &mut self.root {
            Some(root) => root.insert(node, self.order),
            None => {
                self.root = Some(Box::new(node));
                Ok(())
            }
        }
    }

    /// Remove the key equal to `key` from the treap and return it, or `None`
    /// if it is not there.
    ///
    /// `key` may be any borrowed form of the key type, as in [`Treap::get`].
    /// Expected time `O(log n)` with random priorities.
    ///
    /// # Examples
    ///
    /// ```
    /// use menagerie::treap::Treap;
    ///
    /// let mut treap: Treap<i32, u32> = Treap::new();
    /// treap.insert(1, 7).unwrap();
    ///
    /// assert_eq!(treap.remove(&1), Some(1));
    /// assert_eq!(treap.remove(&1), None);
    /// ```
    pub fn remove<Q>(&mut self, key: &Q) -> Option<T>
    where
        T: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        TreapNode::remove(&mut self.root, key, self.order)
    }

    /// Split the treap in two at `key`: `self` keeps the keys smaller than
    /// `key`, and the keys greater than or equal to `key` are returned in a
    /// new treap with the same order. As in `BTreeSet::split_off`, `key`
    /// itself, if present, goes to the returned treap.
    ///
    /// `key` may be any borrowed form of the key type, as in [`Treap::get`].
    /// Expected time `O(log n)` with random priorities.
    ///
    /// # Examples
    ///
    /// ```
    /// use menagerie::treap::Treap;
    ///
    /// let mut treap: Treap<i32, u32> = Treap::new();
    /// for (key, priority) in [(1, 40), (3, 10), (5, 30)] {
    ///     treap.insert(key, priority).unwrap();
    /// }
    ///
    /// let greater = treap.split_off(&3);
    /// assert!(treap.get(&1).is_some() && treap.get(&3).is_none());
    /// assert!(greater.get(&3).is_some() && greater.get(&5).is_some());
    /// ```
    pub fn split_off<Q>(&mut self, key: &Q) -> Self
    where
        T: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        let (smaller, greater) = TreapNode::split_off(self.root.take(), key);
        self.root = smaller;
        Self {
            root: greater,
            order: self.order,
        }
    }

    /// Join two treaps into one: the inverse of [`Treap::split_off`].
    ///
    /// Every key of `t1` must be smaller than every key of `t2`. The result
    /// has the order of `t1` (the same as `t2`, since the order is part of
    /// the type). Expected time `O(log n)` with random priorities.
    ///
    /// With distinct priorities, joining a treap with the one split off from
    /// it by [`Treap::split_off`] rebuilds exactly the original tree.
    ///
    /// # Errors
    ///
    /// [`TreapError::UnjoinableTreaps`] if some key of `t1` is not smaller
    /// than some key of `t2`. Both treaps are consumed, so they are then
    /// lost.
    ///
    /// # Examples
    ///
    /// ```
    /// use menagerie::treap::Treap;
    ///
    /// let mut treap: Treap<i32, u32> = Treap::new();
    /// for (key, priority) in [(1, 40), (3, 10), (5, 30), (7, 5)] {
    ///     treap.insert(key, priority).unwrap();
    /// }
    ///
    /// let greater = treap.split_off(&4);
    /// let joined = Treap::join(treap, greater).unwrap();
    /// let keys: Vec<_> = joined.iter().map(|(key, _)| *key).collect();
    /// assert_eq!(keys, [1, 3, 5, 7]);
    ///
    /// let (mut low, mut high) = (Treap::new(), Treap::new());
    /// low.insert(5, 1).unwrap();
    /// high.insert(2, 2).unwrap();
    /// assert!(Treap::<i32, u32>::join(low, high).is_err());
    /// ```
    pub fn join(t1: Self, t2: Self) -> Result<Self, TreapError> {
        let joined = TreapNode::try_join(t1.root, t2.root, t1.order)?;
        Ok(Self {
            root: joined,
            order: t1.order,
        })
    }
}

impl<T, P, O> Treap<T, P, O> {
    /// An iterator over the keys and their priorities, in increasing key
    /// order.
    ///
    /// Visiting the whole treap takes `O(n)` time.
    ///
    /// # Examples
    ///
    /// ```
    /// use menagerie::treap::Treap;
    ///
    /// let mut treap: Treap<char, u32> = Treap::new();
    /// for (key, priority) in [('c', 1), ('a', 3), ('b', 2)] {
    ///     treap.insert(key, priority).unwrap();
    /// }
    ///
    /// let pairs: Vec<_> = treap.iter().collect();
    /// assert_eq!(pairs, [(&'a', &3), (&'b', &2), (&'c', &1)]);
    /// ```
    pub fn iter(&self) -> Iter<'_, T, P> {
        Iter {
            inner: node::Iter::new(&self.root),
        }
    }
}

/// Iterator over the keys and priorities of a [`Treap`], by reference, in
/// increasing key order.
///
/// Created by [`Treap::iter`].
pub struct Iter<'a, T, P> {
    inner: node::Iter<'a, T, P>,
}

impl<'a, T, P> Iterator for Iter<'a, T, P> {
    type Item = (&'a T, &'a P);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

/// Iterator over the keys and priorities of a [`Treap`], by value, in
/// increasing key order.
///
/// Created by the [`IntoIterator`] implementation of [`Treap`], which
/// consumes the treap.
pub struct IntoIter<T, P> {
    inner: node::IntoIter<T, P>,
}

impl<T, P> Iterator for IntoIter<T, P> {
    type Item = (T, P);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

impl<'a, T, P, O> IntoIterator for &'a Treap<T, P, O> {
    type Item = (&'a T, &'a P);
    type IntoIter = Iter<'a, T, P>;

    /// Same as [`Treap::iter`]: enables `for (key, priority) in &treap`.
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<T, P, O> IntoIterator for Treap<T, P, O> {
    type Item = (T, P);
    type IntoIter = IntoIter<T, P>;

    /// Consume the treap and yield its keys and priorities, in increasing key
    /// order.
    ///
    /// # Examples
    ///
    /// ```
    /// use menagerie::treap::Treap;
    ///
    /// let mut treap: Treap<String, u32> = Treap::new();
    /// treap.insert("b".to_string(), 1).unwrap();
    /// treap.insert("a".to_string(), 2).unwrap();
    ///
    /// let keys: Vec<String> = treap.into_iter().map(|(key, _)| key).collect();
    /// assert_eq!(keys, ["a", "b"]);
    /// ```
    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            inner: node::IntoIter::new(self.root),
        }
    }
}
