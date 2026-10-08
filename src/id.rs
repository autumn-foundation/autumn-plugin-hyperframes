//! [`Id`]: a checked HyperFrames id.

use std::fmt;
use std::str::FromStr;

use crate::error::CompositionError;

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
/// assert!(Id::new("card-pro").is_ok());
/// assert!("12".parse::<Id>().is_err());
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Id(String);

impl Id {
    /// Checks `id`.
    ///
    /// # Errors
    ///
    /// Returns [`CompositionError::InvalidId`] when `id` does not match the grammar.
    pub fn new(id: &str) -> Result<Self, CompositionError> {
        if is_valid_id(id) {
            Ok(Self(id.to_owned()))
        } else {
            Err(CompositionError::InvalidId { id: id.to_owned() })
        }
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

impl TryFrom<&str> for Id {
    type Error = CompositionError;

    fn try_from(id: &str) -> Result<Self, Self::Error> {
        Self::new(id)
    }
}

impl FromStr for Id {
    type Err = CompositionError;

    fn from_str(id: &str) -> Result<Self, Self::Err> {
        Self::new(id)
    }
}

impl AsRef<str> for Id {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// True when `id` matches the id grammar.
pub(crate) fn is_valid_id(id: &str) -> bool {
    let mut bytes = id.bytes();
    id.len() <= MAX_ID_LEN
        && bytes.next().is_some_and(|b| b.is_ascii_alphabetic())
        && bytes.all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
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
        assert_eq!(
            Id::new("1"),
            Err(CompositionError::InvalidId { id: "1".into() })
        );
        assert_eq!(Id::try_from("a").map(|i| i.to_string()), Ok("a".to_owned()));
        assert!("a b".parse::<Id>().is_err());
    }
}
