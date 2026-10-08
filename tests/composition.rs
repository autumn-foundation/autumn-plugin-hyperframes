//! Composition model: build checks (spec S1-S11) and HTML output.

use std::time::Duration;

use autumn_plugin_hyperframes::{
    BuildError, Clip, Composition, CompositionError, HYPERFRAMES_ASSETS, Start, Variable,
    VideoAudio,
};
use autumn_web::html;

fn secs(s: f64) -> Duration {
    Duration::from_secs_f64(s)
}

fn errors(result: Result<Composition, BuildError>) -> Vec<CompositionError> {
    result.expect_err("build fails").errors().to_vec()
}

fn intro() -> Composition {
    Composition::builder("intro")
        .size(1280, 720)
        .duration(secs(6.0))
        .stylesheet("/static/css/intro.css")
        .clip(
            Clip::html("title", html! { h1 { "Hello" } })
                .duration(secs(3.0))
                .track(1)
                .class("title"),
        )
        .clip(
            Clip::image("logo", "/static/logo.png")
                .start(Start::after("title").minus(secs(0.5)))
                .alt("Logo"),
        )
        .clip(
            Clip::video("demo", "/static/demo.mp4", VideoAudio::Muted)
                .start(Start::after("logo"))
                .media_start(secs(1.25))
                .playback_rate(1.5),
        )
        .clip(
            Clip::audio("music", "/static/music.mp3")
                .volume(0.8)
                .fade_in(secs(1.0))
                .fade_out(secs(2.0)),
        )
        .build()
        .expect("valid composition")
}

// ---- Build checks ---------------------------------------------------------

#[test]
fn a_valid_composition_builds() {
    let comp = intro();
    assert_eq!(comp.id().as_str(), "intro");
    assert_eq!((comp.width(), comp.height()), (1280, 720));
    assert_eq!(comp.duration(), Some(secs(6.0)));
    assert_eq!(comp.clip_ids(), ["title", "logo", "demo", "music"]);
}

#[test]
fn default_size_is_full_hd() {
    let comp = Composition::builder("c").duration(secs(1.0)).build().expect("ok");
    assert_eq!((comp.width(), comp.height()), (1920, 1080));
}

#[test]
fn s1_rejects_bad_ids() {
    for bad in ["", "12", "1abc", "has space", "a.b", "a:b", "a+b", "ä", &"a".repeat(129)] {
        let errs = errors(Composition::builder(bad).duration(secs(1.0)).build());
        assert!(
            errs.contains(&CompositionError::InvalidId { id: bad.to_owned() }),
            "{bad}: {errs:?}"
        );
        let errs = errors(
            Composition::builder("ok")
                .duration(secs(1.0))
                .clip(Clip::html(bad, html! {}).duration(secs(1.0)))
                .build(),
        );
        assert!(
            errs.contains(&CompositionError::InvalidId { id: bad.to_owned() }),
            "{bad}: {errs:?}"
        );
    }
}

#[test]
fn s1_accepts_good_ids() {
    for good in ["a", "intro", "card-pro", "Scene_2", &"a".repeat(128)] {
        Composition::builder(good)
            .duration(secs(1.0))
            .build()
            .unwrap_or_else(|e| panic!("{good}: {e}"));
    }
}

#[test]
fn s2_rejects_duplicate_ids() {
    let errs = errors(
        Composition::builder("intro")
            .duration(secs(4.0))
            .clip(Clip::html("intro", html! {}).duration(secs(1.0)))
            .clip(Clip::html("a", html! {}).duration(secs(1.0)))
            .clip(Clip::image("a", "/x.png"))
            .clip(Clip::composition("b", "/b.html").duration(secs(1.0)).composition_id("a"))
            .build(),
    );
    assert_eq!(
        errs.iter()
            .filter(|e| matches!(e, CompositionError::DuplicateId { .. }))
            .count(),
        3,
        "{errs:?}"
    );
}

#[test]
fn s3_rejects_bad_sizes() {
    for (w, h) in [(0, 1080), (1920, 0), (16385, 10), (10, 16385)] {
        let errs = errors(Composition::builder("c").size(w, h).duration(secs(1.0)).build());
        assert_eq!(errs, [CompositionError::InvalidSize { width: w, height: h }]);
    }
    Composition::builder("c")
        .size(16384, 1)
        .duration(secs(1.0))
        .build()
        .expect("edges are valid");
}

#[test]
fn s4_timeline_free_composition_needs_a_duration() {
    let errs = errors(Composition::builder("c").build());
    assert_eq!(errs, [CompositionError::MissingDuration { id: "c".into() }]);
    Composition::builder("c")
        .with_timeline()
        .build()
        .expect("a timeline gives the duration");
}

#[test]
fn s4_durations_must_be_at_least_one_millisecond() {
    let errs = errors(
        Composition::builder("c")
            .duration(Duration::from_micros(400))
            .clip(Clip::image("i", "/i.png").duration(Duration::ZERO))
            .build(),
    );
    assert_eq!(
        errs,
        [
            CompositionError::ZeroDuration { id: "c".into() },
            CompositionError::ZeroDuration { id: "i".into() },
        ]
    );
}

#[test]
fn s5_html_and_nested_clips_need_a_duration() {
    let errs = errors(
        Composition::builder("c")
            .duration(secs(1.0))
            .clip(Clip::html("h", html! {}))
            .clip(Clip::composition("n", "/n.html"))
            .clip(Clip::image("i", "/i.png"))
            .clip(Clip::video("v", "/v.mp4", VideoAudio::HasAudio))
            .clip(Clip::audio("a", "/a.mp3"))
            .build(),
    );
    assert_eq!(
        errs,
        [
            CompositionError::ClipNeedsDuration { clip: "h".into() },
            CompositionError::ClipNeedsDuration { clip: "n".into() },
        ]
    );
}

#[test]
fn s6_references_must_exist_and_not_cycle() {
    let errs = errors(
        Composition::builder("c")
            .duration(secs(1.0))
            .clip(Clip::html("a", html! {}).duration(secs(1.0)).start(Start::after("ghost")))
            .clip(Clip::html("b", html! {}).duration(secs(1.0)).start(Start::after("b")))
            .clip(Clip::html("x", html! {}).duration(secs(1.0)).start(Start::after("y")))
            .clip(Clip::html("y", html! {}).duration(secs(1.0)).start(Start::after("x")))
            .build(),
    );
    assert_eq!(
        errs,
        [
            CompositionError::UnknownReference {
                clip: "a".into(),
                reference: "ghost".into()
            },
            CompositionError::SelfReference { clip: "b".into() },
            CompositionError::ReferenceCycle {
                clips: vec!["x".into(), "y".into()]
            },
        ]
    );
}

#[test]
fn s6_a_reference_to_the_root_is_unknown() {
    let errs = errors(
        Composition::builder("c")
            .duration(secs(1.0))
            .clip(Clip::image("a", "/a.png").start(Start::after("c")))
            .build(),
    );
    assert_eq!(
        errs,
        [CompositionError::UnknownReference {
            clip: "a".into(),
            reference: "c".into()
        }]
    );
}

#[test]
fn s7_rejects_out_of_range_volume_and_rate() {
    let errs = errors(
        Composition::builder("c")
            .duration(secs(1.0))
            .clip(Clip::audio("a", "/a.mp3").volume(4.0))
            .clip(Clip::audio("b", "/b.mp3").volume(-0.1))
            .clip(Clip::audio("d", "/d.mp3").volume(f64::NAN))
            .clip(Clip::video("e", "/e.mp4", VideoAudio::Muted).playback_rate(0.05))
            .clip(Clip::composition("f", "/f.html").duration(secs(1.0)).playback_rate(11.0))
            .clip(Clip::audio("g", "/g.mp3").playback_rate(f64::INFINITY))
            .build(),
    );
    assert_eq!(errs.len(), 6, "{errs:?}");
    assert!(errs.iter().all(|e| matches!(
        e,
        CompositionError::VolumeOutOfRange { .. } | CompositionError::PlaybackRateOutOfRange { .. }
    )));
    Composition::builder("c")
        .duration(secs(1.0))
        .clip(Clip::audio("a", "/a.mp3").volume(0.0).playback_rate(0.1))
        .clip(Clip::audio("b", "/b.mp3").volume(3.98).playback_rate(10.0))
        .build()
        .expect("range edges are valid");
}

#[test]
fn s8_rejects_unsafe_and_empty_urls() {
    let errs = errors(
        Composition::builder("c")
            .duration(secs(1.0))
            .stylesheet("javascript:alert(1)")
            .script(" ")
            .clip(Clip::image("a", "JaVa\tScRiPt:alert(1)"))
            .clip(Clip::video("b", "vbscript:x", VideoAudio::Muted))
            .clip(Clip::composition("d", "data:text/html,<b>").duration(secs(1.0)))
            .clip(Clip::audio("e", ""))
            .build(),
    );
    assert_eq!(errs.len(), 6, "{errs:?}");
    assert!(errs.iter().all(|e| matches!(e, CompositionError::UnsafeUrl { .. })));
    Composition::builder("c")
        .duration(secs(1.0))
        .clip(Clip::image("a", "data:image/png;base64,AAAA"))
        .clip(Clip::image("b", "https://example.com/x.png"))
        .build()
        .expect("data images and https URLs are valid");
}

#[test]
fn s9_rejects_bad_variables() {
    let errs = errors(
        Composition::builder("c")
            .duration(secs(1.0))
            .variable(Variable::string("1bad", "x"))
            .variable(Variable::string("title", "x"))
            .variable(Variable::color("title", "#fff"))
            .variable(Variable::number("n", f64::NAN))
            .variable(Variable::enumeration("e", &[], "a"))
            .variable(Variable::enumeration("f", &["a", "b"], "c"))
            .build(),
    );
    assert_eq!(
        errs,
        [
            CompositionError::InvalidId { id: "1bad".into() },
            CompositionError::DuplicateVariable { variable: "title".into() },
            CompositionError::InvalidVariable {
                variable: "n".into(),
                reason: "the default is not a finite number"
            },
            CompositionError::InvalidVariable {
                variable: "e".into(),
                reason: "an enum needs at least one option"
            },
            CompositionError::InvalidVariable {
                variable: "f".into(),
                reason: "the default is not one of the options"
            },
        ]
    );
}

#[test]
fn s9_rejects_bad_nested_values() {
    let errs = errors(
        Composition::builder("c")
            .duration(secs(1.0))
            .clip(
                Clip::composition("n", "/n.html")
                    .duration(secs(1.0))
                    .value("9x", "a")
                    .value("count", f64::INFINITY),
            )
            .build(),
    );
    assert_eq!(
        errs,
        [
            CompositionError::InvalidId { id: "9x".into() },
            CompositionError::InvalidValue {
                clip: "n".into(),
                name: "count".into(),
                reason: "the value is not a finite number"
            },
        ]
    );
}

#[test]
fn s10_bind_src_needs_a_declared_variable() {
    let errs = errors(
        Composition::builder("c")
            .duration(secs(1.0))
            .variable(Variable::image("logo", "/logo.png"))
            .clip(Clip::image("a", "/a.png").bind_src("logo"))
            .clip(Clip::image("b", "/b.png").bind_src("missing"))
            .build(),
    );
    assert_eq!(
        errs,
        [CompositionError::UnknownVariable {
            clip: "b".into(),
            variable: "missing".into()
        }]
    );
}

#[test]
fn s11_resolved_start_follows_the_runtime_rule() {
    let comp = intro();
    assert_eq!(comp.resolved_start("title"), Some(secs(0.0)));
    assert_eq!(comp.resolved_end("title"), Some(secs(3.0)));
    // After `title`, minus 0.5 s.
    assert_eq!(comp.resolved_start("logo"), Some(secs(2.5)));
    // An image without a duration shows for 3 s.
    assert_eq!(comp.resolved_end("logo"), Some(secs(5.5)));
    assert_eq!(comp.resolved_start("demo"), Some(secs(5.5)));
    // A video without a duration has an unknown end (source length).
    assert_eq!(comp.resolved_end("demo"), None);
    assert_eq!(comp.resolved_start("ghost"), None);
}

#[test]
fn s11_a_negative_relative_start_clamps_to_zero() {
    let comp = Composition::builder("c")
        .duration(secs(5.0))
        .clip(Clip::html("a", html! {}).duration(secs(1.0)))
        .clip(Clip::html("b", html! {}).duration(secs(1.0)).start(Start::after("a").minus(secs(4.0))))
        .clip(Clip::audio("v", "/v.mp3"))
        .clip(Clip::html("d", html! {}).duration(secs(1.0)).start(Start::after("v")))
        .build()
        .expect("ok");
    assert_eq!(comp.resolved_start("b"), Some(Duration::ZERO));
    assert_eq!(comp.resolved_start("d"), None, "the audio end is unknown");
}

#[test]
fn build_error_lists_every_error() {
    let err = Composition::builder("1").size(0, 0).build().expect_err("fails");
    let text = err.to_string();
    assert!(text.starts_with("3 composition errors"), "{text}");
    assert!(text.contains("`1` is not a valid id"), "{text}");
}

// ---- Output ---------------------------------------------------------------

#[test]
fn fragment_has_the_root_contract() {
    let html = intro().fragment().into_string();
    assert!(
        html.starts_with(
            r#"<div id="intro" class="hf-root" data-composition-id="intro" data-start="0" data-duration="6" data-width="1280" data-height="720" data-no-timeline style="width:1280px;height:720px">"#
        ),
        "{html}"
    );
    assert!(html.ends_with("</div>"), "{html}");
}

#[test]
fn clips_render_the_schema_attributes() {
    let html = intro().fragment().into_string();
    for part in [
        r#"<div id="title" class="clip title" data-start="0" data-duration="3" data-track-index="1"><h1>Hello</h1></div>"#,
        r#"<img id="logo" class="clip" src="/static/logo.png" alt="Logo" data-start="title - 0.5">"#,
        r#"<video id="demo" class="clip" src="/static/demo.mp4" data-start="logo" data-media-start="1.25" data-playback-rate="1.5" muted playsinline></video>"#,
        r#"<audio id="music" src="/static/music.mp3" data-start="0" data-volume="0.8" data-fade-in="1" data-fade-out="2"></audio>"#,
    ] {
        assert!(html.contains(part), "missing {part}\nin {html}");
    }
}

#[test]
fn a_video_with_audio_says_so_and_is_not_muted() {
    let comp = Composition::builder("c")
        .duration(secs(1.0))
        .clip(Clip::video("v", "/v.mp4", VideoAudio::HasAudio).volume(1.2))
        .build()
        .expect("ok");
    let html = comp.fragment().into_string();
    assert!(
        html.contains(r#"<video id="v" class="clip" src="/v.mp4" data-start="0" data-volume="1.2" data-has-audio="true" playsinline></video>"#),
        "{html}"
    );
    assert!(!html.contains("muted"), "{html}");
}

#[test]
fn nested_composition_clip_renders_the_host_contract() {
    let comp = Composition::builder("c")
        .duration(secs(9.0))
        .clip(
            Clip::composition("pricing-scene", "/compositions/pricing")
                .composition_id("pricing")
                .start(Start::at(secs(4.0)))
                .duration(secs(5.0))
                .playback_start(secs(0.5))
                .size(1920, 1080)
                .value("title", "Pro")
                .value("count", 3)
                .value("featured", true)
                .track(2),
        )
        .build()
        .expect("ok");
    let html = comp.fragment().into_string();
    assert!(
        html.contains(r#"<div id="pricing-scene" data-composition-id="pricing" data-composition-src="/compositions/pricing" data-start="4" data-duration="5" data-track-index="2" data-playback-start="0.5" data-width="1920" data-height="1080" data-variable-values="{&quot;count&quot;:3,&quot;featured&quot;:true,&quot;title&quot;:&quot;Pro&quot;}"></div>"#),
        "{html}"
    );
}

#[test]
fn nested_composition_id_defaults_to_the_clip_id() {
    let comp = Composition::builder("c")
        .duration(secs(1.0))
        .clip(Clip::composition("pricing", "/p").duration(secs(1.0)))
        .build()
        .expect("ok");
    assert!(comp
        .fragment()
        .into_string()
        .contains(r#"<div id="pricing" data-composition-id="pricing" data-composition-src="/p""#));
}

#[test]
fn with_timeline_drops_data_no_timeline() {
    let comp = Composition::builder("c").with_timeline().build().expect("ok");
    let html = comp.fragment().into_string();
    assert!(!html.contains("data-no-timeline"), "{html}");
    assert!(!html.contains("data-duration"), "{html}");
}

#[test]
fn document_is_a_full_page_with_the_runtime() {
    let comp = Composition::builder("intro")
        .size(1280, 720)
        .duration(secs(2.0))
        .title("Intro video")
        .stylesheet("/static/css/intro.css")
        .script("/static/js/intro.js")
        .variable(Variable::string("title", "Pro").label("Title"))
        .build()
        .expect("ok");
    let html = comp.document().into_string();
    let runtime = HYPERFRAMES_ASSETS.get("hyperframe.runtime.iife.js").expect("runtime");
    let css = HYPERFRAMES_ASSETS.get("composition.css").expect("css");
    assert!(html.starts_with("<!DOCTYPE html><html lang=\"en\" data-composition-variables=\"[{&quot;default&quot;:&quot;Pro&quot;,&quot;id&quot;:&quot;title&quot;,&quot;label&quot;:&quot;Title&quot;,&quot;type&quot;:&quot;string&quot;}]\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=1280, height=720\"><title>Intro video</title>"), "{html}");
    for part in [
        format!(r#"<link rel="stylesheet" href="{}" integrity="{}" crossorigin="anonymous">"#, css.url(), css.integrity()),
        r#"<link rel="stylesheet" href="/static/css/intro.css">"#.to_owned(),
        format!(r#"<script src="{}" integrity="{}" crossorigin="anonymous"></script>"#, runtime.url(), runtime.integrity()),
        r#"<script src="/static/js/intro.js"></script></body></html>"#.to_owned(),
    ] {
        assert!(html.contains(&part), "missing {part}\nin {html}");
    }
    // The runtime loads in <head>, before the body scripts.
    assert!(html.find(runtime.url()) < html.find("<body>"), "{html}");
    // The root in a document has no variables attribute (it is on <html>).
    assert_eq!(html.matches("data-composition-variables").count(), 1, "{html}");
}

#[test]
fn template_wraps_the_root_for_nested_use() {
    let comp = Composition::builder("pricing")
        .duration(secs(2.0))
        .script("/static/js/pricing.js")
        .variable(Variable::number("count", 3.0))
        .build()
        .expect("ok");
    let html = comp.template().into_string();
    assert!(html.starts_with(r#"<template id="pricing-template">"#), "{html}");
    assert!(html.ends_with("</template>"), "{html}");
    assert!(
        html.contains(r#"<div id="pricing" class="hf-root" data-composition-id="pricing" data-duration="2""#),
        "a nested root has no data-start: {html}"
    );
    assert!(html.contains("data-composition-variables="), "{html}");
    assert!(html.contains(r#"<script src="/static/js/pricing.js"></script>"#), "{html}");
    assert!(!html.contains("hyperframe.runtime"), "the parent loads the runtime: {html}");
}

#[test]
fn srcdoc_html_has_no_runtime_tag() {
    let html = intro().srcdoc_html();
    assert!(html.starts_with("<!DOCTYPE html>"), "{html}");
    assert!(!html.contains("hyperframe.runtime"), "the player inserts it: {html}");
    assert!(html.contains("composition."), "{html}");
}

#[test]
fn variables_render_every_type() {
    let comp = Composition::builder("c")
        .duration(secs(1.0))
        .variable(Variable::string("s", "x").description("Text"))
        .variable(Variable::number("n", 2.5))
        .variable(Variable::color("c2", "#ff0000"))
        .variable(Variable::boolean("b", true))
        .variable(Variable::enumeration("e", &["a", "b"], "b"))
        .variable(Variable::font("f", "Inter"))
        .variable(Variable::image("i", "/i.png"))
        .build()
        .expect("ok");
    let html = comp.fragment().into_string().replace("&quot;", "\"");
    for part in [
        r#"{"default":"x","description":"Text","id":"s","label":"s","type":"string"}"#,
        r#"{"default":2.5,"id":"n","label":"n","type":"number"}"#,
        r##"{"default":"#ff0000","id":"c2","label":"c2","type":"color"}"##,
        r#"{"default":true,"id":"b","label":"b","type":"boolean"}"#,
        r#"{"default":"b","id":"e","label":"e","options":[{"label":"a","value":"a"},{"label":"b","value":"b"}],"type":"enum"}"#,
        r#"{"default":"Inter","id":"f","label":"f","type":"font"}"#,
        r#"{"default":"/i.png","id":"i","label":"i","type":"image"}"#,
    ] {
        assert!(html.contains(part), "missing {part}\nin {html}");
    }
}

#[test]
fn bind_src_writes_data_var_src() {
    let comp = Composition::builder("c")
        .duration(secs(1.0))
        .variable(Variable::image("logo", "/logo.png"))
        .clip(Clip::image("a", "/logo.png").bind_src("logo"))
        .build()
        .expect("ok");
    assert!(comp.fragment().into_string().contains(r#"data-var-src="logo""#));
}

#[test]
fn text_and_urls_are_escaped() {
    let comp = Composition::builder("c")
        .duration(secs(1.0))
        .title("<b>&")
        .clip(Clip::image("a", r#"/a.png"onerror="x"#).alt(r#""><script>"#))
        .build()
        .expect("ok");
    let html = comp.document().into_string();
    assert!(html.contains("<title>&lt;b&gt;&amp;</title>"), "{html}");
    assert!(html.contains(r#"src="/a.png&quot;onerror=&quot;x""#), "{html}");
    assert!(!html.contains("<script>\""), "{html}");
}

#[test]
fn times_round_to_milliseconds() {
    let comp = Composition::builder("c")
        .duration(Duration::from_nanos(1_234_500_000))
        .clip(Clip::image("a", "/a.png").start(Start::at(Duration::from_micros(2_000_400))))
        .build()
        .expect("ok");
    let html = comp.fragment().into_string();
    assert!(html.contains(r#"data-duration="1.235""#), "{html}");
    assert!(html.contains(r#"data-start="2""#), "{html}");
}
