# CLAUDE.md

Autumn plugin for HyperFrames. Rust crate plus one JS file (`assets/init.js`).

## Layout

- `src/assets.rs`: `HYPERFRAMES_ASSETS` bundle, version and upstream hash pins.
- `src/plugin.rs`: `HyperframesPlugin` (installs the bundle).
- `src/script.rs`: `hyperframes_script()`, `hyperframes_stylesheet()`.
- `src/composition.rs`: `CompositionBuilder` (checks S1-S10) and `Composition` (renders).
- `src/clip.rs`: `Clip<K>` and the sealed kinds. `src/variable.rs`: variables.
- `src/time.rs`: `Start` and the seconds format. `src/id.rs`: `Id`. `src/url.rs`: URL check.
- `src/player.rs`: `Player`. `src/control.rs`: `PlayerControl`.
- `assets/init.js`: the scanner. `assets/*.css`: styles. `assets/manifest.json`: provenance (not served).
- `tests/composition.rs`, `tests/player.rs`, `tests/properties.rs`: public API tests.
- `tests/golden.rs` + `tests/fixtures/attributes.json`: the Rust/JS contract.
- `tests/js/`: `node:test` tests for `init.js`. `tests/e2e/`: Playwright tests against `examples/hyperframes_demo.rs`.
- `docs/plan.md`: plan, AC and the spec rules S1-S11. `docs/adr/`: decisions.

## Rules

- Write tests first (red, green, refactor).
- A new build rule needs a row in `docs/plan.md` section 7 and a test.
- A new `data-hf-*` attribute needs a parser in `init.js`, a golden case and a README row.
- `RE_SEEK` in `init.js` must match the Rust seconds format (a test checks it).
- After an attribute change, run `UPDATE_GOLDEN=1 cargo test --test golden`.
- Do not change the vendored HyperFrames files. To upgrade, replace them, then update the pins and `manifest.json`.
- No inline scripts in the demo or in compositions.
- Docs and comments: short, active voice, ASD-STE100 style.

## Checks

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps
node --test tests/js/*.test.mjs
cargo build --example hyperframes_demo && npm --prefix tests/e2e ci && npm --prefix tests/e2e test
cargo llvm-cov --fail-under-lines 85
```
