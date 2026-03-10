# ikkinchi

> **Status: In Development**

Your second brain — a zero-friction CLI and TUI for capturing and retrieving thoughts.

```
ikkinchi add "event sourcing could solve our audit trail problem"
ikkinchi search "audit"
```

Thoughts are stored as plain markdown files in `~/.ikkinchi/`. Search is semantic via embeddings (Ollama by default) + fuzzy matching. No folders, no tags.

## Install

```bash
cargo install ikkinchi
```

## Usage

```
ikkinchi add <text>       Capture a thought
ikkinchi search <query>   Semantic + fuzzy search
ikkinchi list             Browse recent memories
ikkinchi tui              Interactive TUI
```

Run `ikkinchi --help` for all commands.
