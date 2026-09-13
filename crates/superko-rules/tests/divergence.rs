//! The divergence register is closed and complete.
//!
//! `tools/check-mirror.sh` holds the slugs and the source markers to the same
//! set; these tests hold `ALL` to the enum, so a variant added without being
//! listed is caught here rather than at the next witness header.

use superko_rules::divergence::Divergence;

#[test]
fn all_lists_every_variant_once() {
    let mut seen = [false; 7];
    for d in Divergence::ALL {
        let i = match d {
            Divergence::DimsAreRuntime => 0,
            Divergence::SuicideRemoveOwn => 1,
            Divergence::PskArchiveProjection => 2,
            Divergence::RuleTableMemo => 3,
            Divergence::WinnerViaFloorKomi => 4,
            Divergence::BoardSymmetry => 5,
            Divergence::ColorSwap => 6,
        };
        assert!(!seen[i], "{d} appears twice in ALL");
        seen[i] = true;
    }
    assert!(seen.iter().all(|s| *s), "ALL is missing a variant");
}

#[test]
fn slugs_are_distinct_kebab_case() {
    let mut slugs: Vec<&str> = Divergence::ALL.iter().map(|d| d.slug()).collect();
    slugs.sort_unstable();
    let distinct = slugs.len();
    slugs.dedup();
    assert_eq!(slugs.len(), distinct, "two divergences share a slug");
    for slug in &slugs {
        assert!(
            slug.chars().all(|c| c.is_ascii_lowercase() || c == '-'),
            "{slug} is not kebab-case, and the gate's marker pattern would miss it"
        );
    }
}

/// Only one departure has a Lean theorem behind it. The others are recorded as
/// unlicensed on purpose, and a run under `SuicideRemoveOwn` produces a number
/// about this crate rather than about `Defs.lean`.
#[test]
fn only_the_komi_floor_is_licensed() {
    for d in Divergence::ALL {
        match d {
            Divergence::WinnerViaFloorKomi => {
                assert_eq!(d.lean_witness(), Some("Superko.winnerZ_eq_winner"));
                assert!(d.licensed());
            }
            other => {
                assert_eq!(other.lean_witness(), None, "{other} claims a license");
                assert!(!other.licensed());
            }
        }
    }
}

#[test]
fn every_divergence_states_a_consequence() {
    for d in Divergence::ALL {
        let sentence = d.consequence();
        assert!(sentence.len() > 40, "{d} has no consequence to print");
        assert!(
            sentence.ends_with('.'),
            "{d}: the consequence is a sentence"
        );
    }
}

/// `Defs.lean` admits `m = 0`, where the board has no points; this crate
/// refuses it. That is a departure from a literal reading, so the
/// `dims-are-runtime` consequence has to name it — a witness header prints
/// that sentence and nothing else about the board's typing.
#[test]
fn the_runtime_dims_consequence_names_what_it_costs() {
    let sentence = Divergence::DimsAreRuntime.consequence();
    for phrase in ["off the board", "different boards", "zero dimension"] {
        assert!(
            sentence.contains(phrase),
            "the consequence does not mention {phrase:?}: {sentence}"
        );
    }
}

#[test]
#[should_panic(expected = "a board dimension must be positive")]
fn a_zero_dimension_is_refused() {
    let _ = superko_rules::config::Dims::new(0, 3);
}
