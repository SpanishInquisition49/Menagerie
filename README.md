# menagerie

A collection of data structures in safe Rust, classic and randomized, each
documented with its algorithms and complexities.

> **Status:** early development (`0.1`). The API may still change.

## Data structures

| Structure     | Type                    | Status  |
| ------------- | ----------------------- | ------- |
| Treap         | `Treap<T, P, O>`        | ready   |
| Random treap  | `RandomTreap<T>`        | ready   |
| Skip list     |                         | planned |
| Ternary search tree |                   | planned |
| Patricia trie |                         | planned |

### Treaps

A treap is a binary search tree on the keys that is also a heap on the
priorities. With random priorities its expected depth is `O(log n)`, whatever
the order of the insertions.

- `Treap<T, P, O>`: the caller chooses every priority, and the heap order `O`
  (`MinHeap` by default, or `MaxHeap`), a zero-sized type fixed at compile
  time.
- `RandomTreap<T>`: draws a random priority for every key.

| Operation          | Expected time (random priorities) |
| ------------------ | --------------------------------- |
| `get`              | `O(log n)`                        |
| `insert`           | `O(log n)`                        |
| `remove`           | `O(log n)`                        |
| `split_off`        | `O(log n)`                        |
| `join`             | `O(log n)`                        |
| `iter` (whole set) | `O(n)`                            |

Lookups take any borrowed form of the key, as in the standard collections: a
treap of `String` can be searched with a `&str`.

## Usage

```toml
[dependencies]
menagerie = "0.1"
```

```rust
use menagerie::{RandomTreap, Treap};

// Random priorities.
let mut treap: RandomTreap<i32> = (1..=10).collect();
assert_eq!(treap.remove(&4), Some(4));
assert_eq!(treap.get(&4), None);

// Split at a key, then join the halves back.
let mut generic: Treap<&str, u32> = Treap::new();
for (key, priority) in [("a", 3), ("c", 1), ("e", 2)] {
    generic.insert(key, priority).unwrap();
}
let greater = generic.split_off("c");
let joined = Treap::join(generic, greater).unwrap();
assert!(joined.iter().map(|(key, _)| *key).eq(["a", "c", "e"]));
```

The API documentation is built with `cargo doc --open`.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.
