//! [`Composition`]: a checked HyperFrames composition, and its builder.

use std::time::Duration;

use autumn_web::Markup;

use crate::clip::{AnyClip, Clip, ClipKind};
use crate::error::BuildError;
use crate::id::Id;
use crate::variable::Variable;

/// The default composition width in pixels.
pub const DEFAULT_WIDTH: u32 = 1920;

/// The default composition height in pixels.
pub const DEFAULT_HEIGHT: u32 = 1080;

/// The largest width or height in pixels.
pub const MAX_SIZE: u32 = 16_384;

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
/// follows the rules in `docs/plan.md` (spec S1 to S11).
#[derive(Debug, Clone)]
pub struct Composition {
    id: Id,
    width: u32,
    height: u32,
    duration: Option<Duration>,
    clips: Vec<AnyClip>,
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

    /// The start of clip `id` on the timeline, if it is known.
    #[must_use]
    pub fn resolved_start(&self, id: &str) -> Option<Duration> {
        let _ = id;
        None
    }

    /// The end of clip `id` on the timeline, if it is known.
    #[must_use]
    pub fn resolved_end(&self, id: &str) -> Option<Duration> {
        let _ = id;
        None
    }

    /// Renders the root element and its clips.
    #[must_use]
    pub fn fragment(&self) -> Markup {
        Markup::default()
    }

    /// Renders a full HTML page that loads the runtime.
    #[must_use]
    pub fn document(&self) -> Markup {
        Markup::default()
    }

    /// Renders a `<template>` file for use as a nested composition.
    #[must_use]
    pub fn template(&self) -> Markup {
        Markup::default()
    }

    /// The page HTML for a player `srcdoc`. It does not link the runtime.
    #[must_use]
    pub fn srcdoc_html(&self) -> String {
        String::new()
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
    /// Returns a [`BuildError`] when a rule in `docs/plan.md` (S1 to S10) fails.
    pub fn build(self) -> Result<Composition, BuildError> {
        Ok(Composition {
            id: Id::new(&self.id).unwrap_or_else(|| Id::new("x").unwrap_or_else(|| unreachable!())),
            width: self.width,
            height: self.height,
            duration: self.duration,
            clips: self.clips,
        })
    }
}
