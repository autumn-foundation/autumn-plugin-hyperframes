//! The Rust/JS contract for the `data-hf-*` attributes (AC 10).
//!
//! Rust writes `tests/fixtures/attributes.json`. `tests/js/init.test.mjs`
//! parses each case with `init.js`. Run `UPDATE_GOLDEN=1 cargo test --test golden`
//! after an attribute change.

use std::fmt::Write as _;
use std::time::Duration;

use autumn_plugin_hyperframes::{Control, Player, PlayerControl, ReducedMotion};
use regex::Regex;

const FIXTURE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/attributes.json"
);

fn attr(html: &str, name: &str) -> Option<String> {
    let re = Regex::new(&format!(r#" {name}(?:="([^"]*)")?[ >]"#)).expect("regex");
    re.captures(html)
        .map(|c| c.get(1).map_or_else(String::new, |m| m.as_str().to_owned()))
}

fn json_str(value: Option<&str>) -> String {
    value.map_or_else(|| "null".to_owned(), |v| format!("\"{v}\""))
}

fn render_fixture() -> String {
    let mut out = String::from("{\n  \"controls\": [\n");
    let controls = [
        (Control::Play, "play", None),
        (Control::Pause, "pause", None),
        (Control::Toggle, "toggle", None),
        (Control::Restart, "restart", None),
        (Control::Mute, "mute", None),
        (Control::Unmute, "unmute", None),
        (Control::ToggleMute, "toggle-mute", None),
        (Control::Seek(Duration::ZERO), "seek", Some(0.0)),
        (
            Control::Seek(Duration::from_millis(2500)),
            "seek",
            Some(2.5),
        ),
        (
            Control::Seek(Duration::from_millis(1234)),
            "seek",
            Some(1.234),
        ),
        (
            Control::Seek(Duration::from_secs(3600)),
            "seek",
            Some(3600.0),
        ),
    ];
    for (i, (control, name, seconds)) in controls.into_iter().enumerate() {
        let html = PlayerControl::new("p", control).render().into_string();
        let control_attr = attr(&html, "data-hf-control");
        let seek_attr = attr(&html, "data-hf-seek");
        assert_eq!(control_attr.as_deref(), Some(name), "{html}");
        let _ = write!(
            out,
            "    {{ \"control\": {}, \"seek\": {}, \"expect\": {{ \"control\": \"{name}\", \"seek\": {} }} }}{}\n",
            json_str(control_attr.as_deref()),
            json_str(seek_attr.as_deref()),
            seconds.map_or_else(|| "null".to_owned(), |s: f64| s.to_string()),
            if i + 1 < controls_len() { "," } else { "" },
        );
    }
    out.push_str("  ],\n  \"players\": [\n");
    let players = [
        (Player::src("/c"), false, None),
        (Player::src("/c").in_view(), true, None),
        (
            Player::src("/c")
                .in_view()
                .reduced_motion(ReducedMotion::Animate),
            true,
            Some("animate"),
        ),
        (
            Player::src("/c").reduced_motion(ReducedMotion::Skip),
            false,
            None,
        ),
    ];
    let n = players.len();
    for (i, (player, in_view, reduced)) in players.into_iter().enumerate() {
        let html = player.render().into_string();
        let in_view_attr = attr(&html, "data-hf-in-view").is_some();
        let reduced_attr = attr(&html, "data-hf-reduced");
        assert_eq!(in_view_attr, in_view, "{html}");
        assert_eq!(reduced_attr.as_deref(), reduced, "{html}");
        let _ = write!(
            out,
            "    {{ \"inView\": {in_view_attr}, \"reduced\": {}, \"expect\": {{ \"inView\": {in_view}, \"animateWhenReduced\": {} }} }}{}\n",
            json_str(reduced_attr.as_deref()),
            reduced == Some("animate"),
            if i + 1 < n { "," } else { "" },
        );
    }
    out.push_str("  ],\n  \"invalid\": {\n");
    out.push_str("    \"controls\": [\"\", \"PLAY\", \"stop\", \"seek \", \"toggle_mute\"],\n");
    out.push_str("    \"seeks\": [\"\", \"-1\", \"1e3\", \"0x10\", \"1.2345\", \"abc\", \" 1\", \"Infinity\"]\n");
    out.push_str("  }\n}\n");
    out
}

const fn controls_len() -> usize {
    11
}

#[test]
fn fixture_matches_rust_output() {
    let rendered = render_fixture();
    if std::env::var_os("UPDATE_GOLDEN").is_some() {
        std::fs::write(FIXTURE, &rendered).expect("write fixture");
    }
    let stored = std::fs::read_to_string(FIXTURE).expect("fixture exists");
    assert_eq!(
        stored, rendered,
        "run UPDATE_GOLDEN=1 cargo test --test golden"
    );
}
