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
    ///
    /// An absolute start saturates at the largest `Duration`.
    #[must_use]
    pub fn plus(self, time: Duration) -> Self {
        match self {
            Self::At(at) => Self::At(at.saturating_add(time)),
            Self::After { clip, offset } => Self::After {
                clip,
                offset: offset.shift(time, true),
            },
        }
    }

    /// Moves the start earlier by `time`.
    ///
    /// An absolute start stops at zero. A relative start can go before the end
    /// of its clip (overlap). The runtime clamps the result at zero.
    #[must_use]
    pub fn minus(self, time: Duration) -> Self {
        match self {
            Self::At(at) => Self::At(at.saturating_sub(time)),
            Self::After { clip, offset } => Self::After {
                clip,
                offset: offset.shift(time, false),
            },
        }
    }
}

impl Offset {
    /// Adds `time` (`later`) or subtracts it.
    fn shift(self, time: Duration, later: bool) -> Self {
        match (self, later) {
            (Self::Plus(d), true) => Self::Plus(d.saturating_add(time)),
            (Self::Minus(d), false) => Self::Minus(d.saturating_add(time)),
            (Self::Plus(d), false) | (Self::Minus(d), true) => {
                let flipped = matches!(self, Self::Plus(_));
                if time > d {
                    if flipped {
                        Self::Minus(time - d)
                    } else {
                        Self::Plus(time - d)
                    }
                } else if flipped {
                    Self::Plus(d - time)
                } else {
                    Self::Minus(d - time)
                }
            }
        }
    }

    /// The offset in whole milliseconds, with its sign.
    pub(crate) fn signed_millis(self) -> i128 {
        match self {
            Self::Plus(d) => i128::from(millis(d)),
            Self::Minus(d) => -i128::from(millis(d)),
        }
    }
}

/// `time` rounded to the nearest millisecond.
pub(crate) fn millis(time: Duration) -> u64 {
    let ms = (time.as_nanos() + 500_000) / 1_000_000;
    u64::try_from(ms).unwrap_or(u64::MAX)
}

/// Formats milliseconds as seconds: `2500` gives `2.5`, `3000` gives `3`.
pub(crate) fn format_millis(ms: u128) -> String {
    let (whole, frac) = (ms / 1000, ms % 1000);
    if frac == 0 {
        whole.to_string()
    } else {
        let digits = format!("{frac:03}");
        format!("{whole}.{}", digits.trim_end_matches('0'))
    }
}

/// Formats `time` in seconds for a `data-*` attribute.
///
/// The value has at most three decimals and no exponent, so the runtime
/// number grammar accepts it. Times round to the nearest millisecond.
pub(crate) fn seconds(time: Duration) -> String {
    format_millis(u128::from(millis(time)))
}

/// The `data-start` value of `start`.
pub(crate) fn start_attr(start: &Start) -> String {
    match start {
        Start::At(at) => seconds(*at),
        Start::After { clip, offset } => {
            let ms = offset.signed_millis();
            match ms.cmp(&0) {
                std::cmp::Ordering::Equal => clip.clone(),
                std::cmp::Ordering::Greater => {
                    format!("{clip} + {}", format_millis(ms.unsigned_abs()))
                }
                std::cmp::Ordering::Less => {
                    format!("{clip} - {}", format_millis(ms.unsigned_abs()))
                }
            }
        }
    }
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
        let s = Start::after("a")
            .plus(Duration::from_secs(1))
            .minus(Duration::from_secs(1));
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
