//! Parse warning collection for developer experience.

use crate::error::SourceSpan;

/// Severity level for parse warnings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum WarningSeverity {
    /// Informational message (HTML5 quirks mode, etc.).
    Info,
    /// Warning (recoverable issue).
    Warning,
    /// Error that was recovered from.
    RecoveredError,
}

/// A warning or error encountered during parsing.
#[derive(Debug, Clone)]
pub struct ParseWarning {
    /// Severity of the warning.
    pub severity: WarningSeverity,
    /// Description of the issue.
    pub message: String,
    /// Location in source, if available.
    pub span: Option<SourceSpan>,
    /// HTML5 spec reference, if applicable.
    pub spec_reference: Option<String>,
}

impl ParseWarning {
    /// Creates a new parse warning.
    #[must_use]
    pub fn new(severity: WarningSeverity, message: impl Into<String>) -> Self {
        Self { severity, message: message.into(), span: None, spec_reference: None }
    }

    /// Sets the span for this warning.
    #[must_use]
    pub fn with_span(mut self, span: SourceSpan) -> Self {
        self.span = Some(span);
        self
    }

    /// Sets the spec reference for this warning.
    #[must_use]
    pub fn with_spec_reference(mut self, reference: impl Into<String>) -> Self {
        self.spec_reference = Some(reference.into());
        self
    }
}

/// Result of parsing with warnings collected.
#[derive(Debug)]
pub struct ParseResultWithWarnings<T> {
    /// The parsed result.
    pub result: T,
    /// Warnings encountered during parsing.
    pub warnings: Vec<ParseWarning>,
}

impl<T> ParseResultWithWarnings<T> {
    /// Creates a new result with warnings.
    #[must_use]
    pub fn new(result: T, warnings: Vec<ParseWarning>) -> Self {
        Self { result, warnings }
    }

    /// Returns true if there are any warnings.
    #[must_use]
    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }

    /// Returns warnings filtered by minimum severity.
    #[must_use]
    pub fn warnings_at_least(&self, min_severity: WarningSeverity) -> Vec<&ParseWarning> {
        self.warnings.iter().filter(|w| w.severity >= min_severity).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::SourcePosition;

    #[test]
    fn test_warning_creation() {
        let warning = ParseWarning::new(WarningSeverity::Warning, "unclosed tag");
        assert_eq!(warning.severity, WarningSeverity::Warning);
        assert_eq!(warning.message, "unclosed tag");
        assert!(warning.span.is_none());
        assert!(warning.spec_reference.is_none());
    }

    #[test]
    fn test_warning_with_span() {
        let span = SourceSpan::new(SourcePosition::new(1, 1, 0), SourcePosition::new(1, 5, 4));
        let warning = ParseWarning::new(WarningSeverity::Warning, "test").with_span(span);

        assert!(warning.span.is_some());
        assert_eq!(warning.span.unwrap(), span);
    }

    #[test]
    fn test_warning_with_spec_reference() {
        let warning = ParseWarning::new(WarningSeverity::Info, "test")
            .with_spec_reference("https://html.spec.whatwg.org/#parse-state");

        assert!(warning.spec_reference.is_some());
        assert!(warning.spec_reference.unwrap().contains("html.spec.whatwg.org"));
    }

    #[test]
    fn test_severity_ordering() {
        assert!(WarningSeverity::Info < WarningSeverity::Warning);
        assert!(WarningSeverity::Warning < WarningSeverity::RecoveredError);
    }

    #[test]
    fn test_parse_result_with_warnings() {
        let warnings = vec![
            ParseWarning::new(WarningSeverity::Info, "info"),
            ParseWarning::new(WarningSeverity::Warning, "warning"),
            ParseWarning::new(WarningSeverity::RecoveredError, "error"),
        ];
        let result = ParseResultWithWarnings::new(42, warnings);

        assert!(result.has_warnings());
        assert_eq!(result.warnings.len(), 3);
    }

    #[test]
    fn test_warnings_filtering() {
        let warnings = vec![
            ParseWarning::new(WarningSeverity::Info, "info"),
            ParseWarning::new(WarningSeverity::Warning, "warning"),
            ParseWarning::new(WarningSeverity::RecoveredError, "error"),
        ];
        let result = ParseResultWithWarnings::new((), warnings);

        let filtered = result.warnings_at_least(WarningSeverity::Warning);
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0].message, "warning");
        assert_eq!(filtered[1].message, "error");
    }

    #[test]
    fn test_no_warnings() {
        let result = ParseResultWithWarnings::new(42, Vec::new());
        assert!(!result.has_warnings());
        assert_eq!(result.warnings_at_least(WarningSeverity::Info).len(), 0);
    }
}
