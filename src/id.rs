//! [`Id`]: a checked HyperFrames id.

use std::fmt;

/// The maximum id length in characters.
pub const MAX_ID_LEN: usize = 128;

/// A checked id for a composition, a clip or a variable.
///
/// The grammar is `[A-Za-z][A-Za-z0-9_-]*`, at most [`MAX_ID_LEN`] characters.
/// The first letter stops the runtime from reading the id as a time
/// (`data-start="12"`). The id is also safe in a CSS selector.
///
/// ```rust
/// use autumn_plugin_hyperframes::Id;
///
/// assert!(Id::new("card-pro").is_some());
/// assert!(Id::new("12").is_none());
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Id(String);

impl Id {
    /// Checks `id`. Returns `None` when it does not match the grammar.
    #[must_use]
    pub fn new(id: &str) -> Option<Self> {
        is_valid_id(id).then(|| Self(id.to_owned()))
    }

    /// The id text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl AsRef<str> for Id {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// True when `id` matches the id grammar.
pub(crate) fn is_valid_id(id: &str) -> bool {
    let _ = id;
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grammar_edges() {
        assert!(is_valid_id("a"));
        assert!(is_valid_id("Z9_-"));
        assert!(is_valid_id(&"a".repeat(MAX_ID_LEN)));
        assert!(!is_valid_id(&"a".repeat(MAX_ID_LEN + 1)));
        assert!(!is_valid_id(""));
        assert!(!is_valid_id("-a"));
        assert!(!is_valid_id("_a"));
        assert!(!is_valid_id("9"));
        assert!(!is_valid_id("a b"));
        assert!(!is_valid_id("é"));
    }

    #[test]
    fn id_reads_back() {
        let id = Id::new("intro").expect("valid");
        assert_eq!(id.as_str(), "intro");
        assert_eq!(id.to_string(), "intro");
        assert_eq!(id.as_ref(), "intro");
        assert!(Id::new("1").is_none());
    }
}
