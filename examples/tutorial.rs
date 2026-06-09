//! Tutorial: Pattern matching in fleet logs, agent messages, and code search
//!
//! Shows real applications of Aho-Corasick multi-pattern matching.

use aho_corasick_rs::AhoCorasick;

fn main() {
    println!("=== Aho-Corasick Tutorial ===\n");

    // === Part 1: Log analysis — find all error keywords ===
    println!("Part 1: Fleet build log scanning\n");
    
    let error_patterns = &["BUILD_FAILED", "TIMEOUT", "OOM", "PANIC", "segfault"];
    let ac = AhoCorasick::new(error_patterns);
    
    let log = b"agent-3: BUILD_FAILED after 12min | agent-7: TIMEOUT at 30min | agent-1: OK | agent-9: OOM killed";
    
    let matches = ac.find_all(log);
    println!("  Patterns: {:?}", error_patterns);
    println!("  Found {} matches in log:", matches.len());
    for m in &matches {
        let matched = std::str::from_utf8(&log[m.start..m.end]).unwrap();
        println!("    '{}' at byte {}..{} (pattern {})", matched, m.start, m.end, m.pattern);
    }
    println!();

    // === Part 2: Agent capability matching ===
    println!("Part 2: Agent capability detection\n");
    
    let capability_keywords = &["rust", "python", "midi", "tensor", "gpu", "esp32"];
    let cap_detector = AhoCorasick::new(capability_keywords);
    
    let agent_descriptions = [
        ("forgemaster", "Builds Rust crates with tensor operations and MIDI output"),
        ("oracle2", "Python analysis with GPU acceleration"),
        ("tiny-agent", "ESP32 firmware deployment"),
    ];
    
    for (name, desc) in &agent_descriptions {
        let matches = cap_detector.find_all(desc.as_bytes());
        let found: Vec<&str> = matches.iter()
            .map(|m| capability_keywords[m.pattern])
            .collect();
        println!("  {}: {:?}", name, found);
    }
    println!();

    // === Part 3: Simple presence check and count ===
    println!("Part 3: Quick checks\n");
    
    let security_patterns = &["api_key", "secret", "password", "token", "credential"];
    let scanner = AhoCorasick::new(security_patterns);
    
    let safe_code = b"let config = load_config(); // loads from env";
    let unsafe_code = b"let api_key = \"sk-abc123\"; // hardcoded secret";
    
    println!("  Safe code contains secrets: {}", scanner.contains(safe_code));
    println!("  Unsafe code contains secrets: {}", scanner.contains(unsafe_code));
    println!("  Unsafe code secret count: {}", scanner.count(unsafe_code));
    println!();

    // === Part 4: Automaton stats ===
    println!("Part 4: Automaton structure\n");
    let patterns = &["error", "warn", "info", "debug", "trace"];
    let ac = AhoCorasick::new(patterns);
    println!("  Patterns: {}", ac.num_patterns());
    println!("  States: {}", ac.num_states());
}
