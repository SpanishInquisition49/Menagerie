//! A collection of data structures, classic and randomized.
//!
//! At the moment the crate provides treaps, in the [`treap`] module: a
//! generic [`Treap`], where the caller chooses every priority, and a
//! [`RandomTreap`], which draws the priorities at random. The main type of
//! every data structure is also re-exported at the crate root.
//!
//! # Examples
//!
//! ```
//! use menagerie::RandomTreap;
//!
//! let mut treap: RandomTreap<&str> = ["b", "c", "a"].into_iter().collect();
//! assert_eq!(treap.remove("b"), Some("b"));
//! assert!(treap.iter().eq(&["a", "c"]));
//! ```

pub mod treap;

pub use treap::{RandomTreap, Treap};

// Compile and run the examples of the README as doctests.
#[doc = include_str!("../README.md")]
#[cfg(doctest)]
pub struct ReadmeDoctests;
