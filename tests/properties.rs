//! Property tests for the composition invariants (spec S1, S2, S6, S11).

use std::time::Duration;

use autumn_plugin_hyperframes::{Clip, Composition, CompositionError, Start};
use autumn_web::html;
use proptest::prelude::*;
use regex::Regex;

const ID: &str = "[A-Za-z][A-Za-z0-9_-]{0,127}";

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

proptest! {
    // S1: every id in the grammar builds.
    #[test]
    fn s1_grammar_ids_build(id in ID) {
        prop_assert!(Composition::builder(&id).duration(ms(1000)).build().is_ok());
    }

    // S1: every id out of the grammar fails with InvalidId.
    #[test]
    fn s1_other_ids_fail(id in "\\PC{0,140}") {
        let valid = Regex::new(&format!("^{ID}$")).expect("regex").is_match(&id);
        let result = Composition::builder(&id).duration(ms(1000)).build();
        if valid {
            prop_assert!(result.is_ok());
        } else {
            let err = result.expect_err("fails");
            let expected = CompositionError::InvalidId { id: id.clone() };
            prop_assert!(err.errors().contains(&expected), "{:?}", err);
        }
    }

    // S2: the number of DuplicateId errors is the number of repeats.
    #[test]
    fn s2_duplicates_are_counted(picks in prop::collection::vec(0usize..5, 1..12)) {
        let names = ["a", "b", "c", "d", "e"];
        let mut builder = Composition::builder("root").duration(ms(1000));
        for &p in &picks {
            builder = builder.clip(Clip::image(names[p], "/x.png"));
        }
        let mut distinct = picks.clone();
        distinct.sort_unstable();
        distinct.dedup();
        let repeats = picks.len() - distinct.len();
        match builder.build() {
            Ok(_) => prop_assert_eq!(repeats, 0),
            Err(e) => {
                let dups = e.errors().iter()
                    .filter(|e| matches!(e, CompositionError::DuplicateId { .. }))
                    .count();
                prop_assert_eq!(dups, repeats);
            }
        }
    }

    // S11: a chain of relative starts resolves like the runtime:
    // start(n) = max(0, end(n-1) + offset).
    #[test]
    fn s11_chain_resolves_like_the_runtime(
        steps in prop::collection::vec((1u64..5000, -5000i64..5000), 1..20),
    ) {
        let mut builder = Composition::builder("root").duration(ms(1000));
        for (i, &(dur, off)) in steps.iter().enumerate() {
            let mut start = if i == 0 {
                Start::at(ms(0))
            } else {
                Start::after(&format!("c{}", i - 1))
            };
            start = if off >= 0 { start.plus(ms(off.unsigned_abs())) } else { start.minus(ms(off.unsigned_abs())) };
            builder = builder.clip(Clip::html(&format!("c{i}"), html! {}).duration(ms(dur)).start(start));
        }
        let comp = builder.build().expect("a chain is valid");
        let mut end: i64 = 0;
        for (i, &(dur, off)) in steps.iter().enumerate() {
            let start = if i == 0 { off.max(0) } else { (end + off).max(0) };
            let id = format!("c{i}");
            prop_assert_eq!(comp.resolved_start(&id), Some(ms(start.unsigned_abs())));
            end = start + i64::try_from(dur).expect("small");
            prop_assert_eq!(comp.resolved_end(&id), Some(ms(end.unsigned_abs())));
        }
    }

    // S6: references to earlier clips never fail. A reference from the first
    // clip to the last clip closes a loop when every clip refers back.
    #[test]
    fn s6_backward_references_are_valid_and_a_closed_ring_is_a_loop(
        targets in prop::collection::vec(any::<prop::sample::Index>(), 2..10),
    ) {
        let n = targets.len();
        let mut builder = Composition::builder("root").duration(ms(1000));
        for (i, t) in targets.iter().enumerate() {
            let start = if i == 0 { Start::default() } else { Start::after(&format!("c{}", t.index(i))) };
            builder = builder.clip(Clip::html(&format!("c{i}"), html! {}).duration(ms(100)).start(start));
        }
        prop_assert!(builder.build().is_ok());

        let mut ring = Composition::builder("root").duration(ms(1000));
        for i in 0..n {
            let prev = (i + n - 1) % n;
            ring = ring.clip(Clip::html(&format!("c{i}"), html! {}).duration(ms(100)).start(Start::after(&format!("c{prev}"))));
        }
        let err = ring.build().expect_err("a ring is a loop");
        let all: Vec<String> = (0..n).map(|i| format!("c{i}")).collect();
        prop_assert_eq!(err.errors(), &[CompositionError::ReferenceCycle { clips: all }]);
    }

    // Times always match the runtime number grammar: digits, then at most
    // three decimals, no exponent.
    #[test]
    fn times_match_the_runtime_number_grammar(nanos in 1_000_000u64..10_000_000_000_000) {
        let comp = Composition::builder("root").duration(Duration::from_nanos(nanos)).build().expect("ok");
        let html = comp.fragment().into_string();
        let re = Regex::new(r#"data-duration="(\d+(?:\.\d{1,3})?)""#).expect("regex");
        let caps = re.captures(&html).expect("duration attribute");
        let value: f64 = caps[1].parse().expect("number");
        let expected = ((u128::from(nanos) + 500_000) / 1_000_000) as f64 / 1000.0;
        prop_assert!((value - expected).abs() < 1e-9, "{} vs {}", value, expected);
        prop_assert!(!caps[1].ends_with('0') || !caps[1].contains('.'));
    }
}
