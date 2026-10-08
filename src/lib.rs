//! [HyperFrames](https://hyperframes.heygen.com) video compositions for Autumn.
//!
//! Write a composition in typed Rust. Play it in the page with
//! `<hyperframes-player>`. No npm, no bundler, no CDN. It works with the
//! default Autumn CSP.
//!
//! ```rust,no_run
//! use std::time::Duration;
//! use autumn_plugin_hyperframes::{
//!     Clip, Composition, Control, HyperframesPlugin, Player, PlayerControl, Start,
//!     hyperframes_script, hyperframes_stylesheet,
//! };
//! use autumn_web::prelude::*;
//!
//! #[get("/")]
//! async fn index() -> Markup {
//!     let intro = Composition::builder("intro")
//!         .duration(Duration::from_secs(4))
//!         .clip(Clip::html("title", html! { h1 { "Hello" } }).duration(Duration::from_secs(2)))
//!         .clip(Clip::html("tagline", html! { p { "World" } })
//!             .start(Start::after("title"))
//!             .duration(Duration::from_secs(2)))
//!         .build()
//!         .expect("valid composition");
//!     html! {
//!         head { (hyperframes_stylesheet()) (hyperframes_script()) }
//!         body {
//!             (Player::composition(&intro).id("intro").controls())
//!             (PlayerControl::new("intro", Control::Restart))
//!         }
//!     }
//! }
//!
//! # async fn run() {
//! autumn_web::app()
//!     .plugin(HyperframesPlugin::new())
//!     .routes(routes![index])
//!     .run()
//!     .await;
//! # }
//! ```
//!
//! # Parts
//!
//! - [`HyperframesPlugin`] serves the vendored player and runtime
//!   ([`HYPERFRAMES_ASSETS`]) at hashed URLs with SRI.
//! - [`Composition::builder`] makes a [`CompositionBuilder`]. Its `build()`
//!   checks the rules and returns a [`Composition`] or a [`BuildError`].
//! - [`Clip`] kinds: HTML, image, video, audio and nested compositions.
//!   [`Start`] places a clip at a time or after another clip.
//! - [`Player`] renders `<hyperframes-player>`. It always sets `runtime-src`,
//!   so the player never loads the runtime from a CDN.
//! - [`PlayerControl`] renders buttons that `init.js` connects to a player.
//!
//! # Limits
//!
//! - The crate does not render MP4 files. Serve [`Composition::document`] and
//!   use the HyperFrames CLI.
//! - The HyperFrames version is pinned per release ([`HYPERFRAMES_VERSION`]).
//! - The builder has no inline scripts. Use stylesheets (CSS animations) or
//!   script files.

mod assets;
mod clip;
mod composition;
mod control;
mod error;
mod id;
mod player;
mod plugin;
mod script;
mod time;
mod url;
mod variable;

pub use assets::{
    ASSETS_NAMESPACE, HYPERFRAMES_ASSETS, HYPERFRAMES_VERSION, PLAYER_JS_INTEGRITY, PLAYER_SOURCE,
    RUNTIME_JS_INTEGRITY, RUNTIME_SOURCE,
};
pub use clip::{Audio, Clip, ClipKind, Html, Image, Nested, Video, VideoAudio};
pub use composition::{Composition, CompositionBuilder, DEFAULT_HEIGHT, DEFAULT_WIDTH, MAX_SIZE};
pub use control::{Control, PlayerControl};
pub use error::{BuildError, CompositionError};
pub use id::{Id, MAX_ID_LEN};
pub use player::{Player, ReducedMotion, ShaderLoading, VideoType};
pub use plugin::{HyperframesPlugin, PLUGIN_NAME};
pub use script::{hyperframes_script, hyperframes_stylesheet};
pub use time::Start;
pub use variable::{Variable, VariableValue};

/// The README examples compile and run as doc tests.
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
pub struct ReadmeDoctests;
