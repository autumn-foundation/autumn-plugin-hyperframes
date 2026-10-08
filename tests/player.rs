//! Player and control markup (AC 8, 10).

use std::time::Duration;

use autumn_plugin_hyperframes::{
    Clip, Composition, Control, HYPERFRAMES_ASSETS, Player, PlayerControl, ReducedMotion,
    ShaderLoading, VideoType,
};
use autumn_web::html;

fn runtime_url() -> String {
    HYPERFRAMES_ASSETS.url("hyperframe.runtime.iife.js")
}

fn comp(width: u32, height: u32) -> Composition {
    Composition::builder("intro")
        .size(width, height)
        .duration(Duration::from_secs(3))
        .clip(Clip::html("title", html! { h1 { "Hi & bye" } }).duration(Duration::from_secs(3)))
        .build()
        .expect("valid")
}

#[test]
fn minimal_player_writes_only_the_source_and_runtime() {
    let html = Player::src("/compositions/intro").render().into_string();
    assert_eq!(
        html,
        format!(
            r#"<hyperframes-player src="/compositions/intro" runtime-src="{}"></hyperframes-player>"#,
            runtime_url()
        )
    );
}

#[test]
fn composition_player_uses_srcdoc_and_the_composition_size() {
    let c = comp(1280, 720);
    let html = Player::composition(&c).render().into_string();
    let escaped = maud::html! { (c.srcdoc_html()) }.into_string();
    assert!(
        html.starts_with(&format!(
            r#"<hyperframes-player srcdoc="{escaped}" width="1280" height="720" runtime-src="{}">"#,
            runtime_url()
        )),
        "{html}"
    );
    assert!(!html.contains(" src="), "{html}");
    // The srcdoc is escaped once: the markup is text in the attribute.
    assert!(
        html.contains("&lt;h1&gt;Hi &amp;amp; bye&lt;/h1&gt;"),
        "{html}"
    );
}

#[test]
fn every_option_maps_to_one_attribute() {
    let html = Player::src("/c")
        .id("intro-player")
        .class("hero")
        .label("Product intro")
        .size(1080, 1920)
        .controls()
        .muted()
        .autoplay()
        .looped()
        .poster("/static/poster.jpg")
        .playback_rate(1.5)
        .volume(0.5)
        .range(Duration::from_millis(500), Duration::from_secs(2))
        .audio_locked()
        .opaque_sandbox()
        .low_power_idle()
        .disable_click_to_play()
        .hide_loading_ui()
        .shader_loading(ShaderLoading::Player)
        .in_view()
        .reduced_motion(ReducedMotion::Animate)
        .render()
        .into_string();
    assert_eq!(
        html,
        format!(
            concat!(
                r#"<hyperframes-player id="intro-player" class="hf-ratio-9x16 hero" role="group" aria-label="Product intro" "#,
                r#"src="/c" width="1080" height="1920" runtime-src="{}" controls muted autoplay loop "#,
                r#"poster="/static/poster.jpg" playback-rate="1.5" volume="0.5" range-start="0.5" range-end="2" "#,
                r#"audio-locked sandbox-origin="opaque" low-power-idle disable-click-to-play assets-loading-ui="none" "#,
                r#"shader-loading="player" data-hf-in-view data-hf-reduced="animate"></hyperframes-player>"#
            ),
            runtime_url()
        )
    );
}

#[test]
fn video_player_sets_the_type() {
    let html = Player::video("/static/render.mp4", VideoType::Mp4)
        .controls()
        .render()
        .into_string();
    assert!(
        html.starts_with(r#"<hyperframes-player src="/static/render.mp4" type="video/mp4""#),
        "{html}"
    );
    assert_eq!(VideoType::WebM.mime(), "video/webm");
    assert_eq!(VideoType::Ogg.mime(), "video/ogg");
}

#[test]
fn numbers_clamp_to_the_player_ranges() {
    let html = Player::src("/c")
        .playback_rate(9.0)
        .volume(2.0)
        .render()
        .into_string();
    assert!(html.contains(r#"playback-rate="5""#), "{html}");
    assert!(html.contains(r#"volume="1""#), "{html}");
    let html = Player::src("/c")
        .playback_rate(0.0)
        .volume(-1.0)
        .render()
        .into_string();
    assert!(html.contains(r#"playback-rate="0.1""#), "{html}");
    assert!(html.contains(r#"volume="0""#), "{html}");
    let html = Player::src("/c")
        .playback_rate(f64::NAN)
        .volume(f64::INFINITY)
        .render()
        .into_string();
    assert!(!html.contains("playback-rate"), "{html}");
    assert!(!html.contains("volume"), "{html}");
}

#[test]
fn unsafe_urls_write_no_attribute() {
    let html = Player::src("javascript:alert(1)")
        .poster("vbscript:x")
        .render()
        .into_string();
    assert!(!html.contains("javascript"), "{html}");
    assert!(!html.contains("vbscript"), "{html}");
    assert!(!html.contains(" src="), "{html}");
}

#[test]
fn aspect_ratio_class_follows_the_size() {
    for ((w, h), class) in [
        ((1920, 1080), None),
        ((1080, 1920), Some("hf-ratio-9x16")),
        ((1080, 1080), Some("hf-ratio-1x1")),
        ((1080, 1350), Some("hf-ratio-4x5")),
        ((1024, 768), Some("hf-ratio-4x3")),
        ((2100, 900), Some("hf-ratio-21x9")),
        ((1000, 999), None),
    ] {
        let html = Player::composition(&comp(w, h)).render().into_string();
        match class {
            Some(class) => assert!(
                html.contains(&format!(r#"class="{class}""#)),
                "{w}x{h}: {html}"
            ),
            None => assert!(!html.contains("hf-ratio"), "{w}x{h}: {html}"),
        }
    }
}

#[test]
fn player_renders_inside_maud() {
    let player = Player::src("/c");
    let page = html! { main { (player) } }.into_string();
    assert!(page.starts_with("<main><hyperframes-player"), "{page}");
}

#[test]
fn controls_render_buttons_for_init_js() {
    let cases = [
        (Control::Play, r#"data-hf-control="play""#, "Play"),
        (Control::Pause, r#"data-hf-control="pause""#, "Pause"),
        (
            Control::Toggle,
            r#"data-hf-control="toggle""#,
            "Play or pause",
        ),
        (Control::Restart, r#"data-hf-control="restart""#, "Restart"),
        (Control::Mute, r#"data-hf-control="mute""#, "Mute"),
        (Control::Unmute, r#"data-hf-control="unmute""#, "Unmute"),
        (
            Control::ToggleMute,
            r#"data-hf-control="toggle-mute""#,
            "Mute or unmute",
        ),
    ];
    for (control, attr, label) in cases {
        let html = PlayerControl::new("intro", control).render().into_string();
        assert_eq!(
            html,
            format!(
                r#"<button type="button" {attr} data-hf-target="intro" aria-controls="intro">{label}</button>"#
            )
        );
    }
}

#[test]
fn seek_control_writes_the_time() {
    let html = PlayerControl::new("intro", Control::Seek(Duration::from_millis(2500)))
        .label("Go to the logo")
        .class("btn")
        .render()
        .into_string();
    assert_eq!(
        html,
        r#"<button type="button" class="btn" data-hf-control="seek" data-hf-target="intro" aria-controls="intro" data-hf-seek="2.5">Go to the logo</button>"#
    );
    let html = PlayerControl::new("intro", Control::Seek(Duration::from_secs(3)))
        .render()
        .into_string();
    assert!(html.ends_with(">Go to 3 s</button>"), "{html}");
}

#[test]
fn control_names_round_trip() {
    for control in [
        Control::Play,
        Control::Pause,
        Control::Toggle,
        Control::Restart,
        Control::Mute,
        Control::Unmute,
        Control::ToggleMute,
        Control::Seek(Duration::ZERO),
    ] {
        assert!(!control.name().is_empty());
    }
    assert_eq!(Control::Seek(Duration::ZERO).name(), "seek");
}
