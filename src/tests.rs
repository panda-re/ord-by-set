use crate::*;

fn ord_set<T: Ord, const N: usize>(from: [T; N]) -> OrdBySet<T> {
    let mut set = OrdBySet::new();

    for item in IntoIterator::into_iter(from) {
        set.insert(item);
    }

    set
}

#[test]
fn empty_index_range() {
    assert!(OrdBySet::<usize>::new().get_index_range_of(&0).is_none());
}

#[test]
fn index_range_presorted() {
    assert_eq!(
        ord_set([1, 2, 3, 3, 3, 4]).get_index_range_of(&3),
        Some(2..5)
    );
    assert_eq!(ord_set([1, 1, 1]).get_index_range_of(&1), Some(0..3));
}

#[test]
fn index_range_unsorted() {
    assert_eq!(
        ord_set([2, 1, 3, 1, 3, 4]).get_index_range_of(&3),
        Some(3..5)
    );
}

#[test]
fn slice_range_unsorted() {
    assert_eq!(
        OrdBySet::fully_ordered()
            .with_items([2, 1, 3, 1, 3, 4])
            .range(&2, &4)
            .unwrap()
            .iter()
            .copied()
            .collect::<Vec<_>>(),
        [2, 3, 3, 4]
    );
}
