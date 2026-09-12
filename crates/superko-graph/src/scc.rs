//! The strongly connected components of the situation graph.
//!
//! # What this is for
//!
//! `Compress.winsFor_seen_inter_cone` (C-42, `proved`) says the archive may be
//! pruned to the **forward cone** — the situations still reachable from the
//! current one by moves the board permits, with the repetition rule out of the
//! way — without changing who wins. Whether that prune is a *compression* is
//! not a question Lean answers: it depends on how large the cone is, which is a
//! fact about Go and not about the definitions.
//!
//! This module measures it. The cone of `s` is the union of the strongly
//! connected components reachable from `s` in the condensation, so the census
//! below settles the question for every situation of a board at once.
//!
//! # The graph
//!
//! Vertices are situations: a position with a player to move, so `2 · 3^(m·n)`
//! of them, which is `Superko.card_situation`. Edges are
//! `Compress.SitStep` — a pass, always; or a play at a point the board permits
//! — and **carry no repetition rule at all**, which is what makes the cone
//! rule-independent and this census one number rather than four.
//!
//! # What the numbers say
//!
//! On every board censused with `3 ≤ m·n ≤ 12` the legal part of the graph has
//! exactly **two** components: the empty board's pass 2-cycle, and everything
//! else. The condensation is two deep. So the cone of any non-empty legal
//! situation is the whole of the giant component, and the prune of C-42 removes
//! at most the two empty situations from an archive that holds up to
//! `2 · 3^(m·n)` of them (C-45, C-46, both `computed`).
//!
//! 1×2 is the exception and is kept: there the giant component splits, because
//! neither of the two ko-shaped 2-cycles reaches the other. 1×1 has one
//! component, since no stone can ever be played.
//!
//! Illegal positions — those carrying a chain with no liberty, which
//! `Defs.lean`'s `Position` admits and play never produces — are each a source,
//! so the full census over all `3^(m·n)` colorings adds them as trivial
//! components above the legal part and changes nothing below it.
//!
//! # Not trusted
//!
//! A number here is `computed`. What it bears on is the *usefulness* of a
//! proved theorem, not its truth: C-42 holds whatever this module prints.

use superko_rules::code::{PosCode, code_space, decode, encode};
use superko_rules::config::{Dims, Suicide};
use superko_rules::reference::{Color, Position, playable_at, resolve_under};

use crate::census::is_legal;

/// The component structure of one situation graph.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SccCensus {
    /// Vertices: `2 · 3^(m·n)` over all colorings, `2 · L(m,n)` over the legal
    /// ones.
    pub situations: u64,
    /// Edges, counted per distinct `(from, to)` pair. The pass edge out of
    /// every situation is one of them.
    pub edges: u64,
    /// Strongly connected components.
    pub components: u64,
    /// The size of the largest.
    pub largest: u64,
    /// Components of one vertex.
    pub trivial: u64,
    /// The size of the component holding the empty board — 2 wherever a pass
    /// is the only move out of it and nothing plays back into it.
    pub empty_component: u64,
    /// Situations outside the largest component. This is the number that
    /// bounds what the cone prune can remove.
    pub outside_largest: u64,
    /// The longest path in the condensation, in components.
    pub depth: u64,
}

impl SccCensus {
    /// The body lines a results file carries, in a fixed order.
    #[must_use]
    pub fn lines(&self, prefix: &str) -> Vec<String> {
        vec![
            format!("{prefix}situations={}", self.situations),
            format!("{prefix}edges={}", self.edges),
            format!("{prefix}components={}", self.components),
            format!("{prefix}largest={}", self.largest),
            format!("{prefix}trivial={}", self.trivial),
            format!("{prefix}empty-component={}", self.empty_component),
            format!("{prefix}outside-largest={}", self.outside_largest),
            format!("{prefix}depth={}", self.depth),
        ]
    }
}

/// A situation-graph vertex set: the positions, and the lookup from a code.
struct Vertices {
    boards: Vec<Position>,
    /// `index[code]` is the vertex block of that position, or `u32::MAX`.
    index: Vec<u32>,
}

impl Vertices {
    fn build(dims: Dims, legal_only: bool) -> Self {
        let mut boards = Vec::new();
        let mut index = vec![u32::MAX; code_space(dims) as usize];
        for code in 0..code_space(dims) {
            let b = decode(dims, PosCode(code));
            if legal_only && !is_legal(&b) {
                continue;
            }
            index[code as usize] = u32::try_from(boards.len()).expect("a vertex index fits a u32");
            boards.push(b);
        }
        Self { boards, index }
    }

    fn len(&self) -> usize {
        self.boards.len()
    }

    /// Vertex id of `(board i, to move c)`. Black is even, White is odd.
    fn vertex(i: usize, c: Color) -> u32 {
        let side = match c {
            Color::Black => 0,
            Color::White => 1,
        };
        u32::try_from(i * 2 + side).expect("a vertex id fits a u32")
    }
}

/// Successors of every vertex, in compressed-sparse-row form.
struct Csr {
    offsets: Vec<u32>,
    targets: Vec<u32>,
}

impl Csr {
    fn succ(&self, v: u32) -> &[u32] {
        let lo = self.offsets[v as usize] as usize;
        let hi = self.offsets[v as usize + 1] as usize;
        &self.targets[lo..hi]
    }
}

fn build_edges(dims: Dims, suicide: Suicide, vs: &Vertices) -> Csr {
    let n = vs.len() * 2;
    let mut offsets = Vec::with_capacity(n + 1);
    let mut targets: Vec<u32> = Vec::new();
    let mut row: Vec<u32> = Vec::new();
    for v in 0..n {
        offsets.push(u32::try_from(targets.len()).expect("an edge offset fits a u32"));
        let i = v / 2;
        let c = if v % 2 == 0 {
            Color::Black
        } else {
            Color::White
        };
        let b = &vs.boards[i];
        row.clear();
        // The pass edge: `SSK _ .pass = True`, so it is unconditional.
        row.push(Vertices::vertex(i, c.other()));
        for p in dims.points() {
            if !playable_at(b, c, p, suicide) {
                continue;
            }
            let q = resolve_under(b, c, p, suicide);
            let j = vs.index[encode(&q).0 as usize];
            // A play from a legal position lands on a legal position, so this
            // lookup misses only when the vertex set was restricted and the
            // source was itself illegal.
            if j == u32::MAX {
                continue;
            }
            row.push(Vertices::vertex(j as usize, c.other()));
        }
        row.sort_unstable();
        row.dedup();
        targets.extend_from_slice(&row);
    }
    offsets.push(u32::try_from(targets.len()).expect("an edge offset fits a u32"));
    Csr { offsets, targets }
}

/// Tarjan's algorithm, iterative because the graph is millions of vertices
/// deep. Components come out in reverse topological order, which the depth
/// pass below relies on.
fn tarjan(n: usize, csr: &Csr) -> Vec<u32> {
    const UNVISITED: u32 = u32::MAX;
    let mut index = vec![UNVISITED; n];
    let mut low = vec![0u32; n];
    let mut on_stack = vec![false; n];
    let mut component = vec![UNVISITED; n];
    let mut stack: Vec<u32> = Vec::new();
    let mut work: Vec<(u32, u32)> = Vec::new();
    let mut counter: u32 = 0;
    let mut components: u32 = 0;

    for root in 0..n {
        let root = u32::try_from(root).expect("a vertex id fits a u32");
        if index[root as usize] != UNVISITED {
            continue;
        }
        work.push((root, 0));
        while let Some(&mut (v, ref mut pi)) = work.last_mut() {
            if *pi == 0 {
                index[v as usize] = counter;
                low[v as usize] = counter;
                counter += 1;
                stack.push(v);
                on_stack[v as usize] = true;
            }
            let succ = csr.succ(v);
            let mut descended = false;
            while (*pi as usize) < succ.len() {
                let w = succ[*pi as usize];
                *pi += 1;
                if index[w as usize] == UNVISITED {
                    work.push((w, 0));
                    descended = true;
                    break;
                } else if on_stack[w as usize] {
                    low[v as usize] = low[v as usize].min(index[w as usize]);
                }
            }
            if descended {
                continue;
            }
            if low[v as usize] == index[v as usize] {
                loop {
                    let w = stack.pop().expect("the Tarjan stack holds the component");
                    on_stack[w as usize] = false;
                    component[w as usize] = components;
                    if w == v {
                        break;
                    }
                }
                components += 1;
            }
            work.pop();
            if let Some(&(u, _)) = work.last() {
                low[u as usize] = low[u as usize].min(low[v as usize]);
            }
        }
    }
    component
}

/// Census the situation graph of a board.
///
/// `legal_only` restricts the vertex set to positions no chain of which lacks a
/// liberty — the positions play can reach. With it false the census runs over
/// every coloring, which is `Defs.lean`'s `Position m n`.
///
/// # Panics
///
/// Panics when the board is too large for a position code (`m · n > 20`).
#[must_use]
pub fn situation_graph(dims: Dims, suicide: Suicide, legal_only: bool) -> SccCensus {
    let vs = Vertices::build(dims, legal_only);
    let csr = build_edges(dims, suicide, &vs);
    let n = vs.len() * 2;
    let component = tarjan(n, &csr);
    let count = component
        .iter()
        .copied()
        .max()
        .map_or(0, |c| c as usize + 1);

    let mut sizes = vec![0u64; count];
    for &c in &component {
        sizes[c as usize] += 1;
    }

    let empty = Position::empty(dims);
    let empty_vertex = Vertices::vertex(vs.index[encode(&empty).0 as usize] as usize, Color::Black);
    let empty_component = sizes[component[empty_vertex as usize] as usize];

    let largest = sizes.iter().copied().max().unwrap_or(0);
    let trivial = sizes.iter().filter(|&&s| s == 1).count() as u64;

    // Longest path in the condensation. Tarjan numbers a component only after
    // every component it reaches, so an edge `cv -> cw` has `cw < cv` and one
    // sweep over the condensation edges in increasing `cv` gets each `depth[cw]`
    // already final.
    let mut cond: Vec<(u32, u32)> = Vec::new();
    for v in 0..n {
        let v = u32::try_from(v).expect("a vertex id fits a u32");
        for &w in csr.succ(v) {
            let (cv, cw) = (component[v as usize], component[w as usize]);
            if cv != cw {
                cond.push((cv, cw));
            }
        }
    }
    cond.sort_unstable();
    cond.dedup();
    let mut depth = vec![1u64; count];
    for &(cv, cw) in &cond {
        depth[cv as usize] = depth[cv as usize].max(depth[cw as usize] + 1);
    }

    SccCensus {
        situations: n as u64,
        edges: csr.targets.len() as u64,
        components: count as u64,
        largest,
        trivial,
        empty_component,
        outside_largest: n as u64 - largest,
        depth: depth.iter().copied().max().unwrap_or(0),
    }
}
