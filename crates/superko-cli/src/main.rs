//! The `superko` binary — the operator surface over the search crates.
//!
//! Three subcommands, all of them counters:
//!
//! ```text
//! superko count-games     --board MxN --rule psk|ssk --suicide forbid|remove-own
//!                         [--naive] [--threads N] [--depth-cap D]
//! superko count-positions --board MxN
//! superko graph-census    --board MxN --suicide forbid|remove-own
//! superko scc-census      --board MxN --suicide forbid|remove-own
//! ```
//!
//! Argument parsing is hand-rolled, because the workspace has no dependencies:
//! a `--flag value` or `--flag=value` pair, an unknown or missing one refused
//! with the usage text and exit status 2.
//!
//! # The body, and what is not in it
//!
//! Standard output is a results **body**: `key=value` lines, one per line, in a
//! fixed order, deterministic across runs and across thread counts.
//! `tools/verify-results.sh` diffs a body, so nothing that varies between two
//! runs of the same command may appear in it — wall time and node rate go to
//! standard error, prefixed with `#`.
//!
//! The witness header lines (`# witness`, `# commit`, `# date`, `# defs-blob`)
//! are **not** printed here. They are the maintainer's to write when a body is
//! promoted to `results/`, because they assert things about the working tree
//! that this binary cannot check.
//!
//! # Status
//!
//! Every number printed is `computed` and carries the divergences named in the
//! `divergences=` line. A count under `--suicide remove-own` has no counterpart
//! in `Defs.lean` at all.

use std::fmt::Write as _;
use std::process::ExitCode;
use std::time::Instant;

use superko_graph::census::{legal_positions, position_graph};
use superko_graph::enumerate::{self, Options};
use superko_graph::scc::situation_graph;
use superko_graph::walk;
use superko_rules::code::render;
use superko_rules::config::{Dims, Repetition, Suicide};
use superko_rules::divergence::Divergence;
use superko_rules::reference::Position;

const USAGE: &str = "\
usage: superko <command> [options]

  count-games     --board MxN --rule psk|ssk --suicide forbid|remove-own
                  [--naive] [--threads N] [--depth-cap D]
  count-positions --board MxN
  graph-census    --board MxN --suicide forbid|remove-own
  scc-census      --board MxN --suicide forbid|remove-own

Standard output is a results body of key=value lines. Timing goes to standard
error. The witness header of a promoted result is written by hand.";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(body) => {
            print!("{body}");
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("superko: {message}");
            eprintln!("{USAGE}");
            ExitCode::from(2)
        }
    }
}

/// Run a command line, returning the body to print or the message to refuse
/// it with.
fn run(args: &[String]) -> Result<String, String> {
    let (command, rest) = args.split_first().ok_or("no command given")?;
    let opts = Flags::parse(rest)?;
    match command.as_str() {
        "count-games" => count_games(&opts),
        "count-positions" => count_positions(&opts),
        "graph-census" => graph_census(&opts),
        "scc-census" => scc_census(&opts),
        other => Err(format!("unknown command {other:?}")),
    }
}

/// The flags a command line carried, unparsed beyond their spelling.
#[derive(Debug, Default)]
struct Flags {
    board: Option<String>,
    rule: Option<String>,
    suicide: Option<String>,
    threads: Option<String>,
    depth_cap: Option<String>,
    naive: bool,
}

impl Flags {
    /// Parse `--flag value` and `--flag=value` pairs, refusing anything else.
    fn parse(args: &[String]) -> Result<Self, String> {
        let mut out = Self::default();
        let mut i = 0;
        while i < args.len() {
            let arg = args[i].as_str();
            let (name, inline) = match arg.split_once('=') {
                Some((name, value)) => (name, Some(value.to_string())),
                None => (arg, None),
            };
            if name == "--naive" {
                if inline.is_some() {
                    return Err("--naive takes no value".to_string());
                }
                out.naive = true;
                i += 1;
                continue;
            }
            let slot = match name {
                "--board" => &mut out.board,
                "--rule" => &mut out.rule,
                "--suicide" => &mut out.suicide,
                "--threads" => &mut out.threads,
                "--depth-cap" => &mut out.depth_cap,
                other => return Err(format!("unknown option {other:?}")),
            };
            if slot.is_some() {
                return Err(format!("{name} given twice"));
            }
            let value = match inline {
                Some(value) => {
                    i += 1;
                    value
                }
                None => {
                    let value = args
                        .get(i + 1)
                        .ok_or_else(|| format!("{name} needs a value"))?;
                    i += 2;
                    value.clone()
                }
            };
            *slot = Some(value);
        }
        Ok(out)
    }

    fn board(&self) -> Result<Dims, String> {
        let text = self.board.as_deref().ok_or("--board is required")?;
        parse_board(text)
    }

    fn rule(&self) -> Result<Repetition, String> {
        match self.rule.as_deref().ok_or("--rule is required")? {
            "psk" => Ok(Repetition::Psk),
            "ssk" => Ok(Repetition::Ssk),
            other => Err(format!("--rule is psk or ssk, not {other:?}")),
        }
    }

    fn suicide(&self) -> Result<Suicide, String> {
        match self.suicide.as_deref().ok_or("--suicide is required")? {
            "forbid" => Ok(Suicide::Forbid),
            "remove-own" => Ok(Suicide::RemoveOwn),
            other => Err(format!("--suicide is forbid or remove-own, not {other:?}")),
        }
    }

    fn threads(&self) -> Result<usize, String> {
        let Some(text) = self.threads.as_deref() else {
            return Ok(1);
        };
        let n: usize = text
            .parse()
            .map_err(|_| format!("--threads takes a positive integer, not {text:?}"))?;
        if n == 0 {
            return Err("--threads takes a positive integer, not 0".to_string());
        }
        Ok(n)
    }

    fn depth_cap(&self) -> Result<Option<usize>, String> {
        let Some(text) = self.depth_cap.as_deref() else {
            return Ok(None);
        };
        text.parse()
            .map(Some)
            .map_err(|_| format!("--depth-cap takes a non-negative integer, not {text:?}"))
    }

    /// Refuse a flag a command does not take, rather than ignoring it.
    fn reject(&self, unwanted: &[&str]) -> Result<(), String> {
        for name in unwanted {
            let given = match *name {
                "--rule" => self.rule.is_some(),
                "--suicide" => self.suicide.is_some(),
                "--threads" => self.threads.is_some(),
                "--depth-cap" => self.depth_cap.is_some(),
                "--naive" => self.naive,
                _ => false,
            };
            if given {
                return Err(format!("{name} is not an option of this command"));
            }
        }
        Ok(())
    }
}

/// `MxN`, with both dimensions positive.
fn parse_board(text: &str) -> Result<Dims, String> {
    let (rows, cols) = text
        .split_once('x')
        .ok_or_else(|| format!("--board is MxN, not {text:?}"))?;
    let rows: usize = rows
        .parse()
        .map_err(|_| format!("--board is MxN, not {text:?}"))?;
    let cols: usize = cols
        .parse()
        .map_err(|_| format!("--board is MxN, not {text:?}"))?;
    if rows == 0 || cols == 0 {
        return Err(format!("a board dimension must be positive: {text:?}"));
    }
    Ok(Dims::new(rows, cols))
}

/// The divergences a run was under, as `slug:license` pairs in the order
/// `Divergence::ALL` fixes.
fn divergences(under: &[Divergence]) -> String {
    let mut names = Vec::new();
    for d in Divergence::ALL {
        if under.contains(&d) {
            names.push(format!(
                "{}:{}",
                d.slug(),
                d.lean_witness().unwrap_or("unlicensed")
            ));
        }
    }
    names.join(",")
}

/// `superko count-games`.
fn count_games(flags: &Flags) -> Result<String, String> {
    let dims = flags.board()?;
    let rep = flags.rule()?;
    let suicide = flags.suicide()?;
    let threads = flags.threads()?;
    let depth_cap = flags.depth_cap()?;
    if flags.naive && threads > 1 {
        return Err("--naive is single-threaded; drop --threads".to_string());
    }

    let mut under = vec![Divergence::DimsAreRuntime];
    if matches!(suicide, Suicide::RemoveOwn) {
        under.push(Divergence::SuicideRemoveOwn);
    }
    if !flags.naive {
        under.push(Divergence::RuleTableMemo);
        if matches!(rep, Repetition::Psk) {
            under.push(Divergence::PskArchiveProjection);
        }
    }

    let started = Instant::now();
    let report = if flags.naive {
        walk::count_games(dims, rep, suicide, depth_cap)
    } else {
        let opts = Options { depth_cap, threads };
        enumerate::count_games(dims, rep, suicide, opts).map_err(|e| e.to_string())?
    };
    let elapsed = started.elapsed();

    let mut body = String::new();
    let _ = writeln!(body, "board={dims}");
    let _ = writeln!(body, "rule={rep}");
    let _ = writeln!(body, "suicide={suicide}");
    let _ = writeln!(body, "divergences={}", divergences(&under));
    let _ = writeln!(body, "root={}", render(&Position::empty(dims)));
    for line in report.lines() {
        let _ = writeln!(body, "{line}");
    }
    if let Some(cap) = depth_cap {
        // A capped run is not a finished count, and a body that did not say so
        // would read as one.
        let _ = writeln!(body, "depth-cap={cap}");
        let _ = writeln!(body, "truncated={}", report.truncated);
    }

    let seconds = elapsed.as_secs_f64();
    let engine = if flags.naive { "naive" } else { "fast" };
    eprintln!(
        "# engine={engine} threads={}",
        if flags.naive { 1 } else { threads }
    );
    eprintln!("# elapsed={seconds:.3}s");
    if seconds > 0.0 {
        let rate = report.nodes as f64 / seconds;
        eprintln!("# rate={rate:.0} nodes/s");
    }
    Ok(body)
}

/// `superko count-positions`.
/// The component structure of the situation graph, under both readings of the
/// vertex set: the positions play can reach, and every coloring `Defs.lean`
/// admits.
///
/// The number the upper-bound work consumes is `outside-largest`: it bounds
/// what the forward-cone prune of `Compress.winsFor_seen_inter_cone` can
/// remove from an archive (C-45, C-46).
fn scc_census(flags: &Flags) -> Result<String, String> {
    flags.reject(&["--rule", "--threads", "--depth-cap", "--naive"])?;
    let dims = flags.board()?;
    let suicide = flags.suicide()?;
    let mut under = vec![Divergence::DimsAreRuntime];
    if matches!(suicide, Suicide::RemoveOwn) {
        under.push(Divergence::SuicideRemoveOwn);
    }

    let started = Instant::now();
    let legal = situation_graph(dims, suicide, true);
    let all = situation_graph(dims, suicide, false);
    let elapsed = started.elapsed();

    let mut body = String::new();
    let _ = writeln!(body, "board={dims}");
    let _ = writeln!(body, "suicide={suicide}");
    let _ = writeln!(body, "divergences={}", divergences(&under));
    for line in legal.lines("legal-") {
        let _ = writeln!(body, "{line}");
    }
    for line in all.lines("all-") {
        let _ = writeln!(body, "{line}");
    }
    eprintln!("# elapsed={:.3}s", elapsed.as_secs_f64());
    Ok(body)
}

fn count_positions(flags: &Flags) -> Result<String, String> {
    flags.reject(&["--rule", "--suicide", "--threads", "--depth-cap", "--naive"])?;
    let dims = flags.board()?;
    let started = Instant::now();
    let positions = legal_positions(dims);
    let elapsed = started.elapsed();

    let mut body = String::new();
    let _ = writeln!(body, "board={dims}");
    let _ = writeln!(
        body,
        "divergences={}",
        divergences(&[Divergence::DimsAreRuntime])
    );
    let _ = writeln!(body, "positions={positions}");
    eprintln!("# elapsed={:.3}s", elapsed.as_secs_f64());
    Ok(body)
}

/// `superko graph-census`.
fn graph_census(flags: &Flags) -> Result<String, String> {
    flags.reject(&["--rule", "--threads", "--depth-cap", "--naive"])?;
    let dims = flags.board()?;
    let suicide = flags.suicide()?;
    let mut under = vec![Divergence::DimsAreRuntime];
    if matches!(suicide, Suicide::RemoveOwn) {
        under.push(Divergence::SuicideRemoveOwn);
    }

    let started = Instant::now();
    let census = position_graph(dims, suicide);
    let elapsed = started.elapsed();

    let mut body = String::new();
    let _ = writeln!(body, "board={dims}");
    let _ = writeln!(body, "suicide={suicide}");
    let _ = writeln!(body, "divergences={}", divergences(&under));
    for line in census.lines() {
        let _ = writeln!(body, "{line}");
    }
    eprintln!("# elapsed={:.3}s", elapsed.as_secs_f64());
    Ok(body)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(text: &str) -> Vec<String> {
        text.split_whitespace().map(str::to_string).collect()
    }

    #[test]
    fn board_spellings() {
        assert_eq!(parse_board("2x3"), Ok(Dims::new(2, 3)));
        assert!(parse_board("2X3").is_err());
        assert!(parse_board("2").is_err());
        assert!(parse_board("0x3").is_err());
        assert!(parse_board("2x").is_err());
    }

    #[test]
    fn flags_take_both_spellings() {
        let a = Flags::parse(&args("--board 1x3 --rule psk")).unwrap();
        let b = Flags::parse(&args("--board=1x3 --rule=psk")).unwrap();
        assert_eq!(a.board, b.board);
        assert_eq!(a.rule, b.rule);
    }

    #[test]
    fn bad_command_lines_are_refused() {
        assert!(run(&args("")).is_err());
        assert!(run(&args("count-games --board 1x2 --rule psk")).is_err());
        assert!(
            run(&args(
                "count-games --board 1x2 --rule psk --suicide forbid --naive --threads 2"
            ))
            .is_err()
        );
        assert!(run(&args("count-positions --board 1x2 --rule psk")).is_err());
        assert!(run(&args("count-games --board 1x2 --rule ppk --suicide forbid")).is_err());
        assert!(
            run(&args(
                "count-games --board 1x2 --board 1x3 --rule psk --suicide forbid"
            ))
            .is_err()
        );
        assert!(run(&args("nonsense --board 1x2")).is_err());
    }

    #[test]
    fn a_body_is_the_documented_lines() {
        let body = run(&args("count-games --board 1x2 --rule psk --suicide forbid")).unwrap();
        let keys: Vec<&str> = body
            .lines()
            .map(|l| l.split_once('=').expect("a body line is key=value").0)
            .collect();
        assert_eq!(
            keys,
            vec![
                "board",
                "rule",
                "suicide",
                "divergences",
                "root",
                "games",
                "nodes",
                "max-depth",
                "max-archive",
                "refused-occupied",
                "refused-suicide",
                "refused-repetition",
            ]
        );
        assert!(body.contains("games=9"));
        assert!(body.contains("root=.."));
    }

    #[test]
    fn a_capped_body_says_so() {
        let body = run(&args(
            "count-games --board 2x2 --rule psk --suicide forbid --depth-cap 4",
        ))
        .unwrap();
        assert!(body.contains("depth-cap=4"));
        assert!(body.contains("truncated="));
    }
}
