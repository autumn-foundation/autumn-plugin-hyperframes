//! Build errors.

use std::fmt;

/// One problem that [`CompositionBuilder::build`](crate::CompositionBuilder::build) found.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
#[non_exhaustive]
pub enum CompositionError {
    /// An id does not match the id grammar (spec S1).
    #[error("`{id}` is not a valid id")]
    InvalidId {
        /// The bad id.
        id: String,
    },
    /// Two elements use the same id (S2).
    #[error("the id `{id}` is used more than one time")]
    DuplicateId {
        /// The repeated id.
        id: String,
    },
    /// The width or the height is not in `1..=16384` (S3).
    #[error("the size {width}x{height} of `{owner}` is not in 1..=16384")]
    InvalidSize {
        /// The composition or clip id.
        owner: String,
        /// The width in pixels.
        width: u32,
        /// The height in pixels.
        height: u32,
    },
    /// A timeline-free composition has no duration (S4).
    #[error("the composition `{id}` has no timeline, so it needs a duration")]
    MissingDuration {
        /// The composition id.
        id: String,
    },
    /// A duration is less than one millisecond (S4).
    #[error("the duration of `{id}` is less than 1 ms")]
    ZeroDuration {
        /// The composition or clip id.
        id: String,
    },
    /// An HTML or nested composition clip has no duration (S5).
    #[error("the clip `{clip}` needs a duration")]
    ClipNeedsDuration {
        /// The clip id.
        clip: String,
    },
    /// A relative start names a clip that is not in the composition (S6).
    #[error(
        "the clip `{clip}` starts after `{reference}`, but that clip is not in the composition"
    )]
    UnknownReference {
        /// The clip id.
        clip: String,
        /// The missing clip id.
        reference: String,
    },
    /// A relative start names its own clip (S6).
    #[error("the clip `{clip}` starts after itself")]
    SelfReference {
        /// The clip id.
        clip: String,
    },
    /// Relative starts make a loop (S6).
    #[error("the relative starts of {} make a loop", .clips.join(", "))]
    ReferenceCycle {
        /// The clips in the loop, in composition order.
        clips: Vec<String>,
    },
    /// A volume is not finite or not in `0..=3.98` (S7).
    #[error("the volume {volume} of `{clip}` is not in 0..=3.98")]
    VolumeOutOfRange {
        /// The clip id.
        clip: String,
        /// The bad volume.
        volume: f64,
    },
    /// A playback rate is not finite or not in `0.1..=10` (S7).
    #[error("the playback rate {rate} of `{clip}` is not in 0.1..=10")]
    PlaybackRateOutOfRange {
        /// The clip id.
        clip: String,
        /// The bad rate.
        rate: f64,
    },
    /// A URL is empty or can run script (S8).
    #[error("the URL `{url}` of `{owner}` is empty or not safe")]
    UnsafeUrl {
        /// The composition or clip id.
        owner: String,
        /// The bad URL.
        url: String,
    },
    /// Two variables use the same id (S9).
    #[error("the variable `{variable}` is declared more than one time")]
    DuplicateVariable {
        /// The variable id.
        variable: String,
    },
    /// A variable declaration is not valid (S9).
    #[error("the variable `{variable}` is not valid: {reason}")]
    InvalidVariable {
        /// The variable id.
        variable: String,
        /// Why it is not valid.
        reason: &'static str,
    },
    /// A value for a nested composition is not valid (S9).
    #[error("the value `{name}` of `{clip}` is not valid: {reason}")]
    InvalidValue {
        /// The clip id.
        clip: String,
        /// The variable name.
        name: String,
        /// Why it is not valid.
        reason: &'static str,
    },
    /// One nested composition with relative starts plays more than one time (S12).
    #[error("`{src}` plays more than one time, but it has relative starts")]
    RepeatedRelativeNested {
        /// The nested composition URL.
        src: String,
    },
    /// HTML content has an element id that is also a clip id (S13).
    #[error("the content of `{clip}` has the id `{id}`, which is also a clip id")]
    ContentIdClash {
        /// The clip with the content.
        clip: String,
        /// The id that clashes.
        id: String,
    },
    /// `bind_src` names a variable that is not declared (S10).
    #[error("the clip `{clip}` binds the variable `{variable}`, but it is not declared")]
    UnknownVariable {
        /// The clip id.
        clip: String,
        /// The missing variable id.
        variable: String,
    },
}

/// All problems that `build()` found. It holds at least one error.
#[derive(Debug, Clone, PartialEq)]
pub struct BuildError(pub(crate) Vec<CompositionError>);

impl BuildError {
    /// The errors, in the order that `build()` found them.
    #[must_use]
    pub fn errors(&self) -> &[CompositionError] {
        &self.0
    }
}

impl fmt::Display for BuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let n = self.0.len();
        write!(
            f,
            "{n} composition error{}: ",
            if n == 1 { "" } else { "s" }
        )?;
        for (i, error) in self.0.iter().enumerate() {
            if i > 0 {
                f.write_str("; ")?;
            }
            write!(f, "{error}")?;
        }
        Ok(())
    }
}

impl std::error::Error for BuildError {}
