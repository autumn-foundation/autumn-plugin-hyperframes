//! [`Composition`]: a checked HyperFrames composition, and its builder.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::time::Duration;

use autumn_web::{Markup, html};
use serde_json::Value;

use crate::assets::{COMPOSITION_CSS, HYPERFRAMES_ASSETS, RUNTIME_JS};
use crate::clip::sealed::{AnyKind, MediaOptions};
use crate::clip::{AnyClip, Clip, ClipKind, VideoAudio};
use crate::error::{BuildError, CompositionError};
use crate::id::{Id, is_valid_id};
use crate::time::{StartKind, millis, seconds, start_attr};
use crate::url::is_safe_url;
use crate::variable::{Variable, VariableKind, VariableValue};

/// The default composition width in pixels.
pub const DEFAULT_WIDTH: u32 = 1920;

/// The default composition height in pixels.
pub const DEFAULT_HEIGHT: u32 = 1080;

/// The largest width or height in pixels.
pub const MAX_SIZE: u32 = 16_384;

/// The volume range (`1.0` is 0 dB, `3.98` is +12 dB).
const VOLUME: std::ops::RangeInclusive<f64> = 0.0..=3.98;

/// The clip playback rate range.
const RATE: std::ops::RangeInclusive<f64> = 0.1..=10.0;

/// How long an image without a duration shows (the runtime default).
const IMAGE_DEFAULT_MS: u64 = 3000;

/// Builds a [`Composition`]. Make one with [`Composition::builder`].
#[derive(Debug, Clone)]
#[must_use]
pub struct CompositionBuilder {
    id: String,
    width: u32,
    height: u32,
    duration: Option<Duration>,
    timeline: bool,
    title: Option<String>,
    stylesheets: Vec<String>,
    scripts: Vec<String>,
    variables: Vec<Variable>,
    clips: Vec<AnyClip>,
}

/// A checked HyperFrames composition.
///
/// Only [`CompositionBuilder::build`] makes one, so every `Composition`
/// obeys the rules S1 to S10 in `docs/plan.md`. It renders in three forms:
///
/// - [`document`](Self::document): a full page. Serve it at a URL.
/// - [`template`](Self::template): a `<template>` file for a nested composition.
/// - [`fragment`](Self::fragment): the root element only.
///
/// [`Player::composition`](crate::Player::composition) embeds it in a page.
///
/// ```rust
/// use std::time::Duration;
/// use autumn_plugin_hyperframes::{Clip, Composition, Start};
/// use autumn_web::html;
///
/// let intro = Composition::builder("intro")
///     .duration(Duration::from_secs(4))
///     .clip(Clip::html("title", html! { h1 { "Hello" } }).duration(Duration::from_secs(2)))
///     .clip(Clip::html("tagline", html! { p { "World" } })
///         .start(Start::after("title"))
///         .duration(Duration::from_secs(2)))
///     .build()?;
/// assert_eq!(intro.resolved_start("tagline"), Some(Duration::from_secs(2)));
/// # Ok::<(), autumn_plugin_hyperframes::BuildError>(())
/// ```
#[derive(Debug, Clone)]
pub struct Composition {
    id: Id,
    width: u32,
    height: u32,
    duration: Option<Duration>,
    timeline: bool,
    title: Option<String>,
    stylesheets: Vec<String>,
    scripts: Vec<String>,
    variables: Vec<Variable>,
    clips: Vec<AnyClip>,
    /// Resolved (start, end) in milliseconds, one entry for each clip.
    timing: Vec<(Option<u64>, Option<u64>)>,
}

impl Composition {
    /// Starts a composition with the id `id`, size 1920x1080 and no clips.
    pub fn builder(id: &str) -> CompositionBuilder {
        CompositionBuilder {
            id: id.to_owned(),
            width: DEFAULT_WIDTH,
            height: DEFAULT_HEIGHT,
            duration: None,
            timeline: false,
            title: None,
            stylesheets: Vec::new(),
            scripts: Vec::new(),
            variables: Vec::new(),
            clips: Vec::new(),
        }
    }

    /// The composition id.
    #[must_use]
    pub const fn id(&self) -> &Id {
        &self.id
    }

    /// The width in pixels.
    #[must_use]
    pub const fn width(&self) -> u32 {
        self.width
    }

    /// The height in pixels.
    #[must_use]
    pub const fn height(&self) -> u32 {
        self.height
    }

    /// The duration, if set.
    #[must_use]
    pub const fn duration(&self) -> Option<Duration> {
        self.duration
    }

    /// The clip ids, in composition order.
    #[must_use]
    pub fn clip_ids(&self) -> Vec<&str> {
        self.clips.iter().map(|c| c.id.as_str()).collect()
    }

    /// The start of clip `id` on the timeline, if it is known (spec S11).
    ///
    /// The start is unknown when it follows a video or audio clip without a
    /// duration (the runtime reads the source length). Times are in whole
    /// milliseconds, as in the HTML.
    #[must_use]
    pub fn resolved_start(&self, id: &str) -> Option<Duration> {
        self.timing_of(id).0.map(Duration::from_millis)
    }

    /// The end of clip `id` on the timeline, if it is known.
    ///
    /// An image without a duration ends 3 s after its start.
    #[must_use]
    pub fn resolved_end(&self, id: &str) -> Option<Duration> {
        self.timing_of(id).1.map(Duration::from_millis)
    }

    fn timing_of(&self, id: &str) -> (Option<u64>, Option<u64>) {
        self.clips
            .iter()
            .position(|c| c.id == id)
            .map_or((None, None), |i| self.timing[i])
    }

    /// Renders the root element and its clips.
    ///
    /// The root carries `data-composition-variables` when the composition
    /// declares variables.
    #[must_use]
    pub fn fragment(&self) -> Markup {
        self.root(true, true)
    }

    /// Renders a full HTML page that loads the runtime.
    ///
    /// Serve it from a route, then play it with [`Player::src`](crate::Player::src).
    /// The page links the composition stylesheet and the runtime with SRI.
    /// Your stylesheets go in `<head>`. Your scripts run at the end of `<body>`.
    #[must_use]
    pub fn document(&self) -> Markup {
        self.page(true)
    }

    /// Renders a `<template>` file for use as a nested composition.
    ///
    /// Serve it from a route and point [`Clip::composition`] at the URL. The
    /// parent page loads the runtime, so the template does not.
    #[must_use]
    pub fn template(&self) -> Markup {
        html! {
            template id=(format!("{}-template", self.id)) {
                (HYPERFRAMES_ASSETS.stylesheet_tag(COMPOSITION_CSS))
                @for href in &self.stylesheets {
                    link rel="stylesheet" href=(href);
                }
                (self.root(true, false))
                @for src in &self.scripts {
                    script src=(src) {}
                }
            }
        }
    }

    /// The page HTML for a player `srcdoc`.
    ///
    /// It is [`document`](Self::document) without the runtime tag: the player
    /// puts the runtime from its `runtime-src` attribute into the page.
    #[must_use]
    pub fn srcdoc_html(&self) -> String {
        self.page(false).into_string()
    }

    fn page(&self, with_runtime: bool) -> Markup {
        let title = self.title.as_deref().unwrap_or_else(|| self.id.as_str());
        html! {
            (maud::DOCTYPE)
            html lang="en" data-composition-variables=[self.variables_json()] {
                head {
                    meta charset="utf-8";
                    meta name="viewport" content=(format!("width={}, height={}", self.width, self.height));
                    title { (title) }
                    (HYPERFRAMES_ASSETS.stylesheet_tag(COMPOSITION_CSS))
                    @for href in &self.stylesheets {
                        link rel="stylesheet" href=(href);
                    }
                    @if with_runtime {
                        (HYPERFRAMES_ASSETS.script_tag(RUNTIME_JS))
                    }
                }
                body {
                    (self.root(false, true))
                    @for src in &self.scripts {
                        script src=(src) {}
                    }
                }
            }
        }
    }

    /// The root element.
    ///
    /// `top_level` adds the element `id` and `data-start="0"`. A nested root
    /// has neither: its host element has the id and the start.
    fn root(&self, with_variables: bool, top_level: bool) -> Markup {
        let variables = if with_variables {
            self.variables_json()
        } else {
            None
        };
        html! {
            div id=[top_level.then_some(self.id.as_str())] class="hf-root" data-composition-id=(self.id)
                data-start=[top_level.then_some("0")]
                data-duration=[self.duration.map(seconds)]
                data-width=(self.width) data-height=(self.height)
                data-no-timeline[!self.timeline]
                data-composition-variables=[variables] {
                @for clip in &self.clips {
                    (clip_markup(clip))
                }
            }
        }
    }

    fn variables_json(&self) -> Option<String> {
        if self.variables.is_empty() {
            return None;
        }
        let declarations: Vec<BTreeMap<&str, Value>> =
            self.variables.iter().map(declaration).collect();
        serde_json::to_string(&declarations).ok()
    }
}

/// The JSON declaration of one variable. Keys sort, so output is stable.
fn declaration(variable: &Variable) -> BTreeMap<&'static str, Value> {
    let (kind, default) = match &variable.kind {
        VariableKind::String(v) => ("string", Value::from(v.as_str())),
        VariableKind::Number(v) => ("number", Value::from(*v)),
        VariableKind::Color(v) => ("color", Value::from(v.as_str())),
        VariableKind::Boolean(v) => ("boolean", Value::from(*v)),
        VariableKind::Enum { default, .. } => ("enum", Value::from(default.as_str())),
        VariableKind::Font(v) => ("font", Value::from(v.as_str())),
        VariableKind::Image(v) => ("image", Value::from(v.as_str())),
    };
    let mut map = BTreeMap::new();
    map.insert("id", Value::from(variable.id.as_str()));
    map.insert("type", Value::from(kind));
    map.insert(
        "label",
        Value::from(variable.label.as_deref().unwrap_or(&variable.id)),
    );
    map.insert("default", default);
    if let Some(description) = &variable.description {
        map.insert("description", Value::from(description.as_str()));
    }
    if let VariableKind::Enum { options, .. } = &variable.kind {
        let options: Vec<Value> = options
            .iter()
            .map(|o| serde_json::json!({ "label": o, "value": o }))
            .collect();
        map.insert("options", Value::from(options));
    }
    map
}

fn value_json(value: &VariableValue) -> Value {
    match value {
        VariableValue::String(v) => Value::from(v.as_str()),
        VariableValue::Integer(v) => Value::from(*v),
        VariableValue::Number(v) => Value::from(*v),
        VariableValue::Boolean(v) => Value::from(*v),
    }
}

/// The `class` value: `clip` (for visual clips) plus the user classes.
fn class_list(classes: &[String], visual: bool) -> Option<String> {
    let mut all: Vec<&str> = Vec::with_capacity(classes.len() + 1);
    if visual {
        all.push("clip");
    }
    all.extend(classes.iter().map(String::as_str));
    (!all.is_empty()).then(|| all.join(" "))
}

fn num(value: f64) -> String {
    format!("{value}")
}

fn clip_markup(clip: &AnyClip) -> Markup {
    let start = start_attr(&clip.start);
    let duration = clip.duration.map(seconds);
    let track = clip.track;
    match &clip.kind {
        AnyKind::Html(h) => html! {
            div id=(clip.id) class=[class_list(&clip.classes, true)]
                data-start=(start) data-duration=[duration] data-track-index=[track] {
                (h.content)
            }
        },
        AnyKind::Image(i) => html! {
            img id=(clip.id) class=[class_list(&clip.classes, true)] src=(i.src) alt=(i.alt)
                data-start=(start) data-duration=[duration] data-track-index=[track]
                data-var-src=[i.bind_src.as_deref()];
        },
        AnyKind::Video(v) => {
            let m = &v.media;
            html! {
                video id=(clip.id) class=[class_list(&clip.classes, true)] src=(v.src)
                    data-start=(start) data-duration=[duration] data-track-index=[track]
                    data-media-start=[m.media_start.map(seconds)]
                    data-playback-rate=[m.playback_rate.map(num)]
                    data-volume=[m.volume.map(num)]
                    data-fade-in=[m.fade_in.map(seconds)]
                    data-fade-out=[m.fade_out.map(seconds)]
                    muted[v.audio == VideoAudio::Muted]
                    data-has-audio=[(v.audio == VideoAudio::HasAudio).then_some("true")]
                    playsinline
                    data-var-src=[v.bind_src.as_deref()] {}
            }
        }
        AnyKind::Audio(a) => {
            let m = &a.media;
            html! {
                audio id=(clip.id) class=[class_list(&clip.classes, false)] src=(a.src)
                    data-start=(start) data-duration=[duration] data-track-index=[track]
                    data-media-start=[m.media_start.map(seconds)]
                    data-playback-rate=[m.playback_rate.map(num)]
                    data-volume=[m.volume.map(num)]
                    data-fade-in=[m.fade_in.map(seconds)]
                    data-fade-out=[m.fade_out.map(seconds)]
                    data-var-src=[a.bind_src.as_deref()] {}
            }
        }
        AnyKind::Nested(n) => {
            let values = (!n.values.is_empty())
                .then(|| {
                    let map: BTreeMap<&str, Value> = n
                        .values
                        .iter()
                        .map(|(k, v)| (k.as_str(), value_json(v)))
                        .collect();
                    serde_json::to_string(&map).ok()
                })
                .flatten();
            html! {
                div id=(clip.id) class=[class_list(&clip.classes, false)]
                    data-composition-id=(n.composition_id.as_deref().unwrap_or(&clip.id))
                    data-composition-src=(n.src)
                    data-start=(start) data-duration=[duration] data-track-index=[track]
                    data-playback-start=[n.playback_start.map(seconds)]
                    data-playback-rate=[n.playback_rate.map(num)]
                    data-width=[n.size.map(|s| s.0)] data-height=[n.size.map(|s| s.1)]
                    data-variable-values=[values] {}
            }
        }
    }
}

impl CompositionBuilder {
    /// Sets the frame size in pixels. Each side must be in `1..=16384`.
    pub const fn size(mut self, width: u32, height: u32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// Sets the total duration.
    pub const fn duration(mut self, duration: Duration) -> Self {
        self.duration = Some(duration);
        self
    }

    /// Says that a script registers a timeline (`window.__timelines[id]`).
    ///
    /// Without this, the root gets `data-no-timeline` and needs a duration.
    pub const fn with_timeline(mut self) -> Self {
        self.timeline = true;
        self
    }

    /// Sets the page title of [`Composition::document`]. The default is the id.
    pub fn title(mut self, title: &str) -> Self {
        self.title = Some(title.to_owned());
        self
    }

    /// Adds a stylesheet URL. Put CSS animations here; the runtime seeks them.
    pub fn stylesheet(mut self, href: &str) -> Self {
        self.stylesheets.push(href.to_owned());
        self
    }

    /// Adds a script URL. Scripts run at the end of `<body>`, in order.
    ///
    /// The builder has no inline scripts: the Autumn CSP blocks them.
    pub fn script(mut self, src: &str) -> Self {
        self.scripts.push(src.to_owned());
        self
    }

    /// Declares a variable.
    pub fn variable(mut self, variable: Variable) -> Self {
        self.variables.push(variable);
        self
    }

    /// Adds a clip.
    pub fn clip<K: ClipKind>(mut self, clip: Clip<K>) -> Self {
        self.clips.push(clip.erase());
        self
    }

    /// Checks the composition. Returns all errors that it finds.
    ///
    /// # Errors
    ///
    /// Returns a [`BuildError`] when a rule S1 to S10 in `docs/plan.md` fails.
    pub fn build(self) -> Result<Composition, BuildError> {
        let mut errors = Vec::new();
        self.check_root(&mut errors);
        self.check_variables(&mut errors);
        let declared: HashSet<&str> = self.variables.iter().map(|v| v.id.as_str()).collect();
        for clip in &self.clips {
            check_clip(clip, &declared, &mut errors);
        }
        self.check_unique_ids(&mut errors);
        self.check_references(&mut errors);
        if !errors.is_empty() {
            return Err(BuildError(errors));
        }
        let id = Id::new(&self.id).map_err(|e| BuildError(vec![e]))?;
        let timing = resolve_timing(&self.clips);
        Ok(Composition {
            id,
            width: self.width,
            height: self.height,
            duration: self.duration,
            timeline: self.timeline,
            title: self.title,
            stylesheets: self.stylesheets,
            scripts: self.scripts,
            variables: self.variables,
            clips: self.clips,
            timing,
        })
    }

    fn check_root(&self, errors: &mut Vec<CompositionError>) {
        check_id(&self.id, errors);
        let in_range = |side: u32| (1..=MAX_SIZE).contains(&side);
        if !in_range(self.width) || !in_range(self.height) {
            errors.push(CompositionError::InvalidSize {
                owner: self.id.clone(),
                width: self.width,
                height: self.height,
            });
        }
        match self.duration {
            None if !self.timeline => errors.push(CompositionError::MissingDuration {
                id: self.id.clone(),
            }),
            Some(d) if millis(d) == 0 => errors.push(CompositionError::ZeroDuration {
                id: self.id.clone(),
            }),
            _ => {}
        }
        for url in self.stylesheets.iter().chain(&self.scripts) {
            check_url(&self.id, url, errors);
        }
    }

    fn check_variables(&self, errors: &mut Vec<CompositionError>) {
        let mut seen = HashSet::new();
        for variable in &self.variables {
            if !is_valid_id(&variable.id) {
                errors.push(CompositionError::InvalidId {
                    id: variable.id.clone(),
                });
            } else if !seen.insert(variable.id.as_str()) {
                errors.push(CompositionError::DuplicateVariable {
                    variable: variable.id.clone(),
                });
            }
            let reason = match &variable.kind {
                VariableKind::Number(n) if !n.is_finite() => {
                    Some("the default is not a finite number")
                }
                VariableKind::Enum { options, .. } if options.is_empty() => {
                    Some("an enum needs at least one option")
                }
                VariableKind::Enum { options, default } if !options.contains(default) => {
                    Some("the default is not one of the options")
                }
                _ => None,
            };
            if let Some(reason) = reason {
                errors.push(CompositionError::InvalidVariable {
                    variable: variable.id.clone(),
                    reason,
                });
            }
        }
    }

    /// S2: the root id and the clip ids are unique. A nested composition id
    /// can repeat (the runtime gives each copy its own id), but it must not be
    /// the root id.
    fn check_unique_ids(&self, errors: &mut Vec<CompositionError>) {
        let mut seen = HashSet::new();
        seen.insert(self.id.as_str());
        for clip in &self.clips {
            if !seen.insert(clip.id.as_str()) {
                errors.push(CompositionError::DuplicateId {
                    id: clip.id.clone(),
                });
            }
        }
        for clip in &self.clips {
            if let AnyKind::Nested(n) = &clip.kind
                && n.composition_id.as_deref() == Some(self.id.as_str())
            {
                errors.push(CompositionError::DuplicateId {
                    id: self.id.clone(),
                });
            }
        }
    }

    /// S6: references exist, are not the clip itself and make no loop.
    fn check_references(&self, errors: &mut Vec<CompositionError>) {
        let index: HashMap<&str, usize> = self
            .clips
            .iter()
            .enumerate()
            .map(|(i, c)| (c.id.as_str(), i))
            .collect();
        // The clip each clip starts after, if that clip exists.
        let mut next: Vec<Option<usize>> = Vec::with_capacity(self.clips.len());
        for clip in &self.clips {
            let target = match &clip.start.0 {
                StartKind::At(_) => None,
                StartKind::After {
                    clip: reference, ..
                } if *reference == clip.id => {
                    errors.push(CompositionError::SelfReference {
                        clip: clip.id.clone(),
                    });
                    None
                }
                StartKind::After {
                    clip: reference, ..
                } => {
                    let found = index.get(reference.as_str()).copied();
                    if found.is_none() {
                        errors.push(CompositionError::UnknownReference {
                            clip: clip.id.clone(),
                            reference: reference.clone(),
                        });
                    }
                    found
                }
            };
            next.push(target);
        }
        let mut reported = vec![false; self.clips.len()];
        for first in 0..self.clips.len() {
            if reported[first] {
                continue;
            }
            // Each clip has at most one outgoing edge: walk the chain.
            let mut path = vec![first];
            let mut on_path = vec![false; self.clips.len()];
            on_path[first] = true;
            let mut at = first;
            while let Some(to) = next[at] {
                if to == at {
                    break;
                }
                if on_path[to] {
                    if to == first {
                        let mut ring = path.clone();
                        ring.sort_unstable();
                        for &i in &ring {
                            reported[i] = true;
                        }
                        errors.push(CompositionError::ReferenceCycle {
                            clips: ring.iter().map(|&i| self.clips[i].id.clone()).collect(),
                        });
                    }
                    break;
                }
                on_path[to] = true;
                path.push(to);
                at = to;
            }
        }
    }
}

fn check_id(id: &str, errors: &mut Vec<CompositionError>) {
    if !is_valid_id(id) {
        errors.push(CompositionError::InvalidId { id: id.to_owned() });
    }
}

fn check_url(owner: &str, url: &str, errors: &mut Vec<CompositionError>) {
    if !is_safe_url(url) {
        errors.push(CompositionError::UnsafeUrl {
            owner: owner.to_owned(),
            url: url.to_owned(),
        });
    }
}

fn check_media(id: &str, media: &MediaOptions, errors: &mut Vec<CompositionError>) {
    if let Some(volume) = media.volume
        && !VOLUME.contains(&volume)
    {
        errors.push(CompositionError::VolumeOutOfRange {
            clip: id.to_owned(),
            volume,
        });
    }
    check_rate(id, media.playback_rate, errors);
}

fn check_rate(id: &str, rate: Option<f64>, errors: &mut Vec<CompositionError>) {
    if let Some(rate) = rate
        && !RATE.contains(&rate)
    {
        errors.push(CompositionError::PlaybackRateOutOfRange {
            clip: id.to_owned(),
            rate,
        });
    }
}

fn check_binding(
    id: &str,
    binding: Option<&String>,
    declared: &HashSet<&str>,
    errors: &mut Vec<CompositionError>,
) {
    if let Some(variable) = binding
        && !declared.contains(variable.as_str())
    {
        errors.push(CompositionError::UnknownVariable {
            clip: id.to_owned(),
            variable: variable.clone(),
        });
    }
}

fn check_clip(clip: &AnyClip, declared: &HashSet<&str>, errors: &mut Vec<CompositionError>) {
    let id = clip.id.as_str();
    check_id(id, errors);
    match clip.duration {
        Some(d) if millis(d) == 0 => {
            errors.push(CompositionError::ZeroDuration { id: id.to_owned() });
        }
        None if matches!(clip.kind, AnyKind::Html(_) | AnyKind::Nested(_)) => {
            errors.push(CompositionError::ClipNeedsDuration {
                clip: id.to_owned(),
            });
        }
        _ => {}
    }
    match &clip.kind {
        AnyKind::Html(_) => {}
        AnyKind::Image(i) => {
            check_url(id, &i.src, errors);
            check_binding(id, i.bind_src.as_ref(), declared, errors);
        }
        AnyKind::Video(v) => {
            check_url(id, &v.src, errors);
            check_media(id, &v.media, errors);
            check_binding(id, v.bind_src.as_ref(), declared, errors);
        }
        AnyKind::Audio(a) => {
            check_url(id, &a.src, errors);
            check_media(id, &a.media, errors);
            check_binding(id, a.bind_src.as_ref(), declared, errors);
        }
        AnyKind::Nested(n) => {
            check_url(id, &n.src, errors);
            check_rate(id, n.playback_rate, errors);
            if let Some(composition_id) = &n.composition_id {
                check_id(composition_id, errors);
            }
            if let Some((w, h)) = n.size
                && !((1..=MAX_SIZE).contains(&w) && (1..=MAX_SIZE).contains(&h))
            {
                errors.push(CompositionError::InvalidSize {
                    owner: id.to_owned(),
                    width: w,
                    height: h,
                });
            }
            for (name, value) in &n.values {
                check_id(name, errors);
                if let VariableValue::Number(v) = value
                    && !v.is_finite()
                {
                    errors.push(CompositionError::InvalidValue {
                        clip: id.to_owned(),
                        name: name.clone(),
                        reason: "the value is not a finite number",
                    });
                }
            }
        }
    }
}

/// S11: resolves (start, end) in milliseconds for each clip, as the runtime does.
///
/// The references are valid and make no loop (S6), so each pass resolves at
/// least one more clip until no clip can change.
fn resolve_timing(clips: &[AnyClip]) -> Vec<(Option<u64>, Option<u64>)> {
    let index: HashMap<&str, usize> = clips
        .iter()
        .enumerate()
        .map(|(i, c)| (c.id.as_str(), i))
        .collect();
    let length = |clip: &AnyClip| match (clip.duration, &clip.kind) {
        (Some(d), _) => Some(millis(d)),
        (None, AnyKind::Image(_)) => Some(IMAGE_DEFAULT_MS),
        (None, _) => None,
    };
    let mut timing: Vec<(Option<u64>, Option<u64>)> = vec![(None, None); clips.len()];
    let mut done = vec![false; clips.len()];
    loop {
        let mut changed = false;
        for (i, clip) in clips.iter().enumerate() {
            if done[i] {
                continue;
            }
            let start = match &clip.start.0 {
                StartKind::At(at) => Some(Some(millis(*at))),
                StartKind::After { clip: r, offset } => {
                    let r = index[r.as_str()];
                    done[r].then(|| {
                        timing[r].1.map(|end| {
                            let at = i128::from(end) + offset.signed_millis();
                            u64::try_from(at.max(0)).unwrap_or(u64::MAX)
                        })
                    })
                }
            };
            if let Some(start) = start {
                let end = start.zip(length(clip)).map(|(s, l)| s.saturating_add(l));
                timing[i] = (start, end);
                done[i] = true;
                changed = true;
            }
        }
        if !changed {
            return timing;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn class_list_joins_and_skips_empty() {
        assert_eq!(class_list(&[], false), None);
        assert_eq!(class_list(&[], true).as_deref(), Some("clip"));
        assert_eq!(
            class_list(&["a".into(), "b".into()], true).as_deref(),
            Some("clip a b")
        );
    }
}
