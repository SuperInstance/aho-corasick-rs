# aho-corasick-rs

[![crates.io](https://img.shields.io/crates/v/aho-corasick-rs.svg)](https://crates.io/crates/aho-corasick-rs)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

Aho-Corasick multi-pattern string matching automaton in pure Rust.

## The Problem

You need to search a text for **all occurrences** of multiple patterns simultaneously. Running a separate search per pattern is O(k · n) where k is the number of patterns and n is the text length. When k is large (e.g., a dictionary of 10,000 keywords), this becomes prohibitive.

## The Insight

The Aho-Corasick algorithm builds a single automaton from all patterns that processes the text in **one pass**. It combines two ideas:

1. **Trie structure** — All patterns share a common prefix tree, so matching "he" and "hers" shares the `h → e` prefix path.
2. **Failure links** — When the current character doesn't match any child, instead of restarting from the root, jump to the longest proper suffix of the current path that is also a trie prefix. This is analogous to KMP's failure function, but generalized to multiple patterns.

With a precomputed `goto` table (a 2D array indexed by `[state][byte]`), each character in the text causes exactly one state transition — O(1) per character, regardless of pattern count.

## How It Works

**Phase 1 — Trie construction:** Insert each pattern byte-by-byte into a trie. Each terminal node records which pattern(s) end there.

**Phase 2 — Failure links (BFS):** Starting from the root's children, perform a BFS. For each node `u` with character `c`:
- If `u` has a child for `c`, set that child's failure link to `goto[fail[u]][c]`.
- If `u` has no child for `c`, set `goto[u][c] = goto[fail[u]][c]`.

This "dangling edge" resolution means every state has a defined transition for every byte — no runtime failure-link chasing.

**Phase 3 — Output propagation:** During BFS, each node inherits the output list of its failure target. This ensures all pattern matches are reported without additional follow-up at search time.

**Search:** Walk the text one byte at a time. At each position, check the current state's output list for matches.

## Usage

```rust
use aho_corasick::{AhoCorasick, Match};

let ac = AhoCorasick::new(&["he", "she", "his", "hers"]);

// Find all matches
let matches: Vec<Match> = ac.find_all(b"ahishers");
assert_eq!(matches.len(), 4);
// Match { pattern_id: 2, start: 1, end: 4 }  — "his"
// Match { pattern_id: 1, start: 3, end: 6 }  — "she"
// Match { pattern_id: 0, start: 4, end: 6 }  — "he"
// Match { pattern_id: 3, start: 4, end: 8 }  — "hers"

// Quick checks
assert!(ac.contains(b"ahishers"));
assert_eq!(ac.count(b"banana"), 0);

// Byte-string patterns (non-UTF8)
let ac_bin = AhoCorasick::from_bytes(&[b"\x00\x01", b"\xff\xfe"]);
let matches = ac_bin.find_all(&[0x00, 0x01, 0xFF, 0xFE]);

// Metadata
println!("{} patterns, {} automaton states", ac.num_patterns(), ac.num_states());
```

## Module Map

Everything lives in the crate root (`src/lib.rs`):

| Type | Description |
|---|---|
| `AhoCorasick` | The automaton. Construct with `new(&str)` or `from_bytes(&[u8])` |
| `Match` | A single occurrence: `pattern_id`, `start` (inclusive), `end` (exclusive) |

## Design Decisions

- **Full goto table.** The `goto` field is a `Vec<[usize; 256]>` — a complete transition table for every state and every byte value. This trades memory (256 × 8 bytes per state) for O(1) transitions with zero branching. For typical pattern sets (< 1000 states), this fits comfortably in L2/L3 cache.
- **Byte-oriented, not character-oriented.** Patterns and text are `&[u8]`. UTF-8 strings are accepted via `new()` but the automaton operates on raw bytes. This makes it correct for binary data and avoids UTF-8 decoding overhead.
- **Output list merging at construction.** Each node's output list includes both its own terminal patterns and those inherited from its failure link. This means `find_all` does one lookup per match — no failure-link chain walking at search time.
- **No allocator optimization.** Each node stores a `Vec<usize>` for children and a `Vec<usize>` for output. For very large pattern sets, a packed representation (e.g., using `smallvec` or arena allocation) would reduce memory. This implementation prioritizes clarity.

## Complexity

| Phase | Time | Space |
|---|---|---|
| Construction | O(m · σ) where m = total pattern length, σ = alphabet size | O(m · σ) for goto table |
| Search | O(n + z) where n = text length, z = number of matches | — |
| `contains` / `count` | O(n) / O(n + z) | — |

The construction cost is dominated by the goto table initialization. For an ASCII workload (σ = 256), each new trie node adds 2 KB to the table.

## Limitations

- **Memory usage scales with alphabet size.** The 256-wide goto table is wasteful for small alphabets (e.g., DNA with σ = 4). A hash-map or compressed transition table would be more efficient for those cases.
- **No streaming API.** `find_all` materializes all matches into a `Vec<Match>`. For very large texts with many matches, this allocates heavily. A streaming iterator would be more memory-efficient.
- **Case sensitivity is exact.** No built-in case-insensitive mode — normalize your patterns and text before searching.
- **No replacement or segmentation.** The automaton finds matches but doesn't perform replacements or split the text into matched/unmatched segments.

## Status

Published to [crates.io](https://crates.io/crates/aho-corasick-rs). Suitable for keyword filtering, log scanning, and multi-pattern search tasks. For production workloads with millions of patterns or streaming requirements, consider the `aho-corasick` crate from the `ripgrep` ecosystem, which implements packed SIMD acceleration and memory-optimized transition tables.
