//! [`PlayerControl`]: a button that drives a player through `init.js`.

use std::time::Duration;

use autumn_web::{Markup, html};

use crate::time::seconds;

/// What a [`PlayerControl`] button does.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
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
        match self {
            Self::Play => "play",
            Self::Pause => "pause",
            Self::Toggle => "toggle",
            Self::Restart => "restart",
            Self::Mute => "mute",
            Self::Unmute => "unmute",
            Self::ToggleMute => "toggle-mute",
            Self::Seek(_) => "seek",
        }
    }

    /// The default button text.
    fn default_label(self) -> String {
        match self {
            Self::Play => "Play".to_owned(),
            Self::Pause => "Pause".to_owned(),
            Self::Toggle => "Play or pause".to_owned(),
            Self::Restart => "Restart".to_owned(),
            Self::Mute => "Mute".to_owned(),
            Self::Unmute => "Unmute".to_owned(),
            Self::ToggleMute => "Mute or unmute".to_owned(),
            Self::Seek(time) => format!("Go to {} s", seconds(time)),
        }
    }
}

/// A `<button>` that controls a player.
///
/// `init.js` handles the click. The button is a native `<button>`, so it works
/// with the keyboard.
///
/// ```rust
/// use std::time::Duration;
/// use autumn_plugin_hyperframes::{Control, PlayerControl};
///
/// let html = PlayerControl::new("intro", Control::Seek(Duration::from_millis(2500)))
///     .label("Go to the logo")
///     .render()
///     .into_string();
/// assert!(html.contains(r#"data-hf-seek="2.5""#));
/// ```
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
        let class = (!self.classes.is_empty()).then(|| self.classes.join(" "));
        let seek = match self.control {
            Control::Seek(time) => Some(seconds(time)),
            _ => None,
        };
        let label = self
            .label
            .clone()
            .unwrap_or_else(|| self.control.default_label());
        html! {
            button type="button" class=[class]
                data-hf-control=(self.control.name())
                data-hf-target=(self.target) aria-controls=(self.target)
                data-hf-seek=[seek] {
                (label)
            }
        }
    }
}

impl maud::Render for PlayerControl {
    fn render(&self) -> Markup {
        Self::render(self)
    }
}
