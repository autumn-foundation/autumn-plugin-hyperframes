# autumn-plugin-hyperframes

[HyperFrames](https://hyperframes.heygen.com) video compositions for Autumn apps.
Write a composition in typed Rust. Play it in the page with `<hyperframes-player>`.
No npm. No bundler. No CDN. It works with the default Autumn CSP.

The crate vendors `@hyperframes/player` and the `@hyperframes/core` runtime
(0.8.140, Apache-2.0) and serves them from memory.

## Quickstart

Add the plugin:

```rust,no_run
use autumn_plugin_hyperframes::HyperframesPlugin;

#[autumn_web::main]
async fn main() {
    autumn_web::app()
        .plugin(HyperframesPlugin::new())
        .run()
        .await;
}
```

Put the tags in your layout:

```rust
use autumn_plugin_hyperframes::{hyperframes_script, hyperframes_stylesheet};
use autumn_web::{Markup, html};

fn layout(content: &Markup) -> Markup {
    html! {
        head {
            (hyperframes_stylesheet())
            (hyperframes_script())
        }
        body { (content) }
    }
}
```

Write a composition and play it:

```rust
use std::time::Duration;
use autumn_plugin_hyperframes::{BuildError, Clip, Composition, Player, Start};
use autumn_web::{Markup, html};

fn intro_player() -> Result<Markup, BuildError> {
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
    Ok(html! { (Player::composition(&intro).id("intro").controls()) })
}

assert!(intro_player().is_ok());
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
| `Clip::nested(id, src, &composition)` | `<div data-composition-src>` | A nested composition. Copies its id, size, duration and timeline flag. |
| `Clip::composition(id, src)` | `<div data-composition-src>` | A nested composition by URL only. Needs a duration. |

All clips: `.start(Start)`, `.duration(Duration)`, `.track(n)`, `.class(..)`.
Video and audio: `.media_start(..)`, `.volume(..)`, `.fade_in(..)`, `.fade_out(..)`,
`.playback_rate(..)`. Image, video and audio: `.bind_src(variable)`.
Nested: `.composition_id(..)`, `.playback_start(..)`, `.playback_rate(..)`,
`.size(w, h)`, `.no_timeline()`, `.value(name, value)`.

`Start::at(t)` is an absolute time. `Start::after("id")` starts when that clip
ends. `.plus(t)` and `.minus(t)` move it. Times round to milliseconds.

## Build rules

`build()` returns a `BuildError` with every `CompositionError` that it finds:

- Ids must match `[A-Za-z][A-Za-z0-9_-]*`, max 128 characters.
- The root id and the clip ids must be unique.
- Width and height must be in `1..=16384`.
- A composition without `.with_timeline()` must have a duration.
  A duration must round to at least 1 ms.
- HTML and nested clips must have a duration.
- `Start::after` must name another clip in the same composition. Loops are errors.
- Volume must be in `0..=3.98`. Clip playback rate must be in `0.1..=10`.
- URLs must be relative or `http(s)`. Image, video and audio URLs can also be
  `blob:` or a `data:` image (not SVG), video or audio.
- Variables must have unique ids, finite numbers and valid choice defaults.
- `bind_src` must name a declared variable.
- A nested composition with relative starts can play only one time.
  The runtime finds a clip by id in the whole page, so a second copy would
  follow the clips of the first copy.
- HTML content must not have an element id that is also a clip id.

`Composition::resolved_start(id)` and `resolved_end(id)` return the start and
end of a clip. They use the runtime rule. Use them in tests.

## Variables and nested compositions

```rust
use std::time::Duration;
use autumn_plugin_hyperframes::{BuildError, Clip, Composition, Start, Variable};
use autumn_web::html;

fn plans() -> Result<(Composition, Composition), BuildError> {
    let card = Composition::builder("card")
        .duration(Duration::from_secs(3))
        .variable(Variable::string("plan", "Free").label("Plan name"))
        .variable(Variable::color("accent", "#f0a35e"))
        .clip(Clip::html("plan", html! { h1 data-var-text="plan" { "Free" } })
            .duration(Duration::from_secs(3)))
        .build()?;
    // Serve card.template() at /compositions/card.html.
    let plans = Composition::builder("plans")
        .duration(Duration::from_secs(6))
        .clip(Clip::nested("pro", "/compositions/card.html", &card).value("plan", "Pro"))
        .clip(Clip::nested("team", "/compositions/card.html", &card)
            .start(Start::after("pro"))
            .value("plan", "Team"))
        .build()?;
    Ok((card, plans))
}

assert!(plans().is_ok());
```

Scalar variables are CSS custom properties in the composition, for example
`color: var(--accent)`. `data-var-text` and `.bind_src(..)` replace text and sources.

## Outputs

| Method | Use |
|---|---|
| `Player::composition(&c)` | Embed in a page (`srcdoc`). Works with the default Autumn headers. |
| `c.document()` | A full page with the runtime. Serve it, then use `Player::src(url)`. The HyperFrames CLI can render it. |
| `c.template()` | The file for `Clip::nested`: a page with one `<template>`. `<html>` has the variables. |
| `c.fragment()` | The root element only. |
| `c.srcdoc_html()` | The page for a player `srcdoc`. |

## Player

```rust
use autumn_plugin_hyperframes::{Composition, Player};
use autumn_web::Markup;

fn player(intro: &Composition) -> Markup {
    Player::composition(intro)
        .id("intro")
        .label("Product intro")
        .controls()
        .muted()
        .autoplay()
        .looped()
        .in_view()
        .render()
}
```

| Method | Attribute |
|---|---|
| `id`, `class`, `label` | `id`, `class`, `role="group"` + `aria-label` |
| `size(w, h)` | `width`, `height` (from the composition in `srcdoc` mode) |
| `controls`, `muted`, `looped` | `controls`, `muted`, `loop` |
| `autoplay` | `data-hf-autoplay`: `init.js` adds `autoplay` when motion is allowed |
| `poster(url)` | `poster` |
| `playback_rate(r)` | `playback-rate` (clamped to `0.1..=5`) |
| `volume(v)` | `volume` (clamped to `0..=1`) |
| `range(start, end)` | `range-start`, `range-end` |
| `audio_locked`, `low_power_idle`, `disable_click_to_play` | `audio-locked`, `low-power-idle`, `disable-click-to-play` |
| `opaque_sandbox` | `sandbox-origin="opaque"` |
| `hide_loading_ui` | `assets-loading-ui="none"` |
| `shader_loading(..)` | `shader-loading` |
| `in_view` | `data-hf-in-view`: play when visible, pause when not |
| `reduced_motion(ReducedMotion::Animate)` | `data-hf-reduced="animate"` |

Every player has `runtime-src` set to the vendored runtime. An option that you
do not set writes no attribute. A URL that is not on the allowlist writes no
attribute. A page `src` must be relative or `http(s)`.
`Player::video(url, VideoType::Mp4)` plays a rendered video file.
`hyperframes.css` sets the aspect ratio. The default is 16:9. The sizes 9:16,
1:1, 4:5, 4:3 and 21:9 get their own class.

## Controls

```rust
use std::time::Duration;
use autumn_plugin_hyperframes::{Control, PlayerControl};
use autumn_web::html;

let buttons = html! {
    (PlayerControl::new("intro", Control::Play))
    (PlayerControl::new("intro", Control::Seek(Duration::from_millis(2500))).label("Go to the logo"))
};
assert!(buttons.into_string().contains(r#"data-hf-control="seek""#));
```

Controls: `Play`, `Pause`, `Toggle`, `Restart`, `Mute`, `Unmute`, `ToggleMute`,
`Seek(t)`. `init.js` handles the clicks, also for content that htmx adds.

`init.js` sets `data-hf-state` (`playing`, `paused`, `ended`) on each player for CSS.
Player events do not bubble. For htmx, name the player:
`hx-trigger="ended from:#intro"`.

## Reduced motion

With `prefers-reduced-motion: reduce`, `init.js` does not autoplay and does not
play in-view players. It sets `data-hf-autoplay-blocked`. The user can still
press play. `.reduced_motion(ReducedMotion::Animate)` opts a player back in.
`.autoplay()` writes `data-hf-autoplay`, not `autoplay`, so the player cannot
start before `init.js` reads the user setting.

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
  The `srcdoc` tags then have no SRI and no CORS attributes, because an opaque
  frame cannot pass those checks. In opaque mode, `Player::src` pages and nested
  compositions need CORS for the null origin (`cors.allowed_origins`).
- Each composition always carries the runtime tag, so text in a composition
  cannot stop the player from loading it.
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
