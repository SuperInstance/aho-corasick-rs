//! # Aho-Corasick Tutorial
//!
//! A progressive walkthrough of the `aho_corasick` crate — a zero-dependency
//! implementation of the Aho-Corasick multi-pattern string matching automaton.
//!
//! Run individual lessons:
//!
//!     cargo run --example tutorial
//!
//! Each lesson is self-contained and prints results to stdout.

use aho_corasick::{AhoCorasick, Match};

fn main() {
    println!("=== Aho-Corasick Tutorial ===\n");

    lesson_1_basic_search();
    lesson_2_match_positions();
    lesson_3_contains_and_count();
    lesson_4_byte_patterns();
    lesson_5_overlapping_patterns();
    lesson_6_real_world_keyword_filter();
    lesson_7_introspection();

    println!("\n✅ All lessons complete!");
}

// ── Lesson 1: Basic Search ────────────────────────────────────────────────────
//
// The classic example from the Aho-Corasick paper. Build an automaton from
// multiple patterns, then search a single text in one pass.

fn lesson_1_basic_search() {
    println!("--- Lesson 1: Basic Multi-Pattern Search ---");

    // Build an automaton from four string patterns.
    let ac = AhoCorasick::new(&["he", "she", "his", "hers"]);

    // Search a byte string — returns every occurrence of every pattern.
    let matches: Vec<Match> = ac.find_all(b"ahishers");

    println!("  Patterns:  [\"he\", \"she\", \"his\", \"hers\"]");
    println!("  Text:      \"ahishers\"");
    println!("  Matches:   {}", matches.len());

    for m in &matches {
        let pattern = match m.pattern_id {
            0 => "he",
            1 => "she",
            2 => "his",
            3 => "hers",
            _ => unreachable!(),
        };
        println!("    pattern \"{}\" at byte range {}..{}", pattern, m.start, m.end);
    }

    assert_eq!(matches.len(), 4);
    println!();
}

// ── Lesson 2: Understanding Match Positions ───────────────────────────────────
//
// Each `Match` carries three fields:
//   • pattern_id — index of the matched pattern (insertion order)
//   • start      — inclusive start byte offset in the text
//   • end        — exclusive end byte offset in the text
//
// You can use start..end to slice the original text and recover the matched
// substring.

fn lesson_2_match_positions() {
    println!("--- Lesson 2: Match Positions ---");

    let ac = AhoCorasick::new(&["cat", "dog", "bird"]);
    let text = b"the cat chased the bird near the dog";

    let matches = ac.find_all(text);

    for m in &matches {
        let matched_bytes = &text[m.start..m.end];
        let matched_str = std::str::from_utf8(matched_bytes).unwrap();
        println!(
            "  pattern_id={}  range={}..{}  matched=\"{}\"",
            m.pattern_id, m.start, m.end, matched_str
        );
    }

    // Verify we can recover exact substrings
    assert_eq!(&text[matches[0].start..matches[0].end], b"cat");
    assert_eq!(&text[matches[1].start..matches[1].end], b"bird");
    assert_eq!(&text[matches[2].start..matches[2].end], b"dog");

    println!();
}

// ── Lesson 3: Quick Checks — contains() and count() ──────────────────────────
//
// When you don't need exact positions, `contains()` is an early-exit boolean
// check and `count()` gives the total number of matches.

fn lesson_3_contains_and_count() {
    println!("--- Lesson 3: contains() and count() ---");

    let ac = AhoCorasick::new(&["error", "warning", "fatal"]);

    let log_line = b"2025-01-15 ERROR: connection timeout";

    // contains() returns true as soon as it finds any match (O(n) worst case)
    println!("  Log contains error keyword: {}", ac.contains(log_line));

    // count() returns total occurrences of all patterns
    let spam_text = b"warning: low disk; error: file not found; fatal: out of memory";
    println!("  Total keyword hits in log:  {}", ac.count(spam_text));

    // Text with no matches
    let clean = b"2025-01-15 INFO: startup complete";
    println!("  Clean log has keywords:      {}", ac.contains(clean));
    assert!(!ac.contains(clean));

    println!();
}

// ── Lesson 4: Byte Patterns with from_bytes() ────────────────────────────────
//
// `from_bytes()` accepts `&[&[u8]]` — useful for non-UTF-8 data like binary
// protocols, network packets, or raw byte sequences.

fn lesson_4_byte_patterns() {
    println!("--- Lesson 4: Byte Patterns (from_bytes) ---");

    // Search for binary magic bytes: PNG header, JPEG header, GIF header
    let ac = AhoCorasick::from_bytes(&[
        b"\x89PNG\r\n\x1a\n",     // PNG signature
        b"\xFF\xD8\xFF",          // JPEG signature
        b"GIF89a",                 // GIF89a signature
    ]);

    // A fake file that starts with PNG header, followed by JPEG magic mid-stream
    let mut data = b"\x89PNG\r\n\x1a\n".to_vec();
    data.extend_from_slice(b"...image data...");
    data.extend_from_slice(b"\xFF\xD8\xFF");
    data.extend_from_slice(b"...more data...");

    let matches = ac.find_all(&data);
    println!("  Found {} binary signatures in {} bytes", matches.len(), data.len());

    for m in &matches {
        let sig_name = match m.pattern_id {
            0 => "PNG",
            1 => "JPEG",
            2 => "GIF89a",
            _ => unreachable!(),
        };
        println!("    {} signature at byte offset {}", sig_name, m.start);
    }

    assert_eq!(matches.len(), 2); // PNG at 0, JPEG mid-stream
    println!();
}

// ── Lesson 5: Overlapping Patterns ────────────────────────────────────────────
//
// Aho-Corasick naturally handles overlapping patterns. When one pattern is a
// prefix or suffix of another, both are reported independently.

fn lesson_5_overlapping_patterns() {
    println!("--- Lesson 5: Overlapping Patterns ---");

    // "a" is a prefix of "ab", which is a prefix of "abc"
    let ac = AhoCorasick::new(&["a", "ab", "abc"]);

    let matches = ac.find_all(b"abc");
    println!("  Patterns: [\"a\", \"ab\", \"abc\"]");
    println!("  Text:     \"abc\"");
    println!("  All {} overlapping matches:", matches.len());

    for m in &matches {
        let pattern = ["a", "ab", "abc"][m.pattern_id];
        println!("    \"{}\" at {}..{}", pattern, m.start, m.end);
    }

    assert_eq!(matches.len(), 3); // all three patterns match at position 0

    // Overlapping with repeated text
    let ac2 = AhoCorasick::new(&["aa", "aaa"]);
    let m2 = ac2.find_all(b"aaaa");
    println!("\n  Patterns: [\"aa\", \"aaa\"]");
    println!("  Text:     \"aaaa\"");
    println!("  Matches:  {} (all overlapping occurrences)", m2.len());
    assert!(m2.len() >= 3);

    println!();
}

// ── Lesson 6: Real-World Keyword Filter ───────────────────────────────────────
//
// Build a simple content moderation filter that checks user input against a
// blocklist of forbidden words. This demonstrates a practical use case.

fn lesson_6_real_world_keyword_filter() {
    println!("--- Lesson 6: Keyword Content Filter ---");

    let blocked = AhoCorasick::new(&["spam", "scam", "phishing", "malware", "exploit"]);

    let messages = [
        "Hello, how are you today?",
        "Click here for a free prize — this is not spam!",
        "Your account has a phishing alert — verify now",
        "The quarterly report is ready for review",
    ];

    for msg in &messages {
        let flagged = blocked.contains(msg.as_bytes());
        let hits = blocked.count(msg.as_bytes());
        let status = if flagged { "⛔ BLOCKED" } else { "✅ CLEAN" };
        println!("  {} ({}) — \"{}\"", status, hits, msg);
    }

    // Count total blocked words across all messages
    let total_hits: usize = messages
        .iter()
        .map(|m| blocked.count(m.as_bytes()))
        .sum();
    println!("  Total blocked-word hits: {}", total_hits);

    println!();
}

// ── Lesson 7: Automaton Introspection ─────────────────────────────────────────
//
// `num_patterns()` and `num_states()` let you inspect the automaton's size.
// Useful for benchmarking or understanding memory footprint.

fn lesson_7_introspection() {
    println!("--- Lesson 7: Automaton Introspection ---");

    let ac = AhoCorasick::new(&["he", "she", "his", "hers"]);

    println!("  Patterns:  {}", ac.num_patterns());
    println!("  States:    {} (trie nodes including root)", ac.num_states());

    // With longer/more patterns the automaton grows
    let big = AhoCorasick::new(&[
        "hello", "world", "hell", "he", "she", "hers", "his", "him", "her",
    ]);
    println!("\n  Larger automaton:");
    println!("    Patterns: {}", big.num_patterns());
    println!("    States:   {}", big.num_states());

    // Empty automaton
    let empty = AhoCorasick::new(&[]);
    println!("\n  Empty automaton: {} patterns, {} states",
        empty.num_patterns(), empty.num_states());

    assert_eq!(empty.num_patterns(), 0);
    assert!(empty.find_all(b"anything").is_empty());

    println!();
}
