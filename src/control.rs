//! [`PlayerControl`]: a button that drives a player through `init.js`.

use std::time::Duration;

use autumn_web::Markup;

/// What a [`PlayerControl`] button does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Control {
    /// Play.
    Play,
    /// Pause.
    Pause,
    /// Play when paused, else pause.
    Toggle,
    /// Go to the start and play.
    Restart,
    /// Mute.
    Mute,
    /// Unmute.
    Unmute,
    /// Mute when unmuted, else unmute.
    ToggleMute,
    /// Go to a time.
    Seek(Duration),
}

impl Control {
    /// The `data-hf-control` value.
    #[must_use]
    pub const fn name(self) -> &'static str {
        ""
    }
}

/// A `<button>` that controls a player.
#[derive(Debug, Clone)]
#[must_use]
pub struct PlayerControl {
    target: String,
    control: Control,
    label: Option<String>,
    classes: Vec<String>,
}

impl PlayerControl {
    /// A button for the player with the id `target`.
    pub fn new(target: &str, control: Control) -> Self {
        Self {
            target: target.to_owned(),
            control,
            label: None,
            classes: Vec::new(),
        }
    }

    /// Sets the button text.
    pub fn label(mut self, label: &str) -> Self {
        self.label = Some(label.to_owned());
        self
    }

    /// Adds a CSS class.
    pub fn class(mut self, class: &str) -> Self {
        self.classes.push(class.to_owned());
        self
    }

    /// Renders the button.
    #[must_use]
    pub fn render(&self) -> Markup {
        Markup::default()
    }
}

impl maud::Render for PlayerControl {
    fn render(&self) -> Markup {
        Self::render(self)
    }
}
