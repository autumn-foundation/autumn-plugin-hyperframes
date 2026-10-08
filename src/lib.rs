//! HyperFrames video compositions and player for Autumn.

mod assets;
mod clip;
mod composition;
mod error;
mod id;
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
pub use error::{BuildError, CompositionError};
pub use id::{Id, MAX_ID_LEN};
pub use plugin::{HyperframesPlugin, PLUGIN_NAME};
pub use script::{hyperframes_script, hyperframes_stylesheet};
pub use time::{Offset, Start};
pub use variable::{Variable, VariableValue};
