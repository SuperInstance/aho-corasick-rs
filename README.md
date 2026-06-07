# aho-corasick-rs

Aho-Corasick multi-pattern string matching automaton. Pure `std`, no dependencies.

## Features

- Trie + BFS failure-link construction
- `find_all(text)` — all matches with pattern id and `[start, end)` span
- `contains(text)` / `count(text)` — fast existence / count queries
- Works on arbitrary byte strings
