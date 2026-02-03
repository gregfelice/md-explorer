# md-explorer

A Midnight Commander-inspired TUI for browsing markdown files.

![Rust](https://img.shields.io/badge/rust-stable-orange)
![License](https://img.shields.io/badge/license-MIT-blue)

## Features

- **File tree navigation** - Browse `~/operations` and `~/development` directories
- **Markdown preview** - Syntax highlighting for headers, code blocks, lists, inline formatting
- **Fuzzy search** - Quick filtering with `/`
- **Collapsible directories** - State persisted between sessions
- **Editor integration** - Open files in `$EDITOR` with Enter
- **CLAUDE.md filter** - Toggle to show only CLAUDE.md files with `c`
- **Respects .gitignore** - Skips node_modules, target, venv, etc.

## Installation

### From source (recommended)

```bash
cargo install --path .
```

### Using just

```bash
just install
```

## Usage

```bash
md-explorer
```

## Keybindings

| Key | Action |
|-----|--------|
| `↑/k` | Move selection up |
| `↓/j` | Move selection down |
| `Tab` | Expand/collapse directory |
| `Enter` | Open file in $EDITOR |
| `Space` | Toggle focus (tree/preview) |
| `/` | Start search/filter |
| `Esc` | Clear search / exit mode |
| `.` | Toggle empty directories |
| `c` | Toggle CLAUDE.md only |
| `r` | Refresh file list |
| `?` | Show help |
| `q` | Quit |

### In Preview Pane

| Key | Action |
|-----|--------|
| `↑/k` | Scroll up |
| `↓/j` | Scroll down |
| `Space` | Return to tree |

## Configuration

State is persisted to `~/.local/state/md-explorer/state`:
- Collapsed directory state
- Show empty directories toggle
- CLAUDE.md filter toggle

## Scanned Directories

By default, md-explorer scans:
- `~/operations`
- `~/development`

## Building

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Run tests
cargo test
```

## License

MIT - see [LICENSE](LICENSE)
