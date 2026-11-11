//! Unit tests of the node algorithms.
//!
//! They live inside `node` to reach the private fields, so that after every
//! operation the whole tree can be checked: binary search tree order on the
//! keys, heap order on the priorities, no sentinel left behind.

use std::collections::BTreeSet;
use std::fmt::Debug;

use super::{Link, TreapNode};
use crate::treap::TreapError;
use crate::treap::order::{MaxHeap, MinHeap, Order};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Deterministic xorshift generator, so that every failure can be reproduced.
struct XorShift(u64);

impl XorShift {
    fn new(seed: u64) -> Self {
        Self(seed.max(1))
    }

    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }

    fn shuffle<T>(&mut self, items: &mut [T]) {
        for i in (1..items.len()).rev() {
            let j = self.below(i as u64 + 1) as usize;
            items.swap(i, j);
        }
    }
}

/// A pseudo-random priority that is distinct for distinct keys (multiplying
/// by an odd constant is a bijection on `u32`), so the shape of a treap is
/// determined by its keys.
fn priority_of(key: i32) -> u32 {
    (key as u32).wrapping_mul(2_654_435_761)
}

/// Check every invariant of the subtree and return its size.
fn check<T, P, O>(link: &Link<T, P>, order: O) -> usize
where
    T: Ord + Debug,
    P: Ord,
    O: Order,
{
    fn walk<T, P, O>(link: &Link<T, P>, low: Option<&T>, high: Option<&T>, order: O) -> usize
    where
        T: Ord + Debug,
        P: Ord,
        O: Order,
    {
        let Some(node) = link else { return 0 };
        if let Some(low) = low {
            assert!(
                node.key > *low,
                "BST order broken: {:?} not > {:?}",
                node.key,
                low
            );
        }
        if let Some(high) = high {
            assert!(
                node.key < *high,
                "BST order broken: {:?} not < {:?}",
                node.key,
                high
            );
        }
        for child in [&node.left, &node.right].into_iter().flatten() {
            assert!(
                !order.is_above(&child.priority, &node.priority),
                "heap order broken: child {:?} above parent {:?}",
                child.key,
                node.key
            );
        }
        1 + walk(&node.left, low, Some(&node.key), order)
            + walk(&node.right, Some(&node.key), high, order)
    }
    walk(link, None, None, order)
}

/// Pre-order serialisation with explicit empty subtrees: two trees have the
/// same shape, keys and priorities exactly when their shapes are equal.
fn shape(link: &Link<i32, u32>) -> Vec<Option<(i32, u32)>> {
    fn walk(link: &Link<i32, u32>, out: &mut Vec<Option<(i32, u32)>>) {
        match link {
            None => out.push(None),
            Some(node) => {
                out.push(Some((node.key, node.priority)));
                walk(&node.left, out);
                walk(&node.right, out);
            }
        }
    }
    let mut out = Vec::new();
    walk(link, &mut out);
    out
}

/// Keys in increasing order.
fn keys<T: Clone, P>(link: &Link<T, P>) -> Vec<T> {
    super::Iter::new(link).map(|(key, _)| key.clone()).collect()
}

/// Insert one pair into the subtree behind `link`.
fn insert<T, P, O>(link: &mut Link<T, P>, key: T, priority: P, order: O) -> Result<(), TreapError>
where
    T: Ord,
    P: Ord,
    O: Order,
{
    let node = TreapNode::new(key, priority);
    match link {
        Some(root) => root.insert(node, order),
        None => {
            *link = Some(Box::new(node));
            Ok(())
        }
    }
}

/// Build a treap from `(key, priority)` pairs, inserted in the given order.
fn build<O: Order>(pairs: &[(i32, u32)], order: O) -> Link<i32, u32> {
    let mut link = None;
    for &(key, priority) in pairs {
        insert(&mut link, key, priority, order).expect("distinct keys");
    }
    link
}

/// The keys `0..n` with their distinct priorities, in a random order.
fn shuffled_pairs(n: i32, seed: u64) -> Vec<(i32, u32)> {
    let mut pairs: Vec<_> = (0..n).map(|key| (key, priority_of(key))).collect();
    XorShift::new(seed).shuffle(&mut pairs);
    pairs
}

// ---------------------------------------------------------------------------
// Insert
// ---------------------------------------------------------------------------

#[test]
fn insert_keeps_the_invariants_in_both_orders() {
    for seed in 1..=20 {
        let pairs = shuffled_pairs(300, seed);
        let min = build(&pairs, MinHeap);
        let max = build(&pairs, MaxHeap);
        assert_eq!(check(&min, MinHeap), 300);
        assert_eq!(check(&max, MaxHeap), 300);
        assert_eq!(keys(&min), (0..300).collect::<Vec<_>>());
    }
}

#[test]
fn the_root_holds_the_extreme_priority() {
    let pairs = shuffled_pairs(100, 7);
    let lowest = pairs.iter().map(|&(_, p)| p).min().unwrap();
    let highest = pairs.iter().map(|&(_, p)| p).max().unwrap();

    let min = build(&pairs, MinHeap);
    let max = build(&pairs, MaxHeap);
    assert_eq!(min.unwrap().priority, lowest);
    assert_eq!(max.unwrap().priority, highest);
}

#[test]
fn the_shape_does_not_depend_on_the_insertion_order() {
    let reference = shape(&build(&shuffled_pairs(200, 1), MinHeap));
    for seed in 2..=10 {
        assert_eq!(
            shape(&build(&shuffled_pairs(200, seed), MinHeap)),
            reference
        );
    }
}

#[test]
fn insert_rejects_a_duplicate_and_leaves_the_tree_unchanged() {
    let mut link = build(&shuffled_pairs(50, 3), MinHeap);
    let before = shape(&link);

    // Same key, with a priority that would rise to the root.
    let result = insert(&mut link, 25, 0, MinHeap);

    assert!(matches!(result, Err(TreapError::DuplicateKey)));
    assert_eq!(shape(&link), before);
}

#[test]
fn equal_priorities_still_give_a_valid_treap() {
    let mut link = None;
    for key in [5, 1, 9, 3, 7, 2, 8] {
        insert(&mut link, key, 42u32, MinHeap).unwrap();
    }
    assert_eq!(check(&link, MinHeap), 7);
}

#[test]
fn sorted_insertions_with_random_priorities_stay_valid() {
    let pairs: Vec<_> = (0..2_000).map(|key| (key, priority_of(key))).collect();
    let link = build(&pairs, MinHeap);
    assert_eq!(check(&link, MinHeap), 2_000);
}

// ---------------------------------------------------------------------------
// Get, first, last
// ---------------------------------------------------------------------------

#[test]
fn get_finds_present_keys_only() {
    let link = build(&shuffled_pairs(100, 4), MinHeap);
    let root = link.as_ref().unwrap();
    for key in 0..100 {
        assert_eq!(root.get(&key), Some(&key));
    }
    assert_eq!(root.get(&-1), None);
    assert_eq!(root.get(&100), None);
}

#[test]
fn get_accepts_a_borrowed_key() {
    let mut link: Link<String, u32> = None;
    for (key, priority) in [("pear", 3), ("apple", 1), ("kiwi", 2)] {
        insert(&mut link, key.to_string(), priority, MinHeap).unwrap();
    }
    let root = link.as_ref().unwrap();
    assert_eq!(root.get("kiwi").map(String::as_str), Some("kiwi"));
    assert_eq!(root.get("plum"), None);
}

#[test]
fn first_and_last_are_the_extreme_keys() {
    let link = build(&shuffled_pairs(100, 5), MaxHeap);
    let root = link.as_ref().unwrap();
    assert_eq!(*root.first(), 0);
    assert_eq!(*root.last(), 99);

    let single = build(&[(7, 1)], MinHeap);
    assert_eq!(*single.as_ref().unwrap().first(), 7);
    assert_eq!(*single.as_ref().unwrap().last(), 7);
}

// ---------------------------------------------------------------------------
// Remove
// ---------------------------------------------------------------------------

#[test]
fn remove_from_an_empty_tree_returns_none() {
    let mut link: Link<i32, u32> = None;
    assert_eq!(TreapNode::remove(&mut link, &1, MinHeap), None);
}

#[test]
fn remove_a_missing_key_leaves_the_tree_unchanged() {
    let mut link = build(&shuffled_pairs(50, 6), MinHeap);
    let before = shape(&link);
    assert_eq!(TreapNode::remove(&mut link, &1_000, MinHeap), None);
    assert_eq!(TreapNode::remove(&mut link, &-1, MinHeap), None);
    assert_eq!(shape(&link), before);
}

#[test]
fn remove_the_only_node_empties_the_tree() {
    let mut link = build(&[(1, 1)], MinHeap);
    assert_eq!(TreapNode::remove(&mut link, &1, MinHeap), Some(1));
    assert!(link.is_none());
}

#[test]
fn remove_a_leaf_a_node_with_one_child_and_one_with_two() {
    // MinHeap: 4 (1) is the root, 2 (2) and 6 (3) its children,
    // 1 (4) a leaf under 2, and 6 has two children 5 (5) and 7 (6).
    let pairs = [(4, 1), (2, 2), (6, 3), (1, 4), (5, 5), (7, 6)];

    for key in [1, 2, 6, 4] {
        let mut link = build(&pairs, MinHeap);
        assert_eq!(TreapNode::remove(&mut link, &key, MinHeap), Some(key));
        assert_eq!(check(&link, MinHeap), 5);
        assert!(!keys(&link).contains(&key));
    }
}

#[test]
fn removing_every_key_in_random_order_keeps_the_invariants() {
    for order_seed in 1..=5 {
        let mut link = build(&shuffled_pairs(300, 9), MaxHeap);
        let mut victims: Vec<i32> = (0..300).collect();
        XorShift::new(order_seed).shuffle(&mut victims);

        for (removed, key) in victims.into_iter().enumerate() {
            assert_eq!(TreapNode::remove(&mut link, &key, MaxHeap), Some(key));
            assert_eq!(check(&link, MaxHeap), 300 - removed - 1);
        }
        assert!(link.is_none());
    }
}

#[test]
fn remove_accepts_a_borrowed_key() {
    let mut link: Link<String, u32> = None;
    insert(&mut link, "a".to_string(), 1, MinHeap).unwrap();
    insert(&mut link, "b".to_string(), 2, MinHeap).unwrap();
    assert_eq!(
        TreapNode::remove(&mut link, "a", MinHeap),
        Some("a".to_string())
    );
    assert_eq!(keys(&link), ["b"]);
}

#[test]
fn insert_after_remove_restores_the_same_shape() {
    let pairs = shuffled_pairs(200, 11);
    let mut link = build(&pairs, MinHeap);
    let before = shape(&link);

    TreapNode::remove(&mut link, &100, MinHeap).unwrap();
    insert(&mut link, 100, priority_of(100), MinHeap).unwrap();

    assert_eq!(shape(&link), before);
}

// ---------------------------------------------------------------------------
// Split off
// ---------------------------------------------------------------------------

#[test]
fn split_off_an_empty_tree_gives_two_empty_trees() {
    let (smaller, greater) = TreapNode::<i32, u32>::split_off(None, &5);
    assert!(smaller.is_none() && greater.is_none());
}

#[test]
fn split_off_partitions_the_keys_and_keeps_the_invariants() {
    // Present keys, absent keys and keys outside the range.
    for at in [-10, 0, 1, 37, 50, 99, 100, 150] {
        for (order_seed, max_heap) in [(12, false), (13, true)] {
            let pairs = shuffled_pairs(100, order_seed);
            let (smaller, greater) = if max_heap {
                let (smaller, greater) = TreapNode::split_off(build(&pairs, MaxHeap), &at);
                assert_eq!(check(&smaller, MaxHeap) + check(&greater, MaxHeap), 100);
                (smaller, greater)
            } else {
                let (smaller, greater) = TreapNode::split_off(build(&pairs, MinHeap), &at);
                assert_eq!(check(&smaller, MinHeap) + check(&greater, MinHeap), 100);
                (smaller, greater)
            };
            assert_eq!(keys(&smaller), (0..at.clamp(0, 100)).collect::<Vec<_>>());
            assert_eq!(keys(&greater), (at.clamp(0, 100)..100).collect::<Vec<_>>());
        }
    }
}

#[test]
fn split_off_puts_a_present_key_in_the_second_part() {
    let (smaller, greater) = TreapNode::split_off(build(&shuffled_pairs(10, 3), MinHeap), &4);
    assert_eq!(keys(&smaller), [0, 1, 2, 3]);
    assert_eq!(keys(&greater), [4, 5, 6, 7, 8, 9]);
}

#[test]
fn split_off_keeps_the_left_subtree_of_a_greater_root() {
    // Regression: when the root goes to the second part, the greater half of
    // its left subtree must become its new left child, not be dropped.
    let tree = build(&[(10, 1), (7, 2)], MinHeap);

    let (smaller, greater) = TreapNode::split_off(tree, &5);

    assert!(smaller.is_none());
    assert_eq!(keys(&greater), [7, 10]);
    assert_eq!(check(&greater, MinHeap), 2);
}

#[test]
fn split_off_accepts_a_borrowed_key() {
    let mut link: Link<String, u32> = None;
    for (key, priority) in [("pear", 3), ("apple", 1), ("kiwi", 2)] {
        insert(&mut link, key.to_string(), priority, MinHeap).unwrap();
    }
    let (smaller, greater) = TreapNode::split_off(link, "kiwi");
    assert_eq!(keys(&smaller), ["apple"]);
    assert_eq!(keys(&greater), ["kiwi", "pear"]);
}

// ---------------------------------------------------------------------------
// Join
// ---------------------------------------------------------------------------

#[test]
fn join_with_an_empty_side_returns_the_other() {
    let tree = build(&shuffled_pairs(20, 14), MinHeap);
    let before = shape(&tree);

    let joined = TreapNode::try_join(tree, None, MinHeap).unwrap();
    assert_eq!(shape(&joined), before);

    let joined = TreapNode::try_join(None, joined, MinHeap).unwrap();
    assert_eq!(shape(&joined), before);

    let empty: Link<i32, u32> = TreapNode::try_join(None, None, MinHeap).unwrap();
    assert!(empty.is_none());
}

#[test]
fn join_rejects_overlapping_or_touching_trees() {
    let overlapping = TreapNode::try_join(
        build(&[(1, 1), (5, 2)], MinHeap),
        build(&[(3, 3), (9, 4)], MinHeap),
        MinHeap,
    );
    assert!(matches!(overlapping, Err(TreapError::UnjoinableTreaps)));

    // The greatest key of the first equals the smallest of the second.
    let touching = TreapNode::try_join(
        build(&[(1, 1), (5, 2)], MinHeap),
        build(&[(5, 3), (9, 4)], MinHeap),
        MinHeap,
    );
    assert!(matches!(touching, Err(TreapError::UnjoinableTreaps)));

    let swapped = TreapNode::try_join(
        build(&[(10, 1)], MinHeap),
        build(&[(1, 2)], MinHeap),
        MinHeap,
    );
    assert!(matches!(swapped, Err(TreapError::UnjoinableTreaps)));
}

#[test]
fn join_when_the_second_root_wins() {
    // Regression: in this case the recursive call once swapped its
    // arguments, which put key 1 to the right of key 2.
    let first = build(&[(1, 5)], MinHeap);
    let second = build(&[(3, 1), (2, 3)], MinHeap);

    let joined = TreapNode::try_join(first, second, MinHeap).unwrap();

    assert_eq!(check(&joined, MinHeap), 3);
    assert_eq!(keys(&joined), [1, 2, 3]);
}

#[test]
fn split_off_then_join_rebuilds_exactly_the_original_tree() {
    // Keys 0, 2, 4, ...: both present (even) and absent (odd) split keys.
    for (seed, at) in [(1, 100), (2, 0), (3, 399), (4, 247), (5, 154), (6, 400)] {
        let pairs: Vec<_> = shuffled_pairs(200, seed)
            .into_iter()
            .map(|(key, priority)| (2 * key, priority))
            .collect();

        let min = build(&pairs, MinHeap);
        let before = shape(&min);
        let (smaller, greater) = TreapNode::split_off(min, &at);
        let joined = TreapNode::try_join(smaller, greater, MinHeap).unwrap();
        assert_eq!(shape(&joined), before);

        let max = build(&pairs, MaxHeap);
        let before = shape(&max);
        let (smaller, greater) = TreapNode::split_off(max, &at);
        let joined = TreapNode::try_join(smaller, greater, MaxHeap).unwrap();
        assert_eq!(shape(&joined), before);
    }
}

#[test]
fn join_of_independent_trees_keeps_the_invariants() {
    let low: Vec<_> = shuffled_pairs(100, 15);
    let high: Vec<_> = shuffled_pairs(100, 16)
        .into_iter()
        .map(|(key, _)| (key + 1_000, priority_of(key + 1_000)))
        .collect();

    let joined = TreapNode::try_join(build(&low, MaxHeap), build(&high, MaxHeap), MaxHeap).unwrap();

    assert_eq!(check(&joined, MaxHeap), 200);
    let expected: Vec<_> = (0..100).chain(1_000..1_100).collect();
    assert_eq!(keys(&joined), expected);
}

// ---------------------------------------------------------------------------
// Iterators
// ---------------------------------------------------------------------------

#[test]
fn iterators_yield_sorted_pairs_with_their_priorities() {
    let pairs = shuffled_pairs(100, 17);
    let link = build(&pairs, MinHeap);

    let mut expected = pairs.clone();
    expected.sort();

    let by_ref: Vec<_> = super::Iter::new(&link).map(|(&k, &p)| (k, p)).collect();
    assert_eq!(by_ref, expected);

    let by_value: Vec<_> = super::IntoIter::new(link).collect();
    assert_eq!(by_value, expected);
}

#[test]
fn iterators_of_an_empty_tree_are_empty() {
    let link: Link<i32, u32> = None;
    assert_eq!(super::Iter::new(&link).count(), 0);
    assert_eq!(super::IntoIter::new(link).count(), 0);
}

// ---------------------------------------------------------------------------
// Random operations against BTreeSet
// ---------------------------------------------------------------------------

fn differential<O: Order>(order: O, seed: u64) {
    let mut rng = XorShift::new(seed);
    let mut link: Link<i32, u32> = None;
    let mut model = BTreeSet::new();

    for _ in 0..3_000 {
        let key = rng.below(400) as i32;
        match rng.below(4) {
            0 | 1 => {
                let inserted = insert(&mut link, key, rng.next() as u32, order).is_ok();
                assert_eq!(inserted, model.insert(key));
            }
            2 => assert_eq!(TreapNode::remove(&mut link, &key, order), model.take(&key)),
            _ => {
                let found = link.as_ref().and_then(|root| root.get(&key));
                assert_eq!(found, model.get(&key));
            }
        }
        assert_eq!(check(&link, order), model.len());
    }
    assert_eq!(keys(&link), model.into_iter().collect::<Vec<_>>());
}

#[test]
fn random_operations_match_btreeset_min_heap() {
    for seed in 1..=5 {
        differential(MinHeap, seed);
    }
}

#[test]
fn random_operations_match_btreeset_max_heap() {
    for seed in 6..=10 {
        differential(MaxHeap, seed);
    }
}
