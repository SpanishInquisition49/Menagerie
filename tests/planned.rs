//! Tests for the API planned in `TODO.md` but not implemented yet.
//!
//! Every section is compiled only when its `planned` cfg value is set, so the
//! file builds today and each feature can be switched on when it lands:
//!
//! ```text
//! RUSTFLAGS='--cfg planned="size"' cargo test --test planned
//! ```
//!
//! The method names and signatures below are a proposal, modelled on
//! `BTreeSet` / `BTreeMap`: adjust them to the final design. When a section
//! passes, move it to `tests/treap.rs` and drop its cfg value from
//! `Cargo.toml`.
//!
//! Sections: `size`, `ends`, `range`, `retain`, `std_insert`,
//! `join_recovery`, `traits`, `iterators`, `deep`, `random_join`, `set_ops`,
//! `order_stats`, `map`.

/// `len`, `is_empty`, `contains`, `clear`.
#[cfg(planned = "size")]
mod size {
    use menagerie::{RandomTreap, Treap};

    #[test]
    fn len_and_is_empty_follow_inserts_and_removes() {
        let mut treap: RandomTreap<i32> = RandomTreap::default();
        assert!(treap.is_empty());
        assert_eq!(treap.len(), 0);

        treap.extend([3, 1, 2, 2]);
        assert_eq!(treap.len(), 3);
        assert!(!treap.is_empty());

        treap.remove(&1);
        treap.remove(&1);
        assert_eq!(treap.len(), 2);
    }

    #[test]
    fn len_after_split_off() {
        let mut treap: RandomTreap<i32> = (0..100).map(|key| 2 * key).collect();
        let greater = treap.split_off(&51);
        assert_eq!(treap.len(), 26);
        assert_eq!(greater.len(), 74);
    }

    #[test]
    fn len_of_the_generic_treap() {
        let mut treap: Treap<i32, u32> = Treap::new();
        treap.insert(1, 1).unwrap();
        let _ = treap.insert(1, 2);
        assert_eq!(treap.len(), 1);
    }

    #[test]
    fn contains_and_clear() {
        let mut treap: RandomTreap<String> = ["a", "b"].into_iter().map(String::from).collect();
        assert!(treap.contains("a"));
        assert!(!treap.contains("c"));

        treap.clear();
        assert!(treap.is_empty());
        assert!(!treap.contains("a"));
    }
}

/// `first`, `last`, `pop_first`, `pop_last`.
#[cfg(planned = "ends")]
mod ends {
    use menagerie::RandomTreap;

    #[test]
    fn first_and_last() {
        let treap: RandomTreap<i32> = [5, 1, 9, 3].into_iter().collect();
        assert_eq!(treap.first(), Some(&1));
        assert_eq!(treap.last(), Some(&9));

        let empty: RandomTreap<i32> = RandomTreap::default();
        assert_eq!(empty.first(), None);
        assert_eq!(empty.last(), None);
    }

    #[test]
    fn pop_first_and_pop_last_drain_in_order() {
        let mut treap: RandomTreap<i32> = (1..=5).collect();
        assert_eq!(treap.pop_first(), Some(1));
        assert_eq!(treap.pop_last(), Some(5));
        assert!(treap.iter().eq(&[2, 3, 4]));

        let mut drained = Vec::new();
        while let Some(key) = treap.pop_first() {
            drained.push(key);
        }
        assert_eq!(drained, [2, 3, 4]);
        assert_eq!(treap.pop_last(), None);
    }
}

/// `range` over any `RangeBounds`.
#[cfg(planned = "range")]
mod range {
    use std::collections::BTreeSet;
    use std::ops::Bound;

    use menagerie::RandomTreap;

    #[test]
    fn range_matches_btreeset() {
        let keys = [1, 4, 6, 8, 10, 15, 20];
        let treap: RandomTreap<i32> = keys.into_iter().collect();
        let model: BTreeSet<i32> = keys.into_iter().collect();

        assert!(treap.range(4..10).eq(model.range(4..10)));
        assert!(treap.range(4..=10).eq(model.range(4..=10)));
        assert!(treap.range(..8).eq(model.range(..8)));
        assert!(treap.range(9..).eq(model.range(9..)));
        assert!(treap.range(..).eq(model.range(..)));
        assert!(treap.range(11..15).eq(model.range(11..15)));
        let bounds = (Bound::Excluded(4), Bound::Excluded(15));
        assert!(treap.range(bounds).eq(model.range(bounds)));
    }

    #[test]
    fn range_with_borrowed_bounds() {
        let treap: RandomTreap<String> =
            ["a", "b", "c", "d"].into_iter().map(String::from).collect();
        let keys: Vec<_> = treap
            .range::<str, _>("b".."d")
            .map(String::as_str)
            .collect();
        assert_eq!(keys, ["b", "c"]);
    }
}

/// `retain`.
#[cfg(planned = "retain")]
mod retain {
    use menagerie::RandomTreap;

    #[test]
    fn retain_keeps_only_matching_keys() {
        let mut treap: RandomTreap<i32> = (0..20).collect();
        treap.retain(|key| key % 3 == 0);
        assert!(treap.iter().copied().eq((0..20).filter(|key| key % 3 == 0)));
    }
}

/// `insert` with the `std` semantics: `false` instead of an error on a
/// duplicate. The current `tests/treap.rs` checks for `Err`, update it too.
#[cfg(planned = "std_insert")]
mod std_insert {
    use menagerie::RandomTreap;

    #[test]
    fn insert_returns_whether_the_key_was_new() {
        let mut treap = RandomTreap::default();
        assert!(treap.insert(1));
        assert!(!treap.insert(1));
        assert!(treap.iter().eq(&[1]));
    }
}

/// A failed `join` gives both treaps back instead of dropping them.
#[cfg(planned = "join_recovery")]
mod join_recovery {
    use menagerie::Treap;

    #[test]
    fn failed_join_returns_both_treaps() {
        let mut low: Treap<i32, u32> = Treap::new();
        let mut high: Treap<i32, u32> = Treap::new();
        low.insert(5, 1).unwrap();
        high.insert(2, 2).unwrap();

        let error = Treap::join(low, high).unwrap_err();
        let (low, high) = error.into_parts();

        assert_eq!(low.get(&5), Some(&5));
        assert_eq!(high.get(&2), Some(&2));
    }
}

/// `Debug`, `Clone`, `PartialEq`, `Eq` on the treaps and on `TreapError`.
#[cfg(planned = "traits")]
mod traits {
    use menagerie::treap::TreapError;
    use menagerie::{RandomTreap, Treap};

    #[test]
    fn debug_prints_the_keys() {
        let treap: RandomTreap<i32> = [2, 1].into_iter().collect();
        let printed = format!("{treap:?}");
        assert!(printed.contains('1') && printed.contains('2'));
    }

    #[test]
    fn clone_is_deep() {
        let original: RandomTreap<i32> = (0..10).collect();
        let mut copy = original.clone();
        copy.remove(&5);
        assert_eq!(original.get(&5), Some(&5));
        assert_eq!(copy.get(&5), None);
    }

    #[test]
    fn equality_compares_the_contents() {
        // Same keys inserted in different orders: different priorities,
        // so possibly a different shape, but equal as sets.
        let a: RandomTreap<i32> = [1, 2, 3].into_iter().collect();
        let b: RandomTreap<i32> = [3, 2, 1].into_iter().collect();
        let c: RandomTreap<i32> = [1, 2].into_iter().collect();
        assert_eq!(a, b);
        assert_ne!(a, c);

        let mut x: Treap<i32, u32> = Treap::new();
        let mut y: Treap<i32, u32> = Treap::new();
        x.insert(1, 10).unwrap();
        y.insert(1, 10).unwrap();
        assert_eq!(x, y);
    }

    #[test]
    fn errors_compare_equal() {
        let mut treap = RandomTreap::new(1);
        assert_eq!(treap.insert(1).unwrap_err(), TreapError::DuplicateKey);
    }
}

/// `DoubleEndedIterator`, `ExactSizeIterator`, `FusedIterator`.
#[cfg(planned = "iterators")]
mod iterators {
    use std::iter::FusedIterator;

    use menagerie::RandomTreap;

    #[test]
    fn iterate_backwards() {
        let treap: RandomTreap<i32> = (1..=5).collect();
        assert!(treap.iter().rev().eq(&[5, 4, 3, 2, 1]));
        assert!(treap.clone().into_iter().rev().eq([5, 4, 3, 2, 1]));
    }

    #[test]
    fn iterate_from_both_ends() {
        let treap: RandomTreap<i32> = (1..=4).collect();
        let mut iter = treap.iter();
        assert_eq!(iter.next(), Some(&1));
        assert_eq!(iter.next_back(), Some(&4));
        assert_eq!(iter.next(), Some(&2));
        assert_eq!(iter.next_back(), Some(&3));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next_back(), None);
    }

    #[test]
    fn exact_size() {
        let treap: RandomTreap<i32> = (0..10).collect();
        let mut iter = treap.iter();
        assert_eq!(iter.len(), 10);
        iter.next();
        assert_eq!(iter.size_hint(), (9, Some(9)));
    }

    #[test]
    fn fused() {
        fn assert_fused<I: FusedIterator>(_: I) {}
        let treap: RandomTreap<i32> = (0..3).collect();
        assert_fused(treap.iter());
        assert_fused(treap.into_iter());
    }
}

/// Degenerate trees: with caller-chosen priorities a `Treap` can be a path.
/// Today these overflow the stack (recursive operations and recursive drop of
/// the `Box`es), aborting the whole test binary instead of failing one test.
#[cfg(planned = "deep")]
mod deep {
    use menagerie::Treap;

    const N: u32 = 300_000;

    /// Sorted keys with increasing priorities in a min-heap: every key becomes
    /// the right child of the previous one.
    fn path() -> Treap<u32, u32> {
        let mut treap = Treap::new();
        for key in 0..N {
            treap.insert(key, key).unwrap();
        }
        treap
    }

    #[test]
    fn build_and_drop_a_path() {
        drop(path());
    }

    #[test]
    fn operations_on_a_path() {
        let mut treap = path();
        assert_eq!(treap.get(&(N - 1)), Some(&(N - 1)));
        assert_eq!(treap.remove(&(N / 2)), Some(N / 2));
        let greater = treap.split_off(&(N / 2));
        let joined = Treap::join(treap, greater).unwrap();
        assert_eq!(joined.iter().count(), N as usize - 1);
    }
}

/// `RandomTreap::join`.
#[cfg(planned = "random_join")]
mod random_join {
    use menagerie::RandomTreap;

    #[test]
    fn join_is_the_inverse_of_split_off() {
        let mut treap: RandomTreap<i32> = (0..100).map(|key| 2 * key).collect();
        let greater = treap.split_off(&77);
        let joined = RandomTreap::join(treap, greater).unwrap();
        assert!(joined.iter().copied().eq((0..100).map(|key| 2 * key)));
    }

    #[test]
    fn join_rejects_overlapping_treaps() {
        let a: RandomTreap<i32> = [1, 5].into_iter().collect();
        let b: RandomTreap<i32> = [3].into_iter().collect();
        assert!(RandomTreap::join(a, b).is_err());
    }
}

/// Union, intersection and difference built on split and join.
#[cfg(planned = "set_ops")]
mod set_ops {
    use std::collections::BTreeSet;

    use menagerie::RandomTreap;

    fn both(keys: &[u32]) -> (RandomTreap<u32>, BTreeSet<u32>) {
        (
            keys.iter().copied().collect(),
            keys.iter().copied().collect(),
        )
    }

    #[test]
    fn set_operations_match_btreeset() {
        let a_keys: Vec<u32> = (0..300).map(|key| key * 7 % 500).collect();
        let b_keys: Vec<u32> = (0..300).map(|key| key * 11 % 500).collect();

        let (a, a_model) = both(&a_keys);
        let (b, b_model) = both(&b_keys);
        let union = a.union(b);
        assert!(union.iter().eq(a_model.union(&b_model)));

        let (a, _) = both(&a_keys);
        let (b, _) = both(&b_keys);
        let intersection = a.intersection(b);
        assert!(intersection.iter().eq(a_model.intersection(&b_model)));

        let (a, _) = both(&a_keys);
        let (b, _) = both(&b_keys);
        let difference = a.difference(b);
        assert!(difference.iter().eq(a_model.difference(&b_model)));
    }

    #[test]
    fn set_operations_with_an_empty_treap() {
        let (a, _) = both(&[1, 2, 3]);
        assert!(a.union(RandomTreap::default()).iter().eq(&[1, 2, 3]));

        let (a, _) = both(&[1, 2, 3]);
        assert_eq!(a.intersection(RandomTreap::default()).iter().count(), 0);

        let (a, _) = both(&[1, 2, 3]);
        assert!(a.difference(RandomTreap::default()).iter().eq(&[1, 2, 3]));
    }
}

/// Order statistics from subtree sizes: `select` (k-th key, from 0) and `rank`
/// (number of smaller keys).
#[cfg(planned = "order_stats")]
mod order_stats {
    use menagerie::RandomTreap;

    #[test]
    fn select_and_rank() {
        let treap: RandomTreap<i32> = (0..50).map(|key| 3 * key).collect();
        for k in 0..50 {
            assert_eq!(treap.select(k), Some(&(3 * k as i32)));
            assert_eq!(treap.rank(&(3 * k as i32)), k);
        }
        assert_eq!(treap.select(50), None);
        // A missing key: its rank is where it would be inserted.
        assert_eq!(treap.rank(&4), 2);
    }

    #[test]
    fn order_statistics_after_updates() {
        let mut treap: RandomTreap<i32> = (0..10).collect();
        treap.remove(&3);
        treap.insert(42).unwrap();
        assert_eq!(treap.select(3), Some(&4));
        assert_eq!(treap.rank(&42), 9);
    }
}

/// `TreapMap<K, V>`: a map with random priorities.
#[cfg(planned = "map")]
mod map {
    use std::collections::BTreeMap;

    use menagerie::treap::TreapMap;

    #[test]
    fn insert_get_remove() {
        let mut map = TreapMap::new();
        assert_eq!(map.insert("a", 1), None);
        assert_eq!(map.insert("a", 2), Some(1));
        assert_eq!(map.get("a"), Some(&2));
        assert!(map.contains_key("a"));

        if let Some(value) = map.get_mut("a") {
            *value += 10;
        }
        assert_eq!(map.get("a"), Some(&12));

        assert_eq!(map.remove("a"), Some(12));
        assert_eq!(map.get("a"), None);
        assert!(map.is_empty());
    }

    #[test]
    fn iterators_are_sorted_by_key() {
        let map: TreapMap<i32, char> = [(3, 'c'), (1, 'a'), (2, 'b')].into_iter().collect();
        assert!(map.iter().eq([(&1, &'a'), (&2, &'b'), (&3, &'c')]));
        assert!(map.keys().eq(&[1, 2, 3]));
        assert!(map.values().eq(&['a', 'b', 'c']));
    }

    #[test]
    fn values_mut() {
        let mut map: TreapMap<i32, i32> = (0..5).map(|key| (key, key)).collect();
        for value in map.values_mut() {
            *value *= 2;
        }
        assert!(map.values().copied().eq((0..5).map(|key| key * 2)));
    }

    #[test]
    fn matches_btreemap() {
        let mut map = TreapMap::new();
        let mut model = BTreeMap::new();
        let mut state: u64 = 0x1234_5678;
        for _ in 0..10_000 {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            let key = state % 500;
            match state % 3 {
                0 => assert_eq!(map.insert(key, state), model.insert(key, state)),
                1 => assert_eq!(map.remove(&key), model.remove(&key)),
                _ => assert_eq!(map.get(&key), model.get(&key)),
            }
        }
        assert!(map.iter().eq(model.iter()));
        assert_eq!(map.len(), model.len());
    }
}
