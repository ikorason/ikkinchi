# Changelog

## [0.3.0] - 2026-03-24

### Added
- `--semantic` flag for `search` command — runs Ollama-powered semantic search independently
- New `src/semantic.rs` module owning the semantic search algorithm (`semantic_search()`)

### Changed
- Search is now **fuzzy by default** (no Ollama required) — `ikkinchi search "query"`
- Semantic search runs only when `--semantic` is passed — `ikkinchi search --semantic "query"`
- Removed hybrid scoring (0.6 × semantic + 0.4 × fuzzy) — modes are now fully isolated

### Fixed
- Fuzzy search no longer errors or degrades when Ollama is unavailable

## [0.2.0] - 2026-03-12

### Added
- Manual tagging (`ikkinchi tag add/remove`)
- Interactive TUI (`ikkinchi tui`) with fuzzy filter and semantic search modes
- Import from `.md`/`.txt` files and directories
- Export to JSON and Markdown
- Stats command
