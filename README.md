# autumn-plugin-hyperframes

[HyperFrames](https://hyperframes.heygen.com) video compositions for Autumn apps.
Write a composition in typed Rust. Play it in the page with `<hyperframes-player>`.
No npm. No bundler. No CDN. It works with the default Autumn CSP.

The crate vendors `@hyperframes/player` and the `@hyperframes/core` runtime
(0.8.140, Apache-2.0) and serves them from memory.

## Quickstart

Add the plugin:

```rust
use autumn_plugin_hyperframes::HyperframesPlugin;

autumn_web::app()
    .plugin(HyperframesPlugin::new())
    .run()
    .await;
```

Put the tags in your layout:

```rust
use autumn_plugin_hyperframes::{hyperframes_script, hyperframes_stylesheet};

html! {
    head {
        (hyperframes_stylesheet())
        (hyperframes_script())
    }
}
```

Write a composition and play it:

```rust
use std::time::Duration;
use autumn_plugin_hyperframes::{Clip, Composition, Player, Start};

let intro = Composition::builder("intro")
    .size(1280, 720)
    .duration(Duration::from_secs(4))
    .stylesheet("/static/css/intro.css")
    .clip(Clip::html("title", html! { h1 { "Write Rust." } })
        .duration(Duration::from_secs(2))
        .class("rise"))
    .clip(Clip::html("tagline", html! { h1 { "Get video." } })
        .start(Start::after("title"))
        .duration(Duration::from_secs(2)))
    .build()?;

html! { (Player::composition(&intro).id("intro").controls()) }
```

`build()` checks the composition and returns all errors. Only a checked
`Composition` can render.

## Animate

The HyperFrames runtime seeks CSS animations. Put them in a stylesheet and add
it with `.stylesheet(..)`. A clip animation starts at the clip start.

```css
.rise h1 { animation: rise 0.8s ease-out both; }
@keyframes rise { from { opacity: 0; transform: translateY(60px); } }
```

For GSAP or other runtimes, add the script file with `.script(..)` and call
`.with_timeline()`. The builder has no inline scripts, because the Autumn CSP
blocks them.

## Clips

| Builder | HTML | Notes |
|---|---|---|
| `Clip::html(id, markup)` | `<div class="clip">` | Needs a duration. |
| `Clip::image(id, src)` | `<img class="clip">` | 3 s without a duration. `.alt(..)`. |
| `Clip::video(id, src, VideoAudio::Muted)` | `<video muted playsinline>` | `VideoAudio::HasAudio` writes `data-has-audio="true"`. |
| `Clip::audio(id, src)` | `<audio>` | Plays to the source end without a duration. |
| `Clip::composition(id, src)` | `<div data-composition-src>` | A nested composition. Needs a duration. |

All clips: `.start(Start)`, `.duration(Duration)`, `.track(n)`, `.class(..)`.
Video and audio: `.media_start(..)`, `.volume(..)`, `.fade_in(..)`, `.fade_out(..)`,
`.playback_rate(..)`. Image, video and audio: `.bind_src(variable)`.
Nested: `.composition_id(..)`, `.playback_start(..)`, `.playback_rate(..)`,
`.size(w, h)`, `.value(name, value)`.

`Start::at(t)` is an absolute time. `Start::after("id")` starts when that clip
ends. `.plus(t)` and `.minus(t)` move it. Times round to milliseconds.

## Build rules

`build()` returns a `BuildError` with every `CompositionError` it finds:

- Ids match `[A-Za-z][A-Za-z0-9_-]*`, max 128 characters.
- The root id and the clip ids are unique.
- Width and height are in `1..=16384`.
- A composition without `.with_timeline()` has a duration. Durations are at least 1 ms.
- HTML and nested clips have a duration.
- `Start::after` names another clip in the same composition. No loops.
- Volume is in `0..=3.98`. Clip playback rate is in `0.1..=10`.
- URLs are not empty and do not use `javascript:`, `vbscript:` or `data:text/html`.
- Variables have unique ids, finite numbers and valid enum defaults.
- `bind_src` names a declared variable.

`Composition::resolved_start(id)` and `resolved_end(id)` give the timeline
times, with the runtime rule. Use them in tests.

## Variables and nested compositions

```rust
let card = Composition::builder("card")
    .duration(Duration::from_secs(3))
    .variable(Variable::string("plan", "Free").label("Plan name"))
    .clip(Clip::html("plan", html! { h1 data-var-text="plan" { "Free" } })
        .duration(Duration::from_secs(3)))
    .build()?;
// Serve card.template() at /compositions/card.html.

let plans = Composition::builder("plans")
    .duration(Duration::from_secs(6))
    .clip(Clip::composition("pro", "/compositions/card.html")
        .composition_id("card").duration(Duration::from_secs(3)).value("plan", "Pro"))
    .clip(Clip::composition("team", "/compositions/card.html")
        .composition_id("card").start(Start::after("pro"))
        .duration(Duration::from_secs(3)).value("plan", "Team"))
    .build()?;
```

## Outputs

| Method | Use |
|---|---|
| `Player::composition(&c)` | Embed in a page (`srcdoc`). Works with the default Autumn headers. |
| `c.document()` | A full page with the runtime. Serve it, then use `Player::src(url)`. The HyperFrames CLI can render it. |
| `c.template()` | A `<template>` file for `Clip::composition`. |
| `c.fragment()` | The root element only. |

## Player

```rust
Player::composition(&intro)
    .id("intro")
    .label("Product intro")
    .controls()
    .muted()
    .autoplay()
    .looped()
    .in_view()
```

| Method | Attribute |
|---|---|
| `id`, `class`, `label` | `id`, `class`, `role="group"` + `aria-label` |
| `size(w, h)` | `width`, `height` (from the composition in `srcdoc` mode) |
| `controls`, `muted`, `autoplay`, `looped` | `controls`, `muted`, `autoplay`, `loop` |
| `poster(url)` | `poster` |
| `playback_rate(r)` | `playback-rate` (clamped to `0.1..=5`) |
| `volume(v)` | `volume` (clamped to `0..=1`) |
| `range(start, end)` | `range-start`, `range-end` |
| `audio_locked`, `low_power_idle`, `disable_click_to_play` | same names |
| `opaque_sandbox` | `sandbox-origin="opaque"` |
| `hide_loading_ui` | `assets-loading-ui="none"` |
| `shader_loading(..)` | `shader-loading` |
| `in_view` | `data-hf-in-view`: play when visible, pause when not |
| `reduced_motion(ReducedMotion::Animate)` | `data-hf-reduced="animate"` |

Every player has `runtime-src` set to the vendored runtime. An option that you
do not set writes no attribute. An unsafe URL writes no attribute.
`Player::video(url, VideoType::Mp4)` plays a rendered video file.
The player aspect ratio comes from `hyperframes.css`: 16:9, or 9:16, 1:1, 4:5,
4:3 and 21:9 from the size.

## Controls

```rust
(PlayerControl::new("intro", Control::Play))
(PlayerControl::new("intro", Control::Seek(Duration::from_millis(2500))).label("Go to the logo"))
```

Controls: `Play`, `Pause`, `Toggle`, `Restart`, `Mute`, `Unmute`, `ToggleMute`,
`Seek(t)`. `init.js` handles the clicks, also for content that htmx adds.

`init.js` sets `data-hf-state` (`playing`, `paused`, `ended`) on each player for CSS.
Player events do not bubble. For htmx, name the player:
`hx-trigger="ended from:#intro"`.

## Reduced motion

With `prefers-reduced-motion: reduce`, `init.js` removes `autoplay` and does not
play in-view players. It sets `data-hf-autoplay-blocked`. The user can still
press play. `.reduced_motion(ReducedMotion::Animate)` opts a player back in.

## Play a composition from a URL

`Player::src(url)` loads a page in the player iframe. The Autumn defaults
(`X-Frame-Options: DENY`, `frame-ancestors 'none'`) block that frame. To allow
same-origin frames, set this in `autumn.toml`:

```toml
[security.headers]
x_frame_options = "SAMEORIGIN"
content_security_policy = "default-src 'self'; img-src 'self' data:; style-src 'self' 'unsafe-inline'; script-src 'self'; connect-src 'self'; form-action 'self'; frame-ancestors 'self'; base-uri 'self'"
```

`Player::composition` does not need this.

## Security

- The player sandbox has `allow-scripts` and `allow-same-origin` by default.
  Same-origin composition code can reach the page. Chromium logs a warning for this.
  Use `.opaque_sandbox()` for a composition that you do not trust fully.
- A `srcdoc` composition gets the CSP of the page. All scripts must be files on
  your origin.
- The runtime adds `<style>` elements. Use `style-src 'unsafe-inline'` (the Autumn
  default) for compositions. With a stricter `style-src` (for example `csp_nonce`),
  timing and sizing still work, but the browser blocks those runtime styles and
  logs a CSP error.

## Demo

```sh
cargo run --example hyperframes_demo
# open http://127.0.0.1:3000
```

## How it works

- `assets/hyperframes-player.global.js`: upstream `@hyperframes/player` IIFE build.
- `assets/hyperframe.runtime.iife.js`: upstream `@hyperframes/core` runtime IIFE build.
- `assets/init.js`: the plugin scanner (controls, in-view, reduced motion, htmx).
- `assets/hyperframes.css`: player size. `assets/composition.css`: root box and clips.
- `assets/manifest.json`: provenance (versions, URLs, `sha384`). Not served.
- `HYPERFRAMES_ASSETS` holds the five served files. `HyperframesPlugin` installs it
  through the Autumn `plugin_assets` seam: hashed, immutable URLs under
  `/static/_plugins/hyperframes/`, ETag/304, Range and computed SRI.
- The host page loads the player and `init.js`. Only compositions load the runtime.
  In `srcdoc` mode, the player puts the runtime from `runtime-src` into the page.

## Limits

- The plugin does not render MP4 files. Rendering needs Node, Chrome and FFmpeg.
  Serve `document()` and use the HyperFrames CLI (`npx hyperframes render`).
- The HyperFrames version is pinned per plugin release. HyperFrames is 0.x; its API can change.
- Not in scope: the SDK (editing), Studio, the slideshow element,
  `setRuntimeData`, shader transitions, audio effect chains and color grading.
  Write those attributes by hand in clip content.
- Developer tools ask for `hyperframes-player.global.js.map`. The plugin does not
  ship source maps, so that request gets 404.

## License

Apache-2.0. The vendored HyperFrames files are Apache-2.0 too
(`LICENSES/HyperFrames-Apache-2.0.txt`).
