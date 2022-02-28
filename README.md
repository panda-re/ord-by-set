# ord-by-set

A library providing a weakly ordered multi-set with compile-time configurable
ordering scheme.

### When To Use This

* When you want a `BTreeSet` but your data involves
partial/loose equivelance, and you want to be able to perform efficient retrievals of
multiple values of loose equivelance.
* When you have ordered keys stored in the same type as the values, allowing
a `BTreeMap`-like data structure but with inline
keys.
    * This is done by using a custom `Order` implementation in order to order
    types by the fields being used as keys, without a reliance on being totally ordered
* When you want a multi-{set, map} but hashing is not an option

### When Not To Use This

* In place of `HashMap`/`HashSet`/`BTreeMap`/`BTreeSet` when you don't need multiple
loosely equivelant values.

## Overview

An `OrdBySet` is composed of two parts: its storage backing (a sorted `Vec<T>`)
and a user-provided orderer. An orderer is a value which can take two items and
loosely compare them. This is done via the `Order<T>` trait, which requires a
single method, `Order::order_of`:

```rust
fn order_of(&self, left: &T, right: &T) -> Ordering;
```

Unlike `Ord`, however, this is not guaranteed to be [totally ordered], and as
such it can be used in such a manner that groups loosely-equivelant values, similarly
to how a [Bag datastructure] allows for storing multiple of the same value.

[totally ordered]: https://wikipedia.org/wiki/Total_order
[Bag datastructure]: https://docs.rs/hashbag/latest/hashbag/struct.HashBag.html

The differentiating feature, however, is that one can then proceed to query all
losely equivelant types[^1]. The ordering scheme.

[^1]: One example being that you might want a query of 3 to turn up both 3 as an
integer and 3 as a string, while still storing both the string and the integer.
For more info on this see `Order`'s docs.

### Example

```rust
use ord_by_set::OrdBySet;

// Our orderer will be a simple function that sorts based on the first 5 characters
let ordering_fn = |left: &&str, right: &&str| left[..5].cmp(&right[..5]);

let set = OrdBySet::new_with_order(ordering_fn)
    .with_items(["00001_foo", "00001_bar", "00002_foo"]);

let id_1_subset = set.get(&"00001").unwrap();

// id_1_subset = unordered(["00001_foo", "00001_bar"])
assert_eq!(id_1_subset.len(), 2);
assert!(id_1_subset.contains(&"00001_bar"));
assert!(id_1_subset.contains(&"00001_foo"));
```

While the above uses a closure for the orderer, it can be any type if you implement
`Order<T>`. Typically this is done via a [zero-sized type] as usually state is not
needed by the ordering mechanism, just behavior:

```rust
use ord_by_set::{OrdBySet, Order};
use std::cmp::Ordering;

#[derive(Default)]
struct EverythingEqual;

impl<T> Order<T> for EverythingEqual {
    fn order_of(&self, _: &T, _: &T) -> Ordering {
        Ordering::Equal
    }
}

type AllEqualSet = OrdBySet<i32, EverythingEqual>;

let mut set = AllEqualSet::new().with_items([3, 5, 2, 7]);

assert_eq!(set.count(&30), 4);
set.remove_all(&0);
assert!(set.is_empty());
```

[zero-sized type]: https://doc.rust-lang.org/nomicon/exotic-sizes.html#zero-sized-types-zsts
