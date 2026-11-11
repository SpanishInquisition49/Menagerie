//! Treap with random priorities.

use rand::Rng;

use std::borrow::Borrow;

use super::error::TreapError;
use super::generic::{self, Treap};
use super::order::MinHeap;

/// A treap that draws a random `u32` priority for every inserted key.
///
/// With random priorities the tree has the shape of a binary search tree
/// built by inserting the keys in random order, whatever the real order of the
/// insertions: search, insertion, deletion and splitting take `O(log n)` expected
/// time.
///
/// It is a thin wrapper around a [`Treap`] with `u32` priorities in
/// [`MinHeap`] order. Duplicate keys are not supported.
///
/// # Examples
///
/// ```
/// use menagerie::treap::RandomTreap;
///
/// let mut treap: RandomTreap<i32> = (1..=10).collect();
///
/// assert_eq!(treap.get(&4), Some(&4));
/// assert!(treap.insert(4).is_err());
/// assert_eq!(treap.remove(&4), Some(4));
/// assert_eq!(treap.get(&4), None);
/// ```
pub struct RandomTreap<T> {
    treap: Treap<T, u32, MinHeap>,
}

impl<T> Default for RandomTreap<T>
where
    T: Ord,
{
    /// An empty treap.
    fn default() -> Self {
        Self {
            treap: Treap::new(),
        }
    }
}

impl<T> From<T> for RandomTreap<T>
where
    T: Ord,
{
    /// A treap holding only `key`; same as [`RandomTreap::new`].
    fn from(key: T) -> Self {
        Self::new(key)
    }
}

impl<T> Extend<T> for RandomTreap<T>
where
    T: Ord,
{
    /// Insert every key of `iter`.
    ///
    /// Keys already in the treap are silently skipped: `Extend` cannot return
    /// errors. Use [`RandomTreap::insert`] to detect them.
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for key in iter {
            let _ = self.insert(key);
        }
    }
}

impl<T> FromIterator<T> for RandomTreap<T>
where
    T: Ord,
{
    /// Build a treap from the keys of `iter`.
    ///
    /// Duplicate keys are silently skipped, as in [`Extend`].
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut treap = Self::default();
        treap.extend(iter);
        treap
    }
}

impl<T: Ord> RandomTreap<T> {
    /// Create a treap holding only `key`.
    ///
    /// For an empty treap use [`RandomTreap::default`].
    pub fn new(key: T) -> Self {
        let mut treap = Self::default();
        // An empty treap cannot contain a duplicate, so this cannot fail.
        let _ = treap.insert(key);
        treap
    }

    /// Return a reference to the key equal to `key`, or `None` if it is not
    /// in the treap.
    ///
    /// `key` may be any borrowed form of the key type, as in [`Treap::get`].
    pub fn get<Q>(&self, key: &Q) -> Option<&T>
    where
        T: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.treap.get(key)
    }

    /// Insert `key` with a fresh random priority.
    ///
    /// # Errors
    ///
    /// [`TreapError::DuplicateKey`] if `key` is already in the treap. The
    /// treap is then left unchanged.
    pub fn insert(&mut self, key: T) -> Result<(), TreapError> {
        self.treap.insert(key, rand::rng().next_u32())
    }

    /// Remove the key equal to `key` from the treap and return it, or `None`
    /// if it is not there.
    ///
    /// `key` may be any borrowed form of the key type, as in [`Treap::get`].
    pub fn remove<Q>(&mut self, key: &Q) -> Option<T>
    where
        T: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.treap.remove(key)
    }

    /// Split the treap in two at `key`: `self` keeps the keys smaller than
    /// `key`, and the keys greater than or equal to `key` are returned in a
    /// new treap. As in `BTreeSet::split_off`, `key` itself, if present, goes
    /// to the returned treap.
    ///
    /// `key` may be any borrowed form of the key type, as in [`Treap::get`].
    ///
    /// # Examples
    ///
    /// ```
    /// use menagerie::treap::RandomTreap;
    ///
    /// let mut treap: RandomTreap<i32> = (1..=6).collect();
    /// let greater = treap.split_off(&4);
    ///
    /// assert!(treap.iter().eq(&[1, 2, 3]));
    /// assert!(greater.iter().eq(&[4, 5, 6]));
    /// ```
    pub fn split_off<Q>(&mut self, key: &Q) -> Self
    where
        T: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        Self {
            treap: self.treap.split_off(key),
        }
    }
}

impl<T> RandomTreap<T> {
    /// An iterator over the keys, in increasing order.
    ///
    /// The random priorities are an implementation detail and are not
    /// returned. Visiting the whole treap takes `O(n)` time.
    ///
    /// # Examples
    ///
    /// ```
    /// use menagerie::treap::RandomTreap;
    ///
    /// let treap: RandomTreap<i32> = [3, 1, 2].into_iter().collect();
    ///
    /// assert!(treap.iter().eq(&[1, 2, 3]));
    /// ```
    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            inner: self.treap.iter(),
        }
    }
}

/// Iterator over the keys of a [`RandomTreap`], by reference, in increasing
/// order.
///
/// Created by [`RandomTreap::iter`].
pub struct Iter<'a, T> {
    inner: generic::Iter<'a, T, u32>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|(key, _)| key)
    }
}

/// Iterator over the keys of a [`RandomTreap`], by value, in increasing
/// order.
///
/// Created by the [`IntoIterator`] implementation of [`RandomTreap`], which
/// consumes the treap.
pub struct IntoIter<T> {
    inner: generic::IntoIter<T, u32>,
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|(key, _)| key)
    }
}

impl<'a, T> IntoIterator for &'a RandomTreap<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    /// Same as [`RandomTreap::iter`]: enables `for key in &treap`.
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<T> IntoIterator for RandomTreap<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    /// Consume the treap and yield its keys in increasing order.
    ///
    /// # Examples
    ///
    /// ```
    /// use menagerie::treap::RandomTreap;
    ///
    /// let treap: RandomTreap<i32> = [3, 1, 2].into_iter().collect();
    ///
    /// assert_eq!(treap.into_iter().collect::<Vec<_>>(), [1, 2, 3]);
    /// ```
    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            inner: self.treap.into_iter(),
        }
    }
}
