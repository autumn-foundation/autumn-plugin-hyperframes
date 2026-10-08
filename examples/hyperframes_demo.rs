//! HyperFrames demo: a small Autumn app that shows `autumn-plugin-hyperframes`.
//!
//! Run it, then open <http://127.0.0.1:3000>:
//!
//! ```sh
//! cargo run --example hyperframes_demo
//! ```
//!
//! The page shows these items:
//!
//! - A composition in a player (`srcdoc` mode), with control buttons.
//!   CSS animations in `static/css/intro.css` move the clips. The runtime seeks them.
//! - A composition with a nested composition (`Clip::composition`) and variable values.
//! - A muted player that plays when it is in view.
//! - An htmx button that adds one more player.
//!
//! `/src-mode` plays a composition page from its URL (`src` mode). It needs the
//! frame settings in `README.md` ("Play a composition from a URL").
//!
//! All JS and CSS are files, so the page works with the default Autumn CSP.

use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;

use autumn_plugin_hyperframes::{
    Clip, Composition, Control, HyperframesPlugin, Player, PlayerControl, Start, Variable,
    hyperframes_script, hyperframes_stylesheet,
};
use autumn_web::assets::asset_url;
use autumn_web::{Markup, html};

/// The crate `static/` directory, embedded in the binary.
static STATIC: autumn_web::include_dir::Dir = autumn_web::embed_static!();

/// Counts the players that htmx adds.
static CARDS: AtomicU32 = AtomicU32::new(1);

const fn secs(n: u64) -> Duration {
    Duration::from_secs(n)
}

const fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

#[autumn_web::main]
async fn main() {
    autumn_web::app()
        .plugin(HyperframesPlugin::new())
        .embedded_static(&STATIC)
        .routes(autumn_web::routes![
            index,
            src_mode,
            intro_page,
            pricing_template,
            card
        ])
        .run()
        .await;
}

/// The intro composition: a title, a tagline, a logo and a closing card.
fn intro() -> Composition {
    let css = asset_url("css/intro.css");
    Composition::builder("intro")
        .size(1280, 720)
        .duration(secs(6))
        .title("Intro")
        .stylesheet(&css)
        .clip(
            Clip::html("title", html! { h1 { "Write Rust." } })
                .duration(secs(2))
                .track(1)
                .class("scene")
                .class("rise"),
        )
        .clip(
            Clip::html("tagline", html! { h1 { "Get video." } })
                .start(Start::after("title"))
                .duration(secs(2))
                .track(1)
                .class("scene")
                .class("pop"),
        )
        .clip(
            Clip::image("logo", &asset_url("img/logo.svg"))
                .start(Start::after("title").minus(ms(500)))
                .duration(ms(2500))
                .track(2)
                .class("logo")
                .alt("Autumn logo"),
        )
        .clip(
            Clip::html("outro", html! { p { "autumn-plugin-hyperframes" } })
                .start(Start::after("tagline"))
                .duration(secs(2))
                .track(1)
                .class("scene")
                .class("fade"),
        )
        .build()
        .unwrap_or_else(|e| unreachable!("the demo composition is valid: {e}"))
}

/// A nested scene. `/compositions/pricing.html` serves its template.
fn pricing() -> Composition {
    Composition::builder("pricing")
        .size(1280, 720)
        .duration(secs(3))
        .stylesheet(&asset_url("css/intro.css"))
        .variable(Variable::string("plan", "Free").label("Plan name"))
        .clip(
            Clip::html("pricing-card", html! { h1 data-var-text="plan" { "Free" } })
                .duration(secs(3))
                .class("scene")
                .class("rise"),
        )
        .build()
        .unwrap_or_else(|e| unreachable!("the demo composition is valid: {e}"))
}

/// A host that plays the nested scene two times with other values.
fn plans() -> Composition {
    Composition::builder("plans")
        .size(1280, 720)
        .duration(secs(6))
        .stylesheet(&asset_url("css/intro.css"))
        .clip(
            Clip::composition("plan-pro", "/compositions/pricing.html")
                .composition_id("pricing")
                .duration(secs(3))
                .value("plan", "Pro"),
        )
        .clip(
            Clip::composition("plan-team", "/compositions/pricing.html")
                .composition_id("pricing")
                .start(Start::after("plan-pro"))
                .duration(secs(3))
                .value("plan", "Team"),
        )
        .build()
        .unwrap_or_else(|e| unreachable!("the demo composition is valid: {e}"))
}

/// A short card for the in-view and htmx players.
fn card_composition(n: u32) -> Composition {
    Composition::builder(&format!("card-{n}"))
        .size(1080, 1080)
        .duration(secs(3))
        .stylesheet(&asset_url("css/intro.css"))
        .clip(
            Clip::html(&format!("card-{n}-text"), html! { h1 { "Card " (n) } })
                .duration(secs(3))
                .class("scene")
                .class("pop"),
        )
        .build()
        .unwrap_or_else(|e| unreachable!("the demo composition is valid: {e}"))
}

fn layout(content: &Markup) -> Markup {
    html! {
        (maud::DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { "HyperFrames demo" }
                // htmx adds an inline <style> for indicators. A strict CSP blocks it.
                meta name="htmx-config" content=r#"{"includeIndicatorStyles":false}"#;
                link rel="stylesheet" href=(asset_url("css/demo.css"));
                (hyperframes_stylesheet())
                (hyperframes_script())
                script src=(asset_url("js/htmx.min.js")) defer {}
            }
            body {
                main class="wrap" { (content) }
            }
        }
    }
}

#[autumn_web::get("/")]
async fn index() -> Markup {
    layout(&html! {
        h1 { "HyperFrames in Autumn" }

        section id="intro-section" {
            h2 { "A composition in a player" }
            (Player::composition(&intro()).id("intro-player").label("Intro video").controls())
            div class="controls" {
                (PlayerControl::new("intro-player", Control::Play))
                (PlayerControl::new("intro-player", Control::Pause))
                (PlayerControl::new("intro-player", Control::Restart))
                (PlayerControl::new("intro-player", Control::Seek(ms(2500))).label("Go to the logo"))
                (PlayerControl::new("intro-player", Control::ToggleMute))
            }
            p { a href="/compositions/intro" { "Open the composition page" } }
        }

        section id="plans-section" {
            h2 { "Nested compositions with variables" }
            (Player::composition(&plans()).id("plans-player").label("Plans video").controls())
        }

        section id="in-view-section" class="tall" {
            h2 { "Plays in view" }
            (Player::composition(&card_composition(0))
                .id("in-view-player")
                .label("In-view card")
                .muted()
                .looped()
                .in_view())
        }

        section id="htmx-section" {
            h2 { "htmx" }
            button id="more" hx-get="/fragments/card" hx-target="#cards" hx-swap="beforeend" {
                "Add a player"
            }
            div id="cards" {}
        }
    })
}

/// A player that plays a composition page from its URL.
#[autumn_web::get("/src-mode")]
async fn src_mode() -> Markup {
    layout(&html! {
        h1 { "src mode" }
        (Player::src("/compositions/intro").size(1280, 720).id("src-player").controls())
        div class="controls" {
            (PlayerControl::new("src-player", Control::Play))
        }
    })
}

/// The intro composition as a page.
#[autumn_web::get("/compositions/intro")]
async fn intro_page() -> Markup {
    intro().document()
}

/// The nested scene as a template file.
#[autumn_web::get("/compositions/pricing.html")]
async fn pricing_template() -> Markup {
    pricing().template()
}

/// htmx partial: one more player.
#[autumn_web::get("/fragments/card")]
async fn card() -> Markup {
    let n = CARDS.fetch_add(1, Ordering::Relaxed);
    html! {
        div class="card" {
            (Player::composition(&card_composition(n))
                .id(&format!("card-player-{n}"))
                .label(&format!("Card {n}"))
                .controls())
            (PlayerControl::new(&format!("card-player-{n}"), Control::Play))
        }
    }
}
