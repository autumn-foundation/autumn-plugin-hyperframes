//! [`Clip`]: a timed element in a composition.

use std::collections::BTreeMap;
use std::time::Duration;

use autumn_web::Markup;

use crate::time::Start;
use crate::variable::VariableValue;

/// Tells if the video has sound. HyperFrames needs this for each video.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VideoAudio {
    /// Silent footage. Writes `muted`.
    Muted,
    /// The video adds sound to the mix. Writes `data-has-audio="true"`.
    HasAudio,
}

/// A timed element in a composition. `K` is the clip kind.
///
/// Make one with [`Clip::html`], [`Clip::image`], [`Clip::video`],
/// [`Clip::audio`] or [`Clip::composition`]. Each kind has only the setters
/// that apply to it, so `Clip::html(..).volume(..)` does not compile.
///
/// ```rust
/// use std::time::Duration;
/// use autumn_plugin_hyperframes::{Clip, Start, VideoAudio};
/// use autumn_web::html;
///
/// let title = Clip::html("title", html! { h1 { "Hello" } })
///     .duration(Duration::from_secs(3))
///     .class("title");
/// let video = Clip::video("demo", "/static/demo.mp4", VideoAudio::Muted)
///     .start(Start::after("title"))
///     .playback_rate(1.5);
/// # let _ = (title, video);
/// ```
#[derive(Debug, Clone)]
#[must_use]
pub struct Clip<K> {
    pub(crate) id: String,
    pub(crate) start: Start,
    pub(crate) duration: Option<Duration>,
    pub(crate) track: Option<u32>,
    pub(crate) classes: Vec<String>,
    pub(crate) kind: K,
}

/// Clip kind: an HTML block.
#[derive(Debug, Clone)]
pub struct Html {
    pub(crate) content: Markup,
}

/// Clip kind: an image.
#[derive(Debug, Clone)]
pub struct Image {
    pub(crate) src: String,
    pub(crate) alt: String,
    pub(crate) bind_src: Option<String>,
}

/// Clip kind: a video.
#[derive(Debug, Clone)]
pub struct Video {
    pub(crate) src: String,
    pub(crate) audio: VideoAudio,
    pub(crate) media: sealed::MediaOptions,
    pub(crate) bind_src: Option<String>,
}

/// Clip kind: an audio track.
#[derive(Debug, Clone)]
pub struct Audio {
    pub(crate) src: String,
    pub(crate) media: sealed::MediaOptions,
    pub(crate) bind_src: Option<String>,
}

/// Clip kind: a nested composition from another file.
#[derive(Debug, Clone)]
pub struct Nested {
    pub(crate) src: String,
    pub(crate) composition_id: Option<String>,
    pub(crate) playback_start: Option<Duration>,
    pub(crate) playback_rate: Option<f64>,
    pub(crate) size: Option<(u32, u32)>,
    pub(crate) values: BTreeMap<String, VariableValue>,
}

pub(crate) mod sealed {
    use std::time::Duration;

    use super::{Audio, Html, Image, Nested, Video};

    /// Media options for video and audio clips.
    #[derive(Debug, Clone, Default, PartialEq)]
    pub struct MediaOptions {
        pub media_start: Option<Duration>,
        pub playback_rate: Option<f64>,
        pub volume: Option<f64>,
        pub fade_in: Option<Duration>,
        pub fade_out: Option<Duration>,
    }

    /// A clip kind with its data erased.
    #[derive(Debug, Clone)]
    pub enum AnyKind {
        Html(Html),
        Image(Image),
        Video(Video),
        Audio(Audio),
        Nested(Nested),
    }

    /// Seals [`ClipKind`](super::ClipKind).
    pub trait Sealed {
        fn erase(self) -> AnyKind;
    }

    /// Kinds with media options.
    pub trait HasMedia: Sealed {
        fn media_mut(&mut self) -> &mut MediaOptions;
    }

    /// Kinds with a playback rate.
    pub trait HasRate: Sealed {
        fn rate_mut(&mut self) -> &mut Option<f64>;
    }

    /// Kinds with a `src` that a variable can replace.
    pub trait HasSrc: Sealed {
        fn bind_mut(&mut self) -> &mut Option<String>;
    }

    impl Sealed for Html {
        fn erase(self) -> AnyKind {
            AnyKind::Html(self)
        }
    }
    impl Sealed for Image {
        fn erase(self) -> AnyKind {
            AnyKind::Image(self)
        }
    }
    impl Sealed for Video {
        fn erase(self) -> AnyKind {
            AnyKind::Video(self)
        }
    }
    impl Sealed for Audio {
        fn erase(self) -> AnyKind {
            AnyKind::Audio(self)
        }
    }
    impl Sealed for Nested {
        fn erase(self) -> AnyKind {
            AnyKind::Nested(self)
        }
    }

    impl HasMedia for Video {
        fn media_mut(&mut self) -> &mut MediaOptions {
            &mut self.media
        }
    }
    impl HasMedia for Audio {
        fn media_mut(&mut self) -> &mut MediaOptions {
            &mut self.media
        }
    }

    impl HasRate for Video {
        fn rate_mut(&mut self) -> &mut Option<f64> {
            &mut self.media.playback_rate
        }
    }
    impl HasRate for Audio {
        fn rate_mut(&mut self) -> &mut Option<f64> {
            &mut self.media.playback_rate
        }
    }
    impl HasRate for Nested {
        fn rate_mut(&mut self) -> &mut Option<f64> {
            &mut self.playback_rate
        }
    }

    impl HasSrc for Image {
        fn bind_mut(&mut self) -> &mut Option<String> {
            &mut self.bind_src
        }
    }
    impl HasSrc for Video {
        fn bind_mut(&mut self) -> &mut Option<String> {
            &mut self.bind_src
        }
    }
    impl HasSrc for Audio {
        fn bind_mut(&mut self) -> &mut Option<String> {
            &mut self.bind_src
        }
    }
}

/// A clip kind: [`Html`], [`Image`], [`Video`], [`Audio`] or [`Nested`].
///
/// The trait is sealed. Other crates cannot add kinds.
pub trait ClipKind: sealed::Sealed {}

impl<K: sealed::Sealed> ClipKind for K {}

/// A clip with its kind erased. The composition stores these.
#[derive(Debug, Clone)]
pub(crate) struct AnyClip {
    pub(crate) id: String,
    pub(crate) start: Start,
    pub(crate) duration: Option<Duration>,
    pub(crate) track: Option<u32>,
    pub(crate) classes: Vec<String>,
    pub(crate) kind: sealed::AnyKind,
}

impl<K: ClipKind> Clip<K> {
    fn with(id: &str, kind: K) -> Self {
        Self {
            id: id.to_owned(),
            start: Start::default(),
            duration: None,
            track: None,
            classes: Vec::new(),
            kind,
        }
    }

    /// Sets when the clip starts. The default is the composition start.
    pub fn start(mut self, start: Start) -> Self {
        self.start = start;
        self
    }

    /// Sets how long the clip shows.
    pub const fn duration(mut self, duration: Duration) -> Self {
        self.duration = Some(duration);
        self
    }

    /// Sets the Studio timeline lane. It does not change paint order.
    pub const fn track(mut self, track: u32) -> Self {
        self.track = Some(track);
        self
    }

    /// Adds a CSS class.
    pub fn class(mut self, class: &str) -> Self {
        self.classes.push(class.to_owned());
        self
    }

    pub(crate) fn erase(self) -> AnyClip {
        AnyClip {
            id: self.id,
            start: self.start,
            duration: self.duration,
            track: self.track,
            classes: self.classes,
            kind: self.kind.erase(),
        }
    }
}

impl Clip<Html> {
    /// An HTML block clip. It needs a duration.
    pub fn html(id: &str, content: Markup) -> Self {
        Self::with(id, Html { content })
    }
}

impl Clip<Image> {
    /// An image clip. Without a duration it shows for 3 s.
    pub fn image(id: &str, src: &str) -> Self {
        Self::with(
            id,
            Image {
                src: src.to_owned(),
                alt: String::new(),
                bind_src: None,
            },
        )
    }

    /// Sets the alternative text. The default is empty (decorative).
    pub fn alt(mut self, alt: &str) -> Self {
        alt.clone_into(&mut self.kind.alt);
        self
    }
}

impl Clip<Video> {
    /// A video clip. Without a duration it plays to the end of the source.
    pub fn video(id: &str, src: &str, audio: VideoAudio) -> Self {
        Self::with(
            id,
            Video {
                src: src.to_owned(),
                audio,
                media: sealed::MediaOptions::default(),
                bind_src: None,
            },
        )
    }
}

impl Clip<Audio> {
    /// An audio clip. Without a duration it plays to the end of the source.
    pub fn audio(id: &str, src: &str) -> Self {
        Self::with(
            id,
            Audio {
                src: src.to_owned(),
                media: sealed::MediaOptions::default(),
                bind_src: None,
            },
        )
    }
}

impl Clip<Nested> {
    /// A nested composition clip. `src` is the URL of a
    /// [`Composition::template`](crate::Composition::template) file. It needs a duration.
    pub fn composition(id: &str, src: &str) -> Self {
        Self::with(
            id,
            Nested {
                src: src.to_owned(),
                composition_id: None,
                playback_start: None,
                playback_rate: None,
                size: None,
                values: BTreeMap::new(),
            },
        )
    }

    /// Sets the id of the nested composition. The default is the clip id.
    pub fn composition_id(mut self, id: &str) -> Self {
        self.kind.composition_id = Some(id.to_owned());
        self
    }

    /// Sets the first moment of the nested timeline to show.
    pub const fn playback_start(mut self, time: Duration) -> Self {
        self.kind.playback_start = Some(time);
        self
    }

    /// Sets the size of the nested composition in pixels.
    pub const fn size(mut self, width: u32, height: u32) -> Self {
        self.kind.size = Some((width, height));
        self
    }

    /// Sets a variable value for this copy of the nested composition.
    pub fn value(mut self, name: &str, value: impl Into<VariableValue>) -> Self {
        self.kind.values.insert(name.to_owned(), value.into());
        self
    }
}

impl<K: sealed::HasMedia> Clip<K> {
    /// Sets the time in the source file where the clip starts (`data-media-start`).
    pub fn media_start(mut self, time: Duration) -> Self {
        self.kind.media_mut().media_start = Some(time);
        self
    }

    /// Sets the gain. `1.0` is 0 dB. The range is `0.0..=3.98` (+12 dB).
    pub fn volume(mut self, volume: f64) -> Self {
        self.kind.media_mut().volume = Some(volume);
        self
    }

    /// Fades the sound in over the first `time` of the clip.
    pub fn fade_in(mut self, time: Duration) -> Self {
        self.kind.media_mut().fade_in = Some(time);
        self
    }

    /// Fades the sound out over the last `time` of the clip.
    pub fn fade_out(mut self, time: Duration) -> Self {
        self.kind.media_mut().fade_out = Some(time);
        self
    }
}

impl<K: sealed::HasRate> Clip<K> {
    /// Sets the playback speed. The range is `0.1..=10`.
    pub fn playback_rate(mut self, rate: f64) -> Self {
        *self.kind.rate_mut() = Some(rate);
        self
    }
}

impl<K: sealed::HasSrc> Clip<K> {
    /// Lets the variable `variable` replace the `src` (`data-var-src`).
    pub fn bind_src(mut self, variable: &str) -> Self {
        *self.kind.bind_mut() = Some(variable.to_owned());
        self
    }
}
