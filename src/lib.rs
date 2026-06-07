//! Aho-Corasick multi-pattern string matching automaton.
//!
//! Constructs a trie over the given pattern set, then adds failure links
//! (BFS) to enable O(n + m + z) searching where n is the text length,
//! m is the total pattern length, and z is the number of matches.
//!
//! # Examples
//!
//! ```
//! use aho_corasick::{AhoCorasick, Match};
//!
//! let ac = AhoCorasick::new(&["he", "she", "his", "hers"]);
//! let matches = ac.find_all(b"ahishers");
//! assert_eq!(matches.len(), 4);
//! ```

use std::collections::VecDeque;

// ── node ──────────────────────────────────────────────────────────────────────

const ALPHA: usize = 256;

struct Node {
    children: Vec<usize>,
    fail: usize,
    output: Vec<usize>,
}

impl Node {
    fn new() -> Self {
        Self { children: vec![usize::MAX; ALPHA], fail: 0, output: vec![] }
    }
}

// ── Match ─────────────────────────────────────────────────────────────────────

/// A pattern occurrence found in the text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Match {
    /// Index of the matched pattern (order it was inserted).
    pub pattern_id: usize,
    /// Start position in the text (inclusive).
    pub start: usize,
    /// End position in the text (exclusive).
    pub end: usize,
}

// ── AhoCorasick ───────────────────────────────────────────────────────────────

/// Aho-Corasick automaton for multi-pattern searching.
pub struct AhoCorasick {
    nodes: Vec<Node>,
    patterns: Vec<Vec<u8>>,
    goto: Vec<[usize; ALPHA]>,
}

impl AhoCorasick {
    /// Build the automaton from a list of patterns.
    pub fn new(patterns: &[&str]) -> Self {
        let pats: Vec<Vec<u8>> = patterns.iter().map(|p| p.as_bytes().to_vec()).collect();
        Self::from_bytes(&pats.iter().map(|v| v.as_slice()).collect::<Vec<_>>())
    }

    /// Build the automaton from byte-string patterns.
    pub fn from_bytes(patterns: &[&[u8]]) -> Self {
        let pats: Vec<Vec<u8>> = patterns.iter().map(|p| p.to_vec()).collect();
        let mut nodes: Vec<Node> = vec![Node::new()];

        // Phase 1: build trie
        for (pid, pat) in pats.iter().enumerate() {
            let mut cur = 0usize;
            for &b in pat {
                let b = b as usize;
                if nodes[cur].children[b] == usize::MAX {
                    nodes[cur].children[b] = nodes.len();
                    nodes.push(Node::new());
                }
                cur = nodes[cur].children[b];
            }
            nodes[cur].output.push(pid);
        }

        // Phase 2: build failure links and goto table with BFS
        let n = nodes.len();
        let mut goto = vec![[0usize; ALPHA]; n];
        let mut queue = VecDeque::new();

        // Root's children: collect first to avoid split borrows.
        let root_children: Vec<usize> = nodes[0].children.clone();
        for (b, child) in root_children.into_iter().enumerate() {
            if child == usize::MAX {
                goto[0][b] = 0; // self-loop at root
            } else {
                goto[0][b] = child;
                nodes[child].fail = 0;
                queue.push_back(child);
            }
        }

        while let Some(u) = queue.pop_front() {
            // Merge output of fail link
            let fail_u = nodes[u].fail;
            let extra: Vec<usize> = nodes[fail_u].output.clone();
            nodes[u].output.extend(extra);

            let u_children: Vec<usize> = nodes[u].children.clone();
            for (b, child) in u_children.into_iter().enumerate() {
                if child == usize::MAX {
                    // No child: follow fail link's goto
                    goto[u][b] = goto[nodes[u].fail][b];
                } else {
                    nodes[child].fail = goto[nodes[u].fail][b];
                    goto[u][b] = child;
                    queue.push_back(child);
                }
            }
        }

        Self { nodes, patterns: pats, goto }
    }

    /// Find all pattern occurrences in `text`.
    pub fn find_all(&self, text: &[u8]) -> Vec<Match> {
        let mut state = 0usize;
        let mut matches = Vec::new();
        for (i, &b) in text.iter().enumerate() {
            state = self.goto[state][b as usize];
            for &pid in &self.nodes[state].output {
                let pat_len = self.patterns[pid].len();
                matches.push(Match {
                    pattern_id: pid,
                    start: i + 1 - pat_len,
                    end: i + 1,
                });
            }
        }
        matches
    }

    /// Returns true if `text` contains any of the patterns.
    pub fn contains(&self, text: &[u8]) -> bool {
        let mut state = 0usize;
        for &b in text {
            state = self.goto[state][b as usize];
            if !self.nodes[state].output.is_empty() {
                return true;
            }
        }
        false
    }

    /// Count total occurrences of all patterns in `text`.
    pub fn count(&self, text: &[u8]) -> usize {
        self.find_all(text).len()
    }

    /// Returns the number of patterns.
    pub fn num_patterns(&self) -> usize {
        self.patterns.len()
    }

    /// Returns the number of trie nodes (automaton states).
    pub fn num_states(&self) -> usize {
        self.nodes.len()
    }
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn find_sorted(ac: &AhoCorasick, text: &[u8]) -> Vec<(usize, usize, usize)> {
        let mut v: Vec<_> =
            ac.find_all(text).into_iter().map(|m| (m.start, m.end, m.pattern_id)).collect();
        v.sort();
        v
    }

    // ── basic search ─────────────────────────────────────────────────────────

    #[test]
    fn classic_ahishers() {
        let ac = AhoCorasick::new(&["he", "she", "his", "hers"]);
        let matches = ac.find_all(b"ahishers");
        assert_eq!(matches.len(), 4);
    }

    #[test]
    fn ahishers_positions() {
        let ac = AhoCorasick::new(&["he", "she", "his", "hers"]);
        let m = find_sorted(&ac, b"ahishers");
        // "his" at 1, "he" at 4, "she" at 3, "hers" at 4
        assert!(m.contains(&(1, 4, 2))); // "his"
        assert!(m.contains(&(3, 6, 1))); // "she"
        assert!(m.contains(&(4, 6, 0))); // "he"
        assert!(m.contains(&(4, 8, 3))); // "hers"
    }

    #[test]
    fn no_match() {
        let ac = AhoCorasick::new(&["xyz", "foo"]);
        assert_eq!(ac.find_all(b"hello world"), vec![]);
    }

    #[test]
    fn single_pattern() {
        let ac = AhoCorasick::new(&["ab"]);
        let m = ac.find_all(b"ababab");
        assert_eq!(m.len(), 3);
        assert_eq!(m[0].start, 0);
        assert_eq!(m[1].start, 2);
        assert_eq!(m[2].start, 4);
    }

    #[test]
    fn overlapping_patterns() {
        let ac = AhoCorasick::new(&["aa", "aaa"]);
        let m = ac.find_all(b"aaaa");
        assert_eq!(ac.count(b"aaaa"), m.len());
        assert!(m.len() >= 3);
    }

    #[test]
    fn pattern_is_prefix_of_another() {
        let ac = AhoCorasick::new(&["a", "ab", "abc"]);
        let m = ac.find_all(b"abc");
        assert_eq!(m.len(), 3);
    }

    #[test]
    fn pattern_is_suffix_of_another() {
        let ac = AhoCorasick::new(&["abc", "bc", "c"]);
        let m = ac.find_all(b"abc");
        assert_eq!(m.len(), 3);
    }

    // ── empty and edge cases ─────────────────────────────────────────────────

    #[test]
    fn empty_text() {
        let ac = AhoCorasick::new(&["hello"]);
        assert_eq!(ac.find_all(b""), vec![]);
    }

    #[test]
    fn empty_patterns_list() {
        let ac = AhoCorasick::new(&[]);
        assert_eq!(ac.find_all(b"hello"), vec![]);
        assert_eq!(ac.num_patterns(), 0);
    }

    #[test]
    fn single_char_patterns() {
        let ac = AhoCorasick::new(&["a", "b", "c"]);
        let m = ac.find_all(b"abc");
        assert_eq!(m.len(), 3);
    }

    #[test]
    fn repeated_pattern() {
        let ac = AhoCorasick::new(&["aa"]);
        let m = ac.find_all(b"aaaa");
        assert_eq!(m.len(), 3);
    }

    // ── contains / count ─────────────────────────────────────────────────────

    #[test]
    fn contains_true() {
        let ac = AhoCorasick::new(&["world"]);
        assert!(ac.contains(b"hello world"));
    }

    #[test]
    fn contains_false() {
        let ac = AhoCorasick::new(&["xyz"]);
        assert!(!ac.contains(b"hello world"));
    }

    #[test]
    fn count_multiple() {
        let ac = AhoCorasick::new(&["an", "ban"]);
        // "banana" → "an"@1, "ban"@0, "an"@3
        assert_eq!(ac.count(b"banana"), 3);
    }

    // ── metadata ─────────────────────────────────────────────────────────────

    #[test]
    fn num_patterns() {
        let ac = AhoCorasick::new(&["a", "bb", "ccc"]);
        assert_eq!(ac.num_patterns(), 3);
    }

    #[test]
    fn num_states_single() {
        let ac = AhoCorasick::new(&["abc"]);
        assert_eq!(ac.num_states(), 4); // root + 3 nodes
    }

    // ── byte patterns ────────────────────────────────────────────────────────

    #[test]
    fn from_bytes() {
        let ac = AhoCorasick::from_bytes(&[b"foo", b"bar"]);
        assert_eq!(ac.count(b"foobar"), 2);
    }

    #[test]
    fn find_all_returns_correct_span() {
        let ac = AhoCorasick::new(&["cat"]);
        let m = ac.find_all(b"concatenate");
        assert!(!m.is_empty());
        let first = &m[0];
        assert_eq!(&b"concatenate"[first.start..first.end], b"cat");
    }

    #[test]
    fn long_text_single_pattern() {
        let ac = AhoCorasick::new(&["ab"]);
        let text = b"abababababababab";
        assert_eq!(ac.count(text), 8);
    }

    #[test]
    fn pattern_id_ordering() {
        let ac = AhoCorasick::new(&["z", "y", "x"]);
        let m = ac.find_all(b"xyz");
        // x→pid2, y→pid1, z→pid0
        let ids: Vec<usize> = m.iter().map(|m| m.pattern_id).collect();
        assert!(ids.contains(&0));
        assert!(ids.contains(&1));
        assert!(ids.contains(&2));
    }
}
