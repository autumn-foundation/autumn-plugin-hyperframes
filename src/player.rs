//! [`Player`]: the `<hyperframes-player>` element.

use std::time::Duration;

use autumn_web::{Markup, html};

use crate::assets::runtime_url;
use crate::composition::Composition;
use crate::time::seconds;
use crate::url::is_safe_url;

/// The lowest playback rate that the player accepts.
const MIN_RATE: f64 = 0.1;

/// The highest playback rate that the player accepts.
const MAX_RATE: f64 = 5.0;

/// The video file type for [`Player::video`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VideoType {
    /// `video/mp4`.
    Mp4,
    /// `video/webm`.
    WebM,
    /// `video/ogg`.
    Ogg,
}

impl VideoType {
    /// The MIME type.
    #[must_use]
    pub const fn mime(self) -> &'static str {
        match self {
            Self::Mp4 => "video/mp4",
            Self::WebM => "video/webm",
            Self::Ogg => "video/ogg",
        }
    }
}

/// Who shows the shader transition loading UI (`shader-loading`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShaderLoading {
    /// The composition shows it (the player default).
    Composition,
    /// The player shows it.
    Player,
    /// Nobody shows it.
    None,
}

/// What the player does when the user prefers reduced motion (`data-hf-reduced`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ReducedMotion {
    /// Do not autoplay and do not play in view (the default). Writes nothing.
    #[default]
    Skip,
    /// Autoplay and play in view all the same.
    Animate,
}

impl ShaderLoading {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Composition => "composition",
            Self::Player => "player",
            Self::None => "none",
        }
    }
}

#[derive(Debug, Clone)]
enum Source {
    Srcdoc { html: String },
    Url(String),
    Video { src: String, kind: VideoType },
}

/// The `<hyperframes-player>` element.
#[derive(Debug, Clone)]
#[must_use]
pub struct Player {
    source: Source,
    size: Option<(u32, u32)>,
    id: Option<String>,
    classes: Vec<String>,
    label: Option<String>,
    flags: Vec<&'static str>,
    poster: Option<String>,
    playback_rate: Option<f64>,
    volume: Option<f64>,
    range: Option<(Duration, Duration)>,
    shader_loading: Option<ShaderLoading>,
    reduced: ReducedMotion,
}

impl Player {
    const fn with(source: Source) -> Self {
        Self {
            source,
            size: None,
            id: None,
            classes: Vec::new(),
            label: None,
            flags: Vec::new(),
            poster: None,
            playback_rate: None,
            volume: None,
            range: None,
            shader_loading: None,
            reduced: ReducedMotion::Skip,
        }
    }

    /// Plays `composition` from a `srcdoc` attribute.
    pub fn composition(composition: &Composition) -> Self {
        let mut player = Self::with(Source::Srcdoc {
            html: composition.srcdoc_html(),
        });
        player.size = Some((composition.width(), composition.height()));
        player
    }

    /// Plays the composition page at `url`.
    pub fn src(url: &str) -> Self {
        Self::with(Source::Url(url.to_owned()))
    }

    /// Plays the video file at `url`.
    pub fn video(url: &str, kind: VideoType) -> Self {
        Self::with(Source::Video {
            src: url.to_owned(),
            kind,
        })
    }

    /// Sets the element id. [`PlayerControl`](crate::PlayerControl) targets it.
    pub fn id(mut self, id: &str) -> Self {
        self.id = Some(id.to_owned());
        self
    }

    /// Adds a CSS class.
    pub fn class(mut self, class: &str) -> Self {
        self.classes.push(class.to_owned());
        self
    }

    /// Sets an accessible name.
    pub fn label(mut self, label: &str) -> Self {
        self.label = Some(label.to_owned());
        self
    }

    /// Sets the native size in pixels.
    pub const fn size(mut self, width: u32, height: u32) -> Self {
        self.size = Some((width, height));
        self
    }

    fn flag(mut self, name: &'static str) -> Self {
        self.flags.push(name);
        self
    }

    /// Shows the controls.
    pub fn controls(self) -> Self {
        self.flag("controls")
    }

    /// Mutes the sound.
    pub fn muted(self) -> Self {
        self.flag("muted")
    }

    /// Plays when ready.
    pub fn autoplay(self) -> Self {
        self.flag("autoplay")
    }

    /// Plays again from the start at the end.
    pub fn looped(self) -> Self {
        self.flag("loop")
    }

    /// Shows an image before playback.
    pub fn poster(mut self, url: &str) -> Self {
        self.poster = Some(url.to_owned());
        self
    }

    /// Sets the speed.
    pub const fn playback_rate(mut self, rate: f64) -> Self {
        self.playback_rate = Some(rate);
        self
    }

    /// Sets the volume.
    pub const fn volume(mut self, volume: f64) -> Self {
        self.volume = Some(volume);
        self
    }

    /// Plays only `[start, end)`.
    pub const fn range(mut self, start: Duration, end: Duration) -> Self {
        self.range = Some((start, end));
        self
    }

    /// Forces mute.
    pub fn audio_locked(self) -> Self {
        self.flag("audio-locked")
    }

    /// Opaque origin sandbox.
    pub fn opaque_sandbox(self) -> Self {
        self.flag("sandbox-origin")
    }

    /// Low power idle.
    pub fn low_power_idle(self) -> Self {
        self.flag("low-power-idle")
    }

    /// No click to play.
    pub fn disable_click_to_play(self) -> Self {
        self.flag("disable-click-to-play")
    }

    /// No loading card.
    pub fn hide_loading_ui(self) -> Self {
        self.flag("assets-loading-ui")
    }

    /// Shader loading UI owner.
    pub const fn shader_loading(mut self, owner: ShaderLoading) -> Self {
        self.shader_loading = Some(owner);
        self
    }

    /// Plays in view.
    pub fn in_view(self) -> Self {
        self.flag("data-hf-in-view")
    }

    /// Reduced motion behavior.
    pub const fn reduced_motion(mut self, reduced: ReducedMotion) -> Self {
        self.reduced = reduced;
        self
    }

    /// Renders the element.
    ///
    /// The element always has `runtime-src` with the vendored runtime URL, so
    /// the player never loads the runtime from a CDN. An option that you do
    /// not set writes no attribute.
    #[must_use]
    pub fn render(&self) -> Markup {
        let (src, srcdoc, kind) = match &self.source {
            Source::Srcdoc { html } => (None, Some(html.as_str()), None),
            Source::Url(url) => (safe(url), None, None),
            Source::Video { src, kind } => (safe(src), None, Some(kind.mime())),
        };
        let mut classes: Vec<&str> = self.size.and_then(ratio_class).into_iter().collect();
        classes.extend(self.classes.iter().map(String::as_str));
        let class = (!classes.is_empty()).then(|| classes.join(" "));
        let has = |flag: &str| self.flags.contains(&flag);
        let rate = self
            .playback_rate
            .filter(|r| r.is_finite())
            .map(|r| num(r.clamp(MIN_RATE, MAX_RATE)));
        let volume = self
            .volume
            .filter(|v| v.is_finite())
            .map(|v| num(v.clamp(0.0, 1.0)));
        html! {
            hyperframes-player id=[self.id.as_deref()] class=[class]
                role=[self.label.as_ref().map(|_| "group")] aria-label=[self.label.as_deref()]
                src=[src] srcdoc=[srcdoc] type=[kind]
                width=[self.size.map(|s| s.0)] height=[self.size.map(|s| s.1)]
                runtime-src=(runtime_url())
                controls[has("controls")] muted[has("muted")] autoplay[has("autoplay")]
                "loop"[has("loop")]
                poster=[self.poster.as_deref().and_then(safe)]
                playback-rate=[rate] volume=[volume]
                range-start=[self.range.map(|r| seconds(r.0))]
                range-end=[self.range.map(|r| seconds(r.1))]
                audio-locked[has("audio-locked")]
                sandbox-origin=[has("sandbox-origin").then_some("opaque")]
                low-power-idle[has("low-power-idle")]
                disable-click-to-play[has("disable-click-to-play")]
                assets-loading-ui=[has("assets-loading-ui").then_some("none")]
                shader-loading=[self.shader_loading.map(ShaderLoading::as_str)]
                data-hf-in-view[has("data-hf-in-view")]
                data-hf-reduced=[(self.reduced == ReducedMotion::Animate).then_some("animate")] {}
        }
    }
}

/// `url` when it is safe, else `None` (no attribute).
fn safe(url: &str) -> Option<&str> {
    is_safe_url(url).then_some(url)
}

fn num(value: f64) -> String {
    format!("{value}")
}

/// The aspect ratio class for sizes that `hyperframes.css` knows.
const fn ratio_class((width, height): (u32, u32)) -> Option<&'static str> {
    const fn gcd(a: u32, b: u32) -> u32 {
        if b == 0 { a } else { gcd(b, a % b) }
    }
    let d = gcd(width, height);
    if d == 0 {
        return None;
    }
    match (width / d, height / d) {
        (9, 16) => Some("hf-ratio-9x16"),
        (1, 1) => Some("hf-ratio-1x1"),
        (4, 5) => Some("hf-ratio-4x5"),
        (4, 3) => Some("hf-ratio-4x3"),
        (7, 3) => Some("hf-ratio-21x9"),
        _ => None,
    }
}

impl maud::Render for Player {
    fn render(&self) -> Markup {
        Self::render(self)
    }
}
