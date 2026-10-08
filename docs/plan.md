# Plan: autumn-plugin-hyperframes 0.1.0

Status: done. Target: autumn-web 0.8.0, HyperFrames 0.8.140.
The repository has no issue. Section 6 holds the acceptance criteria (AC).
The AC evidence table is in the PR description.
Prior art: [autumn-plugin-motion](https://github.com/madmax983/autumn-plugin-motion)
and [autumn-plugin-gsap](https://github.com/autumn-foundation/autumn-plugin-gsap).

## 1. Goal

Let Autumn apps write [HyperFrames](https://hyperframes.heygen.com) compositions
in typed Rust and play them in the page. No npm. No bundler. No CDN.
The default Autumn CSP must not block anything.

A HyperFrames composition is an HTML document with timed clips
(`data-start`, `data-duration`). The HyperFrames runtime shows each clip in its
time window and seeks CSS, WAAPI and GSAP animations. `<hyperframes-player>`
plays a composition in a sandboxed iframe, with video-like controls.

## 2. Brainstorm (what can the plugin do?)

- Vendor `@hyperframes/player` (IIFE build) and the `@hyperframes/core`
  runtime (IIFE build). Both are Apache-2.0.
- Serve them through the Autumn 0.8 `PluginAssets` seam: hashed URLs, SRI,
  ETag, immutable cache.
- Typed `Composition` builder: id, size, duration, clips, stylesheets,
  scripts, variables.
- Typed `Clip` kinds: HTML block, image, video, audio, nested composition.
- Typed `Start`: absolute time, or "after clip X, plus or minus Y".
- Use `std::time::Duration` for times. Video audio is an enum
  (`Muted` or `HasAudio`), so the "video_missing_muted" lint cannot fail.
- `build()` checks the whole composition (ids, references, cycles, ranges)
  and returns all errors. Only a checked `Composition` can render
  (type state: no unchecked output).
- Three outputs from one composition: `document()` (a page),
  `template()` (a nested file), `fragment()` (the root element).
- Typed `Player` builder: composition (`srcdoc`), URL (`src`) or video file.
  All player attributes: controls, muted, autoplay, loop, poster, rate,
  range, audio lock, opaque sandbox, low power idle, and more.
- Point the player at the vendored runtime (`runtime-src`), so the player
  never loads the runtime from jsDelivr.
- `init.js`: declarative control buttons (`data-hf-control`), play when in
  view (`data-hf-in-view`), reduced motion, `data-hf-state` for CSS,
  htmx support (`htmx:load`, `htmx:beforeCleanupElement`).
- Typed `PlayerControl` button builder.
- Host stylesheet (player aspect ratio classes) and composition stylesheet
  (root box, full-frame `.clip`).
- Variables: typed declarations, `bind_src`, values on nested hosts.
- Browser tests with Playwright against a demo app under the default CSP.

## 3. Reverse brainstorm (how can the plugin fail?)

| Way to fail | Prevention |
|---|---|
| The player loads the runtime from `cdn.jsdelivr.net`. The CSP blocks it. | Vendor the runtime. Every `Player` sets `runtime-src` to the vendored URL. `document()` links the runtime. The e2e test fails on a CSP error or a request to another origin. |
| Autumn sends `X-Frame-Options: DENY`. A composition URL cannot load in the player iframe. | Default to `srcdoc` mode: no navigation, so no frame headers. Document the config for `src` mode. The e2e test covers both modes. |
| The `srcdoc` gets the runtime two times. | `srcdoc` output does not link the runtime. The player inserts it from `runtime-src`. |
| Inline `<script>` in a composition breaks the CSP (a `srcdoc` frame gets the page CSP). | The builder accepts only script URLs. Animate with CSS in a stylesheet. The runtime seeks CSS animations. |
| A typo in a clip id breaks a relative start. Nothing shows. | `build()` checks that each reference exists, is not the clip itself and makes no cycle. |
| An id looks like a number (`"12"`). The runtime reads it as an absolute time. | The id grammar is `[A-Za-z][A-Za-z0-9_-]*`, max 128 characters. |
| A video has neither `muted` nor `data-has-audio`. | `VideoAudio` is a required enum. |
| A volume or rate is out of range. The runtime clamps it in silence. | `build()` rejects volume outside `0..=3.98` and rate outside `0.1..=10`. |
| A `javascript:` URL in `src`. | `build()` and `Player` reject `javascript:`, `vbscript:` and `data:text/html` URLs. |
| A timeline-free composition has no duration. The render waits 45 s. | `build()` requires a duration unless `with_timeline()` is set. Timeline-free output has `data-no-timeline`. |
| Variable JSON breaks the attribute or is not valid JSON. | Use `serde_json`. Maud escapes the attribute. |
| Users with reduced motion get autoplay. | `init.js` removes `autoplay` and skips in-view play, unless `data-hf-reduced="animate"`. |
| htmx removes a player. The observer keeps it. | Unobserve on `htmx:beforeCleanupElement`. |
| A control button targets a missing player. A click throws. | `init.js` ignores a missing target or a bad value. |
| The vendored bytes change by accident. | Pin the `sha384` of each upstream file. A test checks it. CI compares with the npm tarball. |
| The plugin forces features on the host app. | `PluginAssets::from_files`. No `embed-assets`. A CI step checks it. |

Found in the multi-angle review, then fixed with a test first:

| Way to fail | Prevention |
|---|---|
| A DOM marker (`data-hf-init`) goes into the htmx history snapshot. After Back, no player is set up. | `init.js` keeps set-up players in a `WeakSet`. |
| With reduced motion, the player reads `autoplay` before `htmx:load` runs. | `.autoplay()` writes `data-hf-autoplay`. `init.js` adds `autoplay` only when motion is allowed. |
| Text such as `__hyperframes =` makes the player skip the runtime. | Each `srcdoc` links the runtime by its plain URL. |
| An opaque frame cannot pass CORS or SRI checks in production. | Opaque `srcdoc` tags have no `integrity` and no `crossorigin`. |
| The runtime reads nested variable defaults only from `<html>`. | `template()` is a page; `<html>` has the declarations. |
| A nested host with another id or no `data-no-timeline` makes a CLI render wait. | `Clip::nested` copies the id, size, duration and timeline flag. |
| `data:image/svg+xml` or `data: text/html` passes a block list. | URLs use an allowlist for each use (S8). |

## 4. Six thinking hats

- **White (facts):** `@hyperframes/player` and `@hyperframes/core` 0.8.140 are on npm,
  Apache-2.0. The player IIFE is 103 KB. The runtime IIFE is 507 KB.
  The default Autumn CSP is `script-src 'self'`, `style-src 'self' 'unsafe-inline'`,
  `frame-ancestors 'none'`, plus `X-Frame-Options: DENY`.
  The player uses `adoptedStyleSheets`, so its own styles do not need `unsafe-inline`.
  The runtime adds `<style>` elements, so compositions want `style-src 'unsafe-inline'`
  (the Autumn default). The runtime sizes the root through CSSOM, so the root needs no
  `style` attribute.
- **Red (feelings):** Users want "write Rust, see video". Setup must be one plugin
  line and one script call. Errors must name the clip.
- **Black (risks):** The runtime is large. It loads only inside compositions, not on
  every page. HyperFrames releases often; the pin can age. The player API is
  0.x and can change. The plugin does not render MP4 files (that needs Node,
  Chrome and FFmpeg); it gives documents that the HyperFrames CLI can render.
- **Yellow (benefits):** Compositions get compile-time types and build-time
  checks that the HyperFrames linter does only later. The same Rust value gives a
  page, a nested file and an embedded player. Works with htmx swaps.
- **Green (ideas):** Type state (`CompositionBuilder` to `Composition`). A golden
  fixture for the Rust/JS attribute contract. `Composition::resolved_start()` to
  test timing without a browser. Later: a `render` task that calls the CLI,
  `setRuntimeData`, the slideshow element, GSAP plugin integration.
- **Blue (process):** Red, green, refactor for each module: assets, plugin,
  script tags, ids, time, clips, variables, composition, player, controls,
  `init.js`. Then demo, e2e, CI, docs. Then a multi-angle agent review.
  Then the AC evidence table.

## 5. Scope

In scope: section 2.
Out of scope for 0.1.0: MP4 rendering, the HyperFrames SDK (editing), Studio,
the slideshow element, `setRuntimeData`, shader transitions, audio effect chains,
color grading JSON. Users can write these attributes by hand in clip content.

## 6. Acceptance criteria

1. `HyperframesPlugin` installs the bundle. All files serve under
   `/static/_plugins/hyperframes/` with hashed, immutable URLs, ETag/304 and SRI.
2. The plugin does not serve `manifest.json`. The bundle does not force
   `embed-assets` on the host.
3. A test pins the `sha384` of each vendored upstream file. The manifest records
   version, source URL and license. CI compares the files with npm.
4. `hyperframes_script()` emits deferred SRI tags: player, then `init.js`.
   `hyperframes_stylesheet()` emits the host CSS link.
5. `Composition` covers id, size, duration, timeline flag, stylesheets, scripts,
   variables and clips. Clips cover HTML, image, video, audio and nested
   compositions, with start, duration, track, classes and media options.
6. `build()` rejects bad ids, duplicate ids, missing or cyclic references, bad
   ranges, unsafe URLs and bad variables. It returns all errors.
7. A `Composition` renders as a document (with the runtime), a template and a
   fragment. The output follows the HyperFrames HTML schema.
8. `Player` renders `<hyperframes-player>` for a composition (`srcdoc`), a URL or a
   video file. It always sets `runtime-src` to the vendored runtime.
   An option that you do not set writes no attribute.
9. `init.js` handles control buttons, in-view play, reduced motion,
   `data-hf-state`, `htmx:load` and `htmx:beforeCleanupElement`. A bad value
   does not stop the scan.
10. Rust and JS agree on the `data-hf-*` attributes (golden fixture, tested on
    both sides).
11. The plugin passes the Autumn plugin conformance check. Asset routes are public
    plugin routes.
12. A runnable demo shows each feature. It works under the default CSP with no
    console errors and no request to another origin.
13. Playwright e2e tests prove the browser behavior: the player gets ready, clips
    show in their time window, controls work, `src` mode works with the frame
    config, htmx swap works, reduced motion stops autoplay.
14. CI runs fmt, clippy (pedantic, nursery), tests, docs, MSRV, coverage
    (>= 85 % lines), JS tests, upstream check and e2e.
15. Docs (README, ADR, rustdoc, CLAUDE.md, CHANGELOG) are short and use ASD-STE100 style.

## 7. Specification (invariants)

`build()` enforces these rules. Unit tests and property tests (`proptest`) check
them. (Verus is not available in this environment; property tests take its place.)

| Rule | Statement |
|---|---|
| S1 | Each id matches `[A-Za-z][A-Za-z0-9_-]{0,127}`. |
| S2 | The root id and all clip ids are unique. A nested composition id can repeat (the runtime gives each copy its own id), but it is not the root id. |
| S3 | Width and height are in `1..=16384`. |
| S4 | A composition without a timeline has a duration. Each set duration rounds to at least 1 ms. |
| S5 | HTML and nested composition clips have a duration. |
| S6 | Each `Start::after` reference names another clip in the same composition. The references make no cycle. |
| S7 | Volume is finite and in `0..=3.98`. Playback rate is finite and in `0.1..=10`. |
| S8 | Each URL is on the allowlist for its use. Pages, nested files, stylesheets and scripts: relative or `http(s)`. Media: also `blob:` and `data:` images (not SVG), videos and audio. |
| S9 | Variable ids follow S1 and are unique. An enum has options and its default is one of them. Numbers are finite. |
| S10 | Each `bind_src` names a declared variable. |
| S11 | `resolved_start` is `max(0, end(ref) + offset)` for a reference and the start time for an absolute start (the runtime rule). |
| S12 | A nested composition with relative starts plays only one time. (The runtime finds a reference with `getElementById` in the whole page.) |
| S13 | HTML content has no element id that is a clip id or the root id. |

## 8. Test plan

| Layer | Tool | What |
|---|---|---|
| Rust unit | `cargo test` | Assets, plugin routes, conformance, script tags, ids, time, clips, variables, composition, player, controls. |
| Rust property | `proptest` | S1, S2, S6, S11 and the time format grammar. |
| Golden | Rust + Node | `tests/fixtures/attributes.json`: Rust makes it, JS parses it. |
| JS unit | `node --test` | `init.js` parsers and handlers in a `vm` sandbox. |
| Browser e2e | Playwright | The demo app in Chromium, default CSP. |
