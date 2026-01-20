//! CSS selector specificity calculation.

/// CSS specificity value (a, b, c) where:
/// - a = number of ID selectors
/// - b = number of class selectors, attribute selectors, pseudo-classes
/// - c = number of type selectors, pseudo-elements
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Specificity {
    /// Number of ID selectors (#id).
    pub ids: u32,
    /// Number of class/attribute/pseudo-class selectors.
    pub classes: u32,
    /// Number of type/pseudo-element selectors.
    pub elements: u32,
}

impl Specificity {
    /// Creates a new specificity value.
    #[must_use]
    pub const fn new(ids: u32, classes: u32, elements: u32) -> Self {
        Self { ids, classes, elements }
    }

    /// Returns the specificity as a single comparable value.
    /// Higher values indicate higher specificity.
    #[must_use]
    pub const fn value(&self) -> u64 {
        ((self.ids as u64) << 32) | ((self.classes as u64) << 16) | (self.elements as u64)
    }

    /// Returns a human-readable specificity string like "(0, 2, 1)".
    #[must_use]
    pub fn display(&self) -> String {
        format!("({}, {}, {})", self.ids, self.classes, self.elements)
    }
}

impl std::fmt::Display for Specificity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_specificity_creation() {
        let spec = Specificity::new(1, 2, 3);
        assert_eq!(spec.ids, 1);
        assert_eq!(spec.classes, 2);
        assert_eq!(spec.elements, 3);
    }

    #[test]
    fn test_specificity_value() {
        let spec1 = Specificity::new(1, 0, 0);
        let spec2 = Specificity::new(0, 1, 0);
        let spec3 = Specificity::new(0, 0, 1);

        assert!(spec1.value() > spec2.value());
        assert!(spec2.value() > spec3.value());
    }

    #[test]
    fn test_specificity_comparison() {
        let spec1 = Specificity::new(1, 0, 0);
        let spec2 = Specificity::new(0, 100, 0);
        let spec3 = Specificity::new(0, 0, 100);

        assert!(spec1 > spec2);
        assert!(spec2 > spec3);
    }

    #[test]
    fn test_specificity_display() {
        let spec = Specificity::new(1, 2, 3);
        assert_eq!(spec.display(), "(1, 2, 3)");
        assert_eq!(spec.to_string(), "(1, 2, 3)");
    }

    #[test]
    fn test_specificity_default() {
        let spec = Specificity::default();
        assert_eq!(spec.ids, 0);
        assert_eq!(spec.classes, 0);
        assert_eq!(spec.elements, 0);
    }

    #[test]
    fn test_specificity_equality() {
        let spec1 = Specificity::new(1, 2, 3);
        let spec2 = Specificity::new(1, 2, 3);
        let spec3 = Specificity::new(0, 2, 3);

        assert_eq!(spec1, spec2);
        assert_ne!(spec1, spec3);
    }
}
