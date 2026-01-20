//! Document lifecycle state types.
//!
//! These types encode the document's lifecycle state at the type level,
//! enabling compile-time enforcement of valid operations.
//!
//! # States
//!
//! - [`Building`] - Document under construction
//! - [`Queryable`] - Document ready for queries
//! - [`Sealed`] - Fully immutable document
//!
//! # State Transitions
//!
//! ```text
//! Building --[.build()]--> Queryable --[.seal()]--> Sealed
//! ```
//!
//! The state is encoded at the type level using [`PhantomData`](std::marker::PhantomData)
//! and has zero runtime overhead.

mod private {
    pub trait Sealed {}
}

/// Marker trait for document states.
///
/// This trait is sealed - only the states defined in this module
/// can implement it. This prevents external code from defining
/// new document states.
pub trait DocumentState: private::Sealed + Clone + Copy + Default {}

/// Document is being constructed.
///
/// In this state:
/// - Structure can be modified (`create_element`, `append_child`, etc.)
/// - Navigation available (`parent`, `children`)
/// - Query methods (`find`, `find_all`) are NOT available
#[derive(Debug, Clone, Copy, Default)]
pub struct Building;

impl private::Sealed for Building {}
impl DocumentState for Building {}

/// Document is built and ready for querying.
///
/// In this state:
/// - Full navigation and query methods available
/// - Structure cannot be modified
/// - Index can be built for faster queries
#[derive(Debug, Clone, Copy, Default)]
pub struct Queryable;

impl private::Sealed for Queryable {}
impl DocumentState for Queryable {}

/// Document is sealed and fully immutable.
///
/// In this state:
/// - Same as Queryable, but guarantees no mutations ever
/// - Enables potential future optimizations (e.g., shared memory)
/// - Cannot transition to any other state
#[derive(Debug, Clone, Copy, Default)]
pub struct Sealed;

impl private::Sealed for Sealed {}
impl DocumentState for Sealed {}

/// Trait for states that support querying.
///
/// Both [`Queryable`] and [`Sealed`] states support querying operations.
pub trait QueryableState: DocumentState {}
impl QueryableState for Queryable {}
impl QueryableState for Sealed {}

/// Trait for states that support modification.
///
/// Only the [`Building`] state supports structural modifications.
pub trait MutableState: DocumentState {}
impl MutableState for Building {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(clippy::no_effect_underscore_binding)]
    fn states_are_copy() {
        // Copy trait is verified by using value twice
        let b = Building;
        let _b2 = b; // First use (copy)
        let _b3 = b; // Second use (copy) - would fail without Copy

        let q = Queryable;
        let _q2 = q;
        let _q3 = q;

        let s = Sealed;
        let _s2 = s;
        let _s3 = s;
    }

    #[test]
    fn states_are_default() {
        let _: Building = Building;
        let _: Queryable = Queryable;
        let _: Sealed = Sealed;
    }

    #[test]
    fn states_are_zero_sized() {
        assert_eq!(std::mem::size_of::<Building>(), 0);
        assert_eq!(std::mem::size_of::<Queryable>(), 0);
        assert_eq!(std::mem::size_of::<Sealed>(), 0);
    }

    #[test]
    fn building_is_mutable_state() {
        fn require_mutable<S: MutableState>(_: S) {}
        require_mutable(Building);
    }

    #[test]
    fn queryable_is_queryable_state() {
        fn require_queryable<S: QueryableState>(_: S) {}
        require_queryable(Queryable);
        require_queryable(Sealed);
    }
}
