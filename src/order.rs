use core::cmp::Ordering;

/// A trait representing the capability of taking two items and ordering them.
///
/// An orderer *is* allowed to have two "equal" values which are not actually equal,
/// but can be considered loosely equal. This is similar to javascript's `==` operator,
/// while [`Ord`] would be equivelant to javascript's `===` operator.
///
/// For example, if you had an enum which allowed both strings and numbers:
///
/// ```
/// enum Val {
///     String(String),
///     Num(i32),
/// }
/// ```
///
/// You *could* allow `Val::String("3")` to be loosely equivelant to `Val::Num(3)`, while still
/// having them be distinct values. Then if the following operation is performed:
///
/// ```
/// # enum Val {
/// #     String(String),
/// #     Num(i32),
/// # }
/// #
/// use ord_by_set::{OrdBySet, Order};
/// use std::cmp::Ordering;
///
/// #[derive(Default)]
/// struct LooseOrder;
///
/// impl Order<Val> for LooseOrder {
///     fn order_of(&self, left: &Val, right: &Val) -> Ordering {
///         match (left, right) {
///             (Val::String(left), Val::String(right)) => left.cmp(right),
///             (Val::Num(left), Val::Num(right)) => left.cmp(right),
///
///             (Val::String(left), Val::Num(right)) => left.parse::<i32>()
///                 .unwrap()
///                 .cmp(right),
///             (Val::Num(left), Val::String(right)) => left.cmp(&right.parse::<i32>().unwrap()),
///         }
///     }
/// }
///
/// let totally_numbers = [
///     Val::Num(100),
///     Val::String("70".into()),
///     Val::Num(70),
///     Val::String("30".into()),
/// ];
/// let ord = OrdBySet::new_with_order(LooseOrder).with_items(totally_numbers);
///
/// assert!(matches!(
///     ord.get(&Val::Num(70)),
///     Some([Val::Num(70), Val::String(num)] | [Val::String(num), Val::Num(70)])
///         if num == "70"
/// ));
/// ```
///
/// ### Specification
///
/// The following behaviors must hold true in a proper `Order<T>` implementation:
///
/// * Exactly one of `a < b`, `a > b`, or `a == b` is true.
/// * LessThan, Equals, and GreaterThan are all transitive. Which is to say that
/// `a == b` and `b == c` implies `a == c`.
///
/// The easiest way to think about this is that `Order<T>` is a proper implementation of
/// [`Ord`] for a subset of the type `T`, albeit with possibly alternate behavior to that
/// of T's [`Ord`] itself, if such an implementation exists.
///
/// Failure to uphold this contract will result in unspecified (albeit safe/sound in the
/// context of Rust's safety guarantees) behavior by [`OrdBySet`].
///
/// While not strictly required for a valid `Order` implementation, if you wish to use
/// the `*_specific` methods in `OrdBySet<T>`, then it is expected that the set of
/// comparison pairs (left, right) will return `Ordering::Equal` for all values of
/// `left` and `right` where `PartialEq::eq(left, right)` returns `true`.
///
/// That is to say, the set of comparison pairs which are equivelant under `Order<T>`
/// should be a superset of those equal under `PartialEq` if you choose to use those
/// methods.
pub trait Order<T> {
    /// Takes two items and compares them, returning if the first is less than, equal to,
    /// or greater than, the latter.
    fn order_of(&self, left: &T, right: &T) -> Ordering;

    /// Takes a slice of items and sorts them using the given order
    fn sort_slice(&self, items: &mut [T]) {
        items.sort_by(|left, right| self.order_of(&left, &right));
    }
}

/// An ordering implementation that just defers to [`Ord`]
#[derive(Default)]
pub struct FullOrd;

impl<T: Ord> Order<T> for FullOrd {
    fn order_of(&self, left: &T, right: &T) -> Ordering {
        left.cmp(right)
    }
}

/// An implementation for closures to allow for ad-hoc ordering
impl<T, OrderFn> Order<T> for OrderFn
where
    OrderFn: Fn(&T, &T) -> Ordering,
{
    fn order_of(&self, left: &T, right: &T) -> Ordering {
        self(left, right)
    }
}
