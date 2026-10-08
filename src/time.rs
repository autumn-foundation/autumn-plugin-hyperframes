//! [`Start`]: when a clip starts. Time formatting for `data-*` attributes.

use std::time::Duration;

/// When a clip starts on the composition timeline.
///
/// ```rust
/// use std::time::Duration;
/// use autumn_plugin_hyperframes::Start;
///
/// let at = Start::at(Duration::from_secs(2));
/// let overlap = Start::after("intro").minus(Duration::from_millis(500));
/// # let _ = (at, overlap);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Start {
    /// An absolute time from the composition start.
    At(Duration),
    /// When clip `clip` ends, moved by `offset`.
    After {
        /// The id of the clip in the same composition.
        clip: String,
        /// The time to move the start by.
        offset: Offset,
    },
}

/// A signed move of a relative start.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Offset {
    /// Start later.
    Plus(Duration),
    /// Start earlier (overlap).
    Minus(Duration),
}

impl Default for Start {
    fn default() -> Self {
        Self::At(Duration::ZERO)
    }
}

impl Start {
    /// Starts at `time` from the composition start.
    #[must_use]
    pub const fn at(time: Duration) -> Self {
        Self::At(time)
    }

    /// Starts when clip `clip` ends.
    #[must_use]
    pub fn after(clip: &str) -> Self {
        Self::After {
            clip: clip.to_owned(),
            offset: Offset::Plus(Duration::ZERO),
        }
    }

    /// Moves the start later by `time`.
    #[must_use]
    pub fn plus(self, time: Duration) -> Self {
        let _ = time;
        self
    }

    /// Moves the start earlier by `time`.
    #[must_use]
    pub fn minus(self, time: Duration) -> Self {
        let _ = time;
        self
    }
}

/// Formats `time` in seconds for a `data-*` attribute.
pub(crate) fn seconds(time: Duration) -> String {
    let _ = time;
    String::new()
}

/// The `data-start` value of `start`.
pub(crate) fn start_attr(start: &Start) -> String {
    let _ = start;
    String::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seconds_use_millisecond_precision() {
        assert_eq!(seconds(Duration::ZERO), "0");
        assert_eq!(seconds(Duration::from_secs(3)), "3");
        assert_eq!(seconds(Duration::from_millis(2500)), "2.5");
        assert_eq!(seconds(Duration::from_millis(1250)), "1.25");
        assert_eq!(seconds(Duration::from_millis(5)), "0.005");
        assert_eq!(seconds(Duration::from_micros(1_999_500)), "2");
        assert_eq!(seconds(Duration::from_micros(400)), "0");
        assert_eq!(seconds(Duration::from_secs(86_400)), "86400");
    }

    #[test]
    fn start_attributes_match_the_runtime_grammar() {
        assert_eq!(start_attr(&Start::default()), "0");
        assert_eq!(start_attr(&Start::at(Duration::from_millis(1500))), "1.5");
        assert_eq!(start_attr(&Start::after("intro")), "intro");
        assert_eq!(
            start_attr(&Start::after("intro").plus(Duration::from_secs(2))),
            "intro + 2"
        );
        assert_eq!(
            start_attr(&Start::after("intro").minus(Duration::from_millis(500))),
            "intro - 0.5"
        );
    }

    #[test]
    fn plus_and_minus_add_up() {
        let s = Start::after("a")
            .plus(Duration::from_secs(2))
            .minus(Duration::from_millis(500));
        assert_eq!(start_attr(&s), "a + 1.5");
        let s = Start::after("a")
            .minus(Duration::from_secs(1))
            .plus(Duration::from_millis(250));
        assert_eq!(start_attr(&s), "a - 0.75");
        let s = Start::after("a").plus(Duration::from_secs(1)).minus(Duration::from_secs(1));
        assert_eq!(start_attr(&s), "a");
    }

    #[test]
    fn absolute_start_moves_and_saturates_at_zero() {
        let s = Start::at(Duration::from_secs(2)).plus(Duration::from_secs(1));
        assert_eq!(s, Start::at(Duration::from_secs(3)));
        let s = Start::at(Duration::from_secs(1)).minus(Duration::from_secs(5));
        assert_eq!(s, Start::at(Duration::ZERO));
    }
}
