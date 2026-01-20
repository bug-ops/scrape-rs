//! Selector explanation and performance analysis.

use super::{CompiledSelector, QueryResult, specificity::Specificity};
use crate::dom::Document;

/// Performance hint for selector optimization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OptimizationHint {
    /// Selector is already optimal.
    Optimal,
    /// Consider using ID selector for better performance.
    UseIdSelector {
        /// Current selector pattern.
        current: String,
        /// Suggested optimized selector.
        suggested: String,
    },
    /// Selector is too broad, consider being more specific.
    TooBroad {
        /// Reason why the selector is too broad.
        reason: String,
    },
    /// Descendant combinator can be slow, consider child combinator.
    PreferChildCombinator {
        /// Location of the descendant combinator.
        at: String,
    },
    /// Universal selector (*) should be avoided.
    AvoidUniversalSelector,
    /// Consider caching this compiled selector.
    CacheSelector,
}

/// Detailed explanation of a CSS selector.
#[derive(Debug, Clone)]
pub struct SelectorExplanation {
    /// The original selector string.
    pub source: String,
    /// Parsed specificity.
    pub specificity: Specificity,
    /// Human-readable description of what the selector matches.
    pub description: String,
    /// Performance characteristics.
    pub performance_notes: Vec<String>,
    /// Optimization suggestions.
    pub hints: Vec<OptimizationHint>,
    /// Estimated match count (if document provided).
    pub estimated_matches: Option<usize>,
}

impl SelectorExplanation {
    /// Creates a new selector explanation by analyzing the selector.
    ///
    /// # Performance
    ///
    /// This function is O(n) where n is the selector length.
    /// Target: <1ms for typical selectors.
    #[must_use]
    pub fn analyze(selector: &CompiledSelector) -> Self {
        let specificity = calculate_specificity(selector);
        let description = generate_description(selector);
        let (performance_notes, hints) = analyze_performance(selector);

        Self {
            source: selector.source().to_string(),
            specificity,
            description,
            performance_notes,
            hints,
            estimated_matches: None,
        }
    }

    /// Creates explanation with estimated match count from a document.
    #[must_use]
    pub fn analyze_with_document(selector: &CompiledSelector, doc: &Document) -> Self {
        let mut explanation = Self::analyze(selector);
        explanation.estimated_matches = Some(count_matches(selector, doc));
        explanation
    }

    /// Formats the explanation for human-readable output.
    #[must_use]
    pub fn format(&self) -> String {
        use std::fmt::Write;

        let mut output = String::new();
        let source = &self.source;
        let specificity = &self.specificity;
        let description = &self.description;

        let _ = writeln!(output, "Selector: {source}");
        let _ = writeln!(output, "Specificity: {specificity}");
        let _ = writeln!(output, "Description: {description}");

        if let Some(count) = self.estimated_matches {
            let _ = writeln!(output, "Estimated matches: {count}");
        }

        if !self.performance_notes.is_empty() {
            output.push_str("\nPerformance:\n");
            for note in &self.performance_notes {
                let _ = writeln!(output, "  - {note}");
            }
        }

        if !self.hints.is_empty() {
            output.push_str("\nOptimization hints:\n");
            for hint in &self.hints {
                let hint_str = format_hint(hint);
                let _ = writeln!(output, "  - {hint_str}");
            }
        }

        output
    }
}

// Internal functions for analysis

fn calculate_specificity(selector: &CompiledSelector) -> Specificity {
    use selectors::parser::Component;

    let mut ids = 0u32;
    let mut classes = 0u32;
    let mut elements = 0u32;

    for sel in selector.selector_list().slice() {
        for component in sel.iter_raw_parse_order_from(0) {
            match component {
                Component::ID(_) => ids += 1,
                Component::Class(_)
                | Component::AttributeInNoNamespace { .. }
                | Component::AttributeInNoNamespaceExists { .. }
                | Component::AttributeOther(_)
                | Component::NonTSPseudoClass(_)
                | Component::Negation(_)
                | Component::Is(_)
                | Component::Where(_)
                | Component::Has(_) => classes += 1,
                Component::LocalName(_) | Component::PseudoElement(_) => elements += 1,
                _ => {}
            }
        }
    }

    Specificity::new(ids, classes, elements)
}

fn generate_description(selector: &CompiledSelector) -> String {
    let source = selector.source();

    // Simple heuristics for common patterns
    if source.starts_with('#') && !source.contains(' ') && !source.contains('>') {
        let id = source.get(1..).unwrap_or("");
        format!("Element with ID '{id}'")
    } else if source.starts_with('.') && !source.contains(' ') && !source.contains('>') {
        let class = source.get(1..).unwrap_or("");
        format!("Elements with class '{class}'")
    } else if source.contains(' ') && !source.contains('>') {
        "Elements matching a descendant selector".to_string()
    } else if source.contains('>') {
        "Elements matching a child selector".to_string()
    } else if source.contains('+') {
        "Elements matching an adjacent sibling selector".to_string()
    } else if source.contains('~') {
        "Elements matching a general sibling selector".to_string()
    } else {
        format!("Elements matching '{source}'")
    }
}

fn analyze_performance(selector: &CompiledSelector) -> (Vec<String>, Vec<OptimizationHint>) {
    let mut notes = Vec::new();
    let mut hints = Vec::new();
    let source = selector.source();

    // Check for universal selector
    if source.contains('*') && !source.contains("[*") {
        notes.push("Contains universal selector - may be slow on large documents".to_string());
        hints.push(OptimizationHint::AvoidUniversalSelector);
    }

    // Check for deep descendant selectors
    let descendant_count = source.split_whitespace().count();
    if descendant_count > 3 {
        notes.push(format!(
            "Deep descendant chain ({descendant_count} levels) - consider simplifying"
        ));
        hints.push(OptimizationHint::TooBroad {
            reason: "Deep nesting requires traversing many ancestors".to_string(),
        });
    }

    // Check for ID selector (fast path)
    if source.starts_with('#') && !source.contains(' ') && !source.contains('>') {
        notes.push("ID selector - uses fast indexed lookup".to_string());
        hints.push(OptimizationHint::Optimal);
    }

    // Suggest caching for complex selectors
    if source.len() > 30 || descendant_count > 2 {
        hints.push(OptimizationHint::CacheSelector);
    }

    // Check for descendant vs child combinator
    if source.contains(' ') && !source.contains('>') {
        notes.push(
            "Uses descendant combinator - child combinator (>) may be faster for direct children"
                .to_string(),
        );
    }

    (notes, hints)
}

fn count_matches(selector: &CompiledSelector, doc: &Document) -> usize {
    use super::find_all_compiled;
    find_all_compiled(doc, selector).len()
}

fn format_hint(hint: &OptimizationHint) -> String {
    match hint {
        OptimizationHint::Optimal => "Selector is already optimal".to_string(),
        OptimizationHint::UseIdSelector { current, suggested } => {
            format!("Consider ID selector: '{current}' -> '{suggested}'")
        }
        OptimizationHint::TooBroad { reason } => format!("Too broad: {reason}"),
        OptimizationHint::PreferChildCombinator { at } => {
            format!("Consider child combinator (>) at: {at}")
        }
        OptimizationHint::AvoidUniversalSelector => {
            "Avoid universal selector (*) for better performance".to_string()
        }
        OptimizationHint::CacheSelector => {
            "Consider caching this compiled selector for reuse".to_string()
        }
    }
}

/// Convenience function to explain a selector string.
///
/// # Errors
///
/// Returns `QueryError::InvalidSelector` if the selector is invalid.
pub fn explain(selector: &str) -> QueryResult<SelectorExplanation> {
    let compiled = CompiledSelector::compile(selector)?;
    Ok(SelectorExplanation::analyze(&compiled))
}

/// Explains a selector with match count from a document.
///
/// # Errors
///
/// Returns `QueryError::InvalidSelector` if the selector is invalid.
pub fn explain_with_document(selector: &str, doc: &Document) -> QueryResult<SelectorExplanation> {
    let compiled = CompiledSelector::compile(selector)?;
    Ok(SelectorExplanation::analyze_with_document(&compiled, doc))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_specificity_calculation() {
        let selector = CompiledSelector::compile("#id .class tag").unwrap();
        let explanation = SelectorExplanation::analyze(&selector);
        assert_eq!(explanation.specificity.ids, 1);
        assert_eq!(explanation.specificity.classes, 1);
        assert_eq!(explanation.specificity.elements, 1);
    }

    #[test]
    fn test_explain_performance_hint_universal() {
        let explanation = explain("*").unwrap();
        assert!(explanation.hints.contains(&OptimizationHint::AvoidUniversalSelector));
    }

    #[test]
    fn test_explain_id_selector_optimal() {
        let explanation = explain("#myid").unwrap();
        assert!(explanation.hints.contains(&OptimizationHint::Optimal));
    }

    #[test]
    fn test_explain_deep_nesting() {
        let explanation = explain("div span p a").unwrap();
        assert!(explanation.hints.iter().any(|h| matches!(h, OptimizationHint::TooBroad { .. })));
    }

    #[test]
    fn test_explain_cache_suggestion() {
        let explanation = explain("div.container > ul.list > li.item:first-child").unwrap();
        assert!(explanation.hints.contains(&OptimizationHint::CacheSelector));
    }

    #[test]
    fn test_description_generation() {
        let id_sel = explain("#test").unwrap();
        assert!(id_sel.description.contains("ID"));

        let class_sel = explain(".test").unwrap();
        assert!(class_sel.description.contains("class"));

        let descendant_sel = explain("div span").unwrap();
        assert!(descendant_sel.description.contains("descendant"));

        let child_sel = explain("div > span").unwrap();
        assert!(child_sel.description.contains("child"));
    }

    #[test]
    fn test_format_output() {
        let explanation = explain("div.test").unwrap();
        let formatted = explanation.format();

        assert!(formatted.contains("Selector:"));
        assert!(formatted.contains("Specificity:"));
        assert!(formatted.contains("Description:"));
    }

    #[test]
    fn test_explain_invalid_selector() {
        let result = explain(":::");
        assert!(result.is_err());
    }

    #[test]
    fn test_explain_under_1ms() {
        use std::time::Instant;

        let start = Instant::now();
        for _ in 0..1000 {
            let _ = explain("div.container > ul.list > li.item:first-child");
        }
        let elapsed = start.elapsed();

        assert!(
            elapsed.as_micros() / 1000 < 1000,
            "explain() should be <1ms per call, got {}μs average",
            elapsed.as_micros() / 1000
        );
    }
}
