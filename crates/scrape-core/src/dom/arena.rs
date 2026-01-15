//! Arena allocator for DOM nodes.
//!
//! Provides a simple bump-allocator pattern using contiguous `Vec` storage.
//! All nodes are allocated sequentially with O(1) amortized allocation.

use std::ops::{Index, IndexMut};

/// Arena allocator for contiguous node storage.
///
/// Nodes are stored in a single `Vec`, indexed by their allocation order.
/// This provides cache-friendly access and eliminates per-node heap allocations.
#[derive(Debug)]
pub struct Arena<T> {
    nodes: Vec<T>,
}

impl<T> Default for Arena<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Arena<T> {
    /// Creates a new empty arena.
    #[must_use]
    pub const fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    /// Creates an arena with pre-allocated capacity.
    ///
    /// Use this when the approximate number of nodes is known
    /// to avoid reallocations during parsing.
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self { nodes: Vec::with_capacity(capacity) }
    }

    /// Allocates a new item in the arena, returning its index.
    ///
    /// # Time Complexity
    /// O(1) amortized
    pub fn alloc(&mut self, item: T) -> usize {
        let index = self.nodes.len();
        self.nodes.push(item);
        index
    }

    /// Returns a reference to the item at the given index.
    #[must_use]
    pub fn get(&self, index: usize) -> Option<&T> {
        self.nodes.get(index)
    }

    /// Returns a mutable reference to the item at the given index.
    #[must_use]
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.nodes.get_mut(index)
    }

    /// Returns the number of items in the arena.
    #[must_use]
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Returns `true` if the arena contains no items.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Returns an iterator over all items with their indices.
    pub fn iter(&self) -> impl Iterator<Item = (usize, &T)> {
        self.nodes.iter().enumerate()
    }

    /// Clears all items from the arena.
    pub fn clear(&mut self) {
        self.nodes.clear();
    }
}

impl<T> Index<usize> for Arena<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.nodes[index]
    }
}

impl<T> IndexMut<usize> for Arena<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.nodes[index]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alloc_returns_sequential_indices() {
        let mut arena: Arena<i32> = Arena::new();
        assert_eq!(arena.alloc(10), 0);
        assert_eq!(arena.alloc(20), 1);
        assert_eq!(arena.alloc(30), 2);
    }

    #[test]
    fn get_returns_allocated_item() {
        let mut arena: Arena<&str> = Arena::new();
        let idx = arena.alloc("hello");
        assert_eq!(arena.get(idx), Some(&"hello"));
    }

    #[test]
    fn get_returns_none_for_invalid_index() {
        let arena: Arena<i32> = Arena::new();
        assert!(arena.get(0).is_none());
        assert!(arena.get(999).is_none());
    }

    #[test]
    fn index_operator_accesses_items() {
        let mut arena: Arena<i32> = Arena::new();
        arena.alloc(42);
        assert_eq!(arena[0], 42);
    }

    #[test]
    fn index_mut_allows_modification() {
        let mut arena: Arena<i32> = Arena::new();
        arena.alloc(10);
        arena[0] = 20;
        assert_eq!(arena[0], 20);
    }

    #[test]
    fn len_and_is_empty() {
        let mut arena: Arena<i32> = Arena::new();
        assert!(arena.is_empty());
        assert_eq!(arena.len(), 0);

        arena.alloc(1);
        assert!(!arena.is_empty());
        assert_eq!(arena.len(), 1);
    }

    #[test]
    fn iter_enumerates_all_items() {
        let mut arena: Arena<char> = Arena::new();
        arena.alloc('a');
        arena.alloc('b');
        arena.alloc('c');

        let collected: Vec<_> = arena.iter().collect();
        assert_eq!(collected, vec![(0, &'a'), (1, &'b'), (2, &'c')]);
    }

    #[test]
    fn clear_removes_all_items() {
        let mut arena: Arena<i32> = Arena::new();
        arena.alloc(1);
        arena.alloc(2);
        arena.clear();

        assert!(arena.is_empty());
        assert!(arena.get(0).is_none());
    }

    #[test]
    fn with_capacity_preallocates() {
        let arena: Arena<i32> = Arena::with_capacity(100);
        assert!(arena.is_empty());
    }
}
