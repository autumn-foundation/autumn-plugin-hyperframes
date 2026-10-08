//! HyperFrames video compositions and player for Autumn.

mod assets;
mod plugin;
mod script;

pub use assets::{
    ASSETS_NAMESPACE, HYPERFRAMES_ASSETS, HYPERFRAMES_VERSION, PLAYER_JS_INTEGRITY, PLAYER_SOURCE,
    RUNTIME_JS_INTEGRITY, RUNTIME_SOURCE,
};
pub use plugin::{HyperframesPlugin, PLUGIN_NAME};
pub use script::{hyperframes_script, hyperframes_stylesheet};
