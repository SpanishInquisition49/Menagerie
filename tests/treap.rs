//! Integration tests of the public treap API, as a user of the crate sees it.

use std::collections::{BTreeMap, BTreeSet};

use menagerie::treap::{self, MaxHeap, MinHeap, Order, RandomTreap, Treap, TreapError};

/// Deterministic xorshift generator, so that every failure can be reproduced.
struct XorShift(u64);

impl XorShift {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
}

fn treap_of(pairs: &[(i32, u32)]) -> Treap<i32, u32> {
    let mut treap = Treap::new();
    for &(key, priority) in pairs {
        treap.insert(key, priority).unwrap();
    }
    treap
}

fn pairs_of<O>(treap: &Treap<i32, u32, O>) -> Vec<(i32, u32)> {
    treap
        .iter()
        .map(|(&key, &priority)| (key, priority))
        .collect()
}

// ---------------------------------------------------------------------------
// Crate layout
// ---------------------------------------------------------------------------

#[test]
fn main_types_are_reexported_at_the_crate_root() {
    let _: menagerie::Treap<i32, u32> = menagerie::Treap::new();
    let _: menagerie::RandomTreap<i32> = menagerie::RandomTreap::default();
    let random: RandomTreap<i32> = [1].into_iter().collect();
    let _: treap::random::Iter<'_, i32> = random.iter();
    let _: treap::Iter<'_, i32, u32> = Treap::<i32, u32>::new().iter();
}

// ---------------------------------------------------------------------------
// Treap: construction, insert, get, remove
// ---------------------------------------------------------------------------

#[test]
fn a_new_treap_is_empty() {
    let treap: Treap<i32, u32> = Treap::new();
    assert_eq!(treap.get(&1), None);
    assert_eq!(treap.iter().count(), 0);

    let treap: Treap<i32, u32, MaxHeap> = Treap::default();
    assert_eq!(treap.iter().count(), 0);

    let treap: Treap<i32, u32, MaxHeap> = Treap::with_order(MaxHeap);
    assert_eq!(treap.into_iter().count(), 0);
}

#[test]
fn insert_then_get() {
    let treap = treap_of(&[(5, 1), (3, 2), (8, 3)]);
    assert_eq!(treap.get(&5), Some(&5));
    assert_eq!(treap.get(&3), Some(&3));
    assert_eq!(treap.get(&8), Some(&8));
    assert_eq!(treap.get(&4), None);
}

#[test]
fn insert_rejects_a_duplicate_key_even_with_another_priority() {
    let mut treap = treap_of(&[(1, 10), (2, 20)]);
    let before = pairs_of(&treap);

    assert!(matches!(treap.insert(1, 0), Err(TreapError::DuplicateKey)));
    assert_eq!(pairs_of(&treap), before, "the treap must be unchanged");
}

#[test]
fn remove_returns_the_key_once() {
    let mut treap = treap_of(&[(1, 3), (2, 1), (3, 2)]);
    assert_eq!(treap.remove(&2), Some(2));
    assert_eq!(treap.remove(&2), None);
    assert_eq!(treap.get(&2), None);
    assert_eq!(pairs_of(&treap), [(1, 3), (3, 2)]);
}

#[test]
fn remove_from_an_empty_treap() {
    let mut treap: Treap<i32, u32> = Treap::new();
    assert_eq!(treap.remove(&1), None);
}

#[test]
fn lookups_accept_borrowed_keys() {
    let mut treap: Treap<String, u32> = Treap::new();
    treap.insert("rust".to_string(), 1).unwrap();
    treap.insert("go".to_string(), 2).unwrap();

    assert_eq!(treap.get("rust").map(String::as_str), Some("rust"));
    assert_eq!(treap.get("zig"), None);
    assert_eq!(treap.remove("go"), Some("go".to_string()));
    assert_eq!(treap.get("go"), None);
}

#[test]
fn keys_can_be_any_ordered_type() {
    let mut treap: Treap<(u8, char), String> = Treap::new();
    treap.insert((1, 'b'), "x".to_string()).unwrap();
    treap.insert((1, 'a'), "y".to_string()).unwrap();
    treap.insert((0, 'z'), "z".to_string()).unwrap();

    let keys: Vec<_> = treap.iter().map(|(key, _)| *key).collect();
    assert_eq!(keys, [(0, 'z'), (1, 'a'), (1, 'b')]);
}

// ---------------------------------------------------------------------------
// Treap: heap orders
// ---------------------------------------------------------------------------

#[test]
fn orders_are_strict() {
    assert!(MinHeap.is_above(&1, &2));
    assert!(!MinHeap.is_above(&2, &1));
    assert!(!MinHeap.is_above(&2, &2));

    assert!(MaxHeap.is_above(&2, &1));
    assert!(!MaxHeap.is_above(&1, &2));
    assert!(!MaxHeap.is_above(&2, &2));
}

#[test]
fn orders_are_zero_sized() {
    assert_eq!(std::mem::size_of::<MinHeap>(), 0);
    assert_eq!(std::mem::size_of::<MaxHeap>(), 0);
}

#[test]
fn the_order_does_not_change_the_keys() {
    let pairs = [(4, 9), (1, 3), (7, 5), (2, 8)];
    let mut min: Treap<i32, u32> = Treap::new();
    let mut max = Treap::with_order(MaxHeap);
    for (key, priority) in pairs {
        min.insert(key, priority).unwrap();
        max.insert(key, priority).unwrap();
    }
    assert_eq!(pairs_of(&min), pairs_of(&max));
}

// ---------------------------------------------------------------------------
// Treap: split and join
// ---------------------------------------------------------------------------

#[test]
fn split_off_keeps_the_smaller_keys_and_returns_the_others() {
    let mut treap = treap_of(&[(1, 40), (3, 10), (5, 30), (7, 20)]);

    let greater = treap.split_off(&4);

    assert_eq!(pairs_of(&treap), [(1, 40), (3, 10)]);
    assert_eq!(pairs_of(&greater), [(5, 30), (7, 20)]);
}

#[test]
fn split_off_puts_a_present_key_in_the_returned_treap() {
    let mut treap = treap_of(&[(1, 1), (2, 2), (3, 3)]);
    let greater = treap.split_off(&2);
    assert_eq!(pairs_of(&treap), [(1, 1)]);
    assert_eq!(pairs_of(&greater), [(2, 2), (3, 3)]);
}

#[test]
fn split_off_outside_the_key_range() {
    let mut treap = treap_of(&[(1, 1), (2, 2)]);
    let greater = treap.split_off(&0);
    assert_eq!(treap.iter().count(), 0);
    assert_eq!(pairs_of(&greater), [(1, 1), (2, 2)]);

    let mut treap = treap_of(&[(1, 1), (2, 2)]);
    let greater = treap.split_off(&3);
    assert_eq!(pairs_of(&treap), [(1, 1), (2, 2)]);
    assert_eq!(greater.iter().count(), 0);
}

#[test]
fn split_off_an_empty_treap() {
    let mut treap: Treap<i32, u32> = Treap::new();
    let greater = treap.split_off(&1);
    assert_eq!(treap.iter().count() + greater.iter().count(), 0);
}

#[test]
fn split_off_accepts_a_borrowed_key() {
    let mut treap: Treap<String, u32> = Treap::new();
    for (key, priority) in [("a", 1), ("b", 2), ("c", 3)] {
        treap.insert(key.to_string(), priority).unwrap();
    }
    let greater = treap.split_off("b");
    assert!(treap.get("a").is_some() && treap.get("b").is_none());
    assert!(greater.get("b").is_some() && greater.get("c").is_some());
}

#[test]
fn split_off_keeps_the_order() {
    let mut treap = Treap::with_order(MaxHeap);
    for (key, priority) in [(1, 1), (2, 3), (3, 2)] {
        treap.insert(key, priority).unwrap();
    }
    let mut greater = treap.split_off(&0);
    // Still a max-heap: a higher priority must still be accepted and found.
    greater.insert(4, 100).unwrap();
    assert_eq!(greater.get(&4), Some(&4));
}

#[test]
fn join_is_the_inverse_of_split_off() {
    let mut rng = XorShift(0x9e37_79b9);
    for round in 0..20 {
        let mut treap: Treap<i32, u32> = Treap::new();
        for key in 0..200 {
            treap.insert(2 * key, rng.next() as u32).unwrap();
        }
        let before = pairs_of(&treap);
        // Alternately an absent (odd) and a present (even) key.
        let at = 20 * round + round % 2;

        let greater = treap.split_off(&at);
        let joined = Treap::join(treap, greater).unwrap();

        assert_eq!(pairs_of(&joined), before);
    }
}

#[test]
fn join_with_empty_treaps() {
    let treap = treap_of(&[(1, 1), (2, 2)]);
    let joined = Treap::join(treap, Treap::new()).unwrap();
    assert_eq!(pairs_of(&joined), [(1, 1), (2, 2)]);

    let joined = Treap::join(Treap::new(), joined).unwrap();
    assert_eq!(pairs_of(&joined), [(1, 1), (2, 2)]);

    let empty: Treap<i32, u32> = Treap::join(Treap::new(), Treap::new()).unwrap();
    assert_eq!(empty.iter().count(), 0);
}

#[test]
fn join_rejects_overlapping_or_equal_keys() {
    let result = Treap::join(treap_of(&[(1, 1), (5, 2)]), treap_of(&[(3, 3)]));
    assert!(matches!(result, Err(TreapError::UnjoinableTreaps)));

    let result = Treap::join(treap_of(&[(1, 1), (5, 2)]), treap_of(&[(5, 3)]));
    assert!(matches!(result, Err(TreapError::UnjoinableTreaps)));
}

// ---------------------------------------------------------------------------
// Treap: iterators
// ---------------------------------------------------------------------------

#[test]
fn iterators_visit_keys_in_increasing_order() {
    let treap = treap_of(&[(3, 1), (1, 2), (2, 3)]);

    let by_ref: Vec<_> = treap.iter().collect();
    assert_eq!(by_ref, [(&1, &2), (&2, &3), (&3, &1)]);

    let mut by_for = Vec::new();
    for (key, _) in &treap {
        by_for.push(*key);
    }
    assert_eq!(by_for, [1, 2, 3]);

    let by_value: Vec<_> = treap.into_iter().collect();
    assert_eq!(by_value, [(1, 2), (2, 3), (3, 1)]);
}

#[test]
fn into_iter_moves_out_non_copy_keys() {
    let mut treap: Treap<String, u32> = Treap::new();
    treap.insert("b".to_string(), 1).unwrap();
    treap.insert("a".to_string(), 2).unwrap();

    let keys: Vec<String> = treap.into_iter().map(|(key, _)| key).collect();
    assert_eq!(keys, ["a", "b"]);
}

#[test]
fn a_partially_consumed_into_iter_can_be_dropped() {
    let mut treap: Treap<String, u32> = Treap::new();
    for (i, key) in ["d", "b", "a", "c", "e"].into_iter().enumerate() {
        treap.insert(key.to_string(), i as u32).unwrap();
    }
    let mut iter = treap.into_iter();
    assert_eq!(iter.next().map(|(key, _)| key), Some("a".to_string()));
    drop(iter);
}

// ---------------------------------------------------------------------------
// Treap: random operations against BTreeMap
// ---------------------------------------------------------------------------

#[test]
fn random_operations_match_btreemap() {
    let mut rng = XorShift(0x2545_f491);
    let mut treap: Treap<i32, u32, MaxHeap> = Treap::new();
    let mut model = BTreeMap::new();

    for _ in 0..20_000 {
        let key = (rng.next() % 1_000) as i32;
        match rng.next() % 4 {
            0 | 1 => {
                let priority = rng.next() as u32;
                let inserted = treap.insert(key, priority).is_ok();
                assert_eq!(inserted, !model.contains_key(&key));
                model.entry(key).or_insert(priority);
            }
            2 => assert_eq!(treap.remove(&key), model.remove(&key).map(|_| key)),
            _ => assert_eq!(treap.get(&key).is_some(), model.contains_key(&key)),
        }
    }
    let expected: Vec<_> = model.into_iter().collect();
    assert_eq!(pairs_of(&treap), expected);
}

// ---------------------------------------------------------------------------
// RandomTreap
// ---------------------------------------------------------------------------

#[test]
fn random_treap_constructors() {
    let empty: RandomTreap<i32> = RandomTreap::default();
    assert_eq!(empty.iter().count(), 0);

    let one = RandomTreap::new(7);
    assert!(one.iter().eq(&[7]));

    let from: RandomTreap<i32> = RandomTreap::from(7);
    assert!(from.iter().eq(&[7]));

    let collected: RandomTreap<i32> = [3, 1, 2].into_iter().collect();
    assert!(collected.iter().eq(&[1, 2, 3]));
}

#[test]
fn random_treap_skips_duplicates_when_collecting_and_extending() {
    let mut treap: RandomTreap<i32> = [2, 1, 2, 1].into_iter().collect();
    assert!(treap.iter().eq(&[1, 2]));

    treap.extend([3, 2, 4, 3]);
    assert!(treap.iter().eq(&[1, 2, 3, 4]));
}

#[test]
fn random_treap_insert_get_remove() {
    let mut treap = RandomTreap::default();
    treap.insert(10).unwrap();
    treap.insert(5).unwrap();

    assert!(matches!(treap.insert(10), Err(TreapError::DuplicateKey)));
    assert_eq!(treap.get(&5), Some(&5));
    assert_eq!(treap.remove(&5), Some(5));
    assert_eq!(treap.remove(&5), None);
    assert!(treap.iter().eq(&[10]));
}

#[test]
fn random_treap_lookups_accept_borrowed_keys() {
    let mut treap: RandomTreap<String> = ["x", "y"].into_iter().map(String::from).collect();
    assert!(treap.get("x").is_some());
    assert_eq!(treap.remove("y"), Some("y".to_string()));
}

#[test]
fn random_treap_split_off() {
    let mut treap: RandomTreap<i32> = (0..100).map(|key| 2 * key).collect();

    let greater = treap.split_off(&101);

    assert!(treap.iter().copied().eq((0..51).map(|key| 2 * key)));
    assert!(greater.iter().copied().eq((51..100).map(|key| 2 * key)));

    let mut treap: RandomTreap<i32> = (1..=6).collect();
    let greater = treap.split_off(&4);
    assert!(treap.iter().eq(&[1, 2, 3]));
    assert!(greater.iter().eq(&[4, 5, 6]));
}

#[test]
fn random_treap_split_off_matches_btreeset() {
    let mut rng = XorShift(0x0bad_cafe);
    for _ in 0..50 {
        let keys: Vec<u64> = (0..300).map(|_| rng.next() % 1_000).collect();
        let at = rng.next() % 1_100;

        let mut treap: RandomTreap<u64> = keys.iter().copied().collect();
        let mut model: BTreeSet<u64> = keys.iter().copied().collect();
        let greater = treap.split_off(&at);
        let model_greater = model.split_off(&at);

        assert!(treap.iter().eq(model.iter()));
        assert!(greater.iter().eq(model_greater.iter()));
    }
}

#[test]
fn random_treap_iterators() {
    let treap: RandomTreap<i32> = [5, 3, 9, 1].into_iter().collect();

    let by_ref: Vec<_> = (&treap).into_iter().copied().collect();
    assert_eq!(by_ref, [1, 3, 5, 9]);

    let by_value: Vec<_> = treap.into_iter().collect();
    assert_eq!(by_value, [1, 3, 5, 9]);
}

#[test]
fn random_treap_handles_many_sorted_insertions() {
    // Sorted input is the worst case of a plain BST; random priorities keep
    // the depth logarithmic, so the recursive operations do not overflow.
    let mut treap: RandomTreap<u32> = (0..200_000).collect();
    assert_eq!(treap.iter().count(), 200_000);
    for key in (0..200_000).step_by(2) {
        assert_eq!(treap.remove(&key), Some(key));
    }
    assert!(treap.iter().copied().eq((1..200_000).step_by(2)));
}

#[test]
fn random_treap_matches_btreeset() {
    let mut rng = XorShift(0x5851_f42d);
    let mut treap = RandomTreap::default();
    let mut model = BTreeSet::new();

    for _ in 0..20_000 {
        let key = rng.next() % 2_000;
        match rng.next() % 3 {
            0 => assert_eq!(treap.insert(key).is_ok(), model.insert(key)),
            1 => assert_eq!(treap.remove(&key), model.take(&key)),
            _ => assert_eq!(treap.get(&key), model.get(&key)),
        }
    }
    assert!(treap.iter().eq(model.iter()));
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[test]
fn errors_are_displayable_std_errors() {
    fn assert_error<E: std::error::Error + Send + Sync + 'static>(_: &E) {}

    for error in [TreapError::DuplicateKey, TreapError::UnjoinableTreaps] {
        assert_error(&error);
        assert!(!error.to_string().is_empty());
        assert!(!format!("{error:?}").is_empty());
    }
}

#[test]
fn errors_work_with_the_question_mark_operator() {
    fn run() -> Result<(), Box<dyn std::error::Error>> {
        let mut treap = RandomTreap::new(1);
        treap.insert(1)?;
        Ok(())
    }
    assert!(run().is_err());
}
