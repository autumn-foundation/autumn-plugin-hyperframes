# Changelog

## 0.1.0 (2026-10-08)

First release. Target: autumn-web 0.8.0, HyperFrames 0.8.140.

- `HyperframesPlugin` serves the vendored player, runtime, `init.js` and styles.
- `hyperframes_script()` and `hyperframes_stylesheet()` for the host page.
- `Composition` builder with build checks (ids, references, loops, ranges, URLs, variables).
- `Clip` kinds: HTML, image, video, audio, nested composition. `Clip::nested` copies the inner contract.
- `Start` for absolute and relative times. `resolved_start` / `resolved_end`.
- Outputs: `document()`, `template()` (a page with `<template>`), `fragment()`, `srcdoc_html()`.
- URL allowlist for each use. `Id` with `TryFrom` and `FromStr`.
- `Player` for `srcdoc`, URL and video sources. Always sets `runtime-src`.
- `PlayerControl` buttons. `init.js`: controls, autoplay, in-view play, reduced motion, htmx.
