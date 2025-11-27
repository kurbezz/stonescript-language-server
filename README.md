# StoneScript Language Server

A Language Server Protocol (LSP) implementation for the StoneScript programming language, providing intelligent code editing features for StoneScript, the scripting language used in Stone Story RPG.

**Now powered by a pure Rust nom-based parser** - no external C dependencies or build scripts required!

## Features

- **Syntax Highlighting** - Semantic tokens for enhanced code coloring
- **Code Completion** - Intelligent autocomplete for:
  - Variables and functions
  - Built-in game objects (foe, item, armor, etc.)
  - Keywords and operators
  - Abilities, locations, and game state
  - UI elements, sounds, and music
- **Diagnostics** - Real-time error detection and warnings
- **Hover Information** - Documentation and type information on hover
- **Go to Definition** - Navigate to variable and function declarations
- **Document Symbols** - Outline view of document structure
- **Signature Help** - Parameter hints for function calls
- **Code Formatting** - Automatic code formatting

## Installation

### Automatic (Recommended)

The easiest way to use the StoneScript Language Server is through the [Zed StoneScript extension](https://github.com/kurbezz/zed-stonescript):

1. Install the StoneScript extension in Zed
2. The LSP server will be automatically downloaded and installed on first use
3. No manual setup required!

The extension automatically manages LSP updates and provides pre-built binaries for:
- macOS (Apple Silicon & Intel)
- Linux (x86_64)
- Windows (x86_64)

### From Source

For development or custom builds:

```bash
# Clone the repository
git clone https://github.com/kurbezz/stonescript-language-server.git

# Build the LSP (pure Rust, no external dependencies needed!)
cd stonescript-language-server
cargo build --release
```

The compiled binary will be available at `target/release/stonescript-lsp`.

## Architecture

This LSP uses a custom **nom-based parser** instead of tree-sitter:

- **Pure Rust**: No C dependencies, faster compilation
- **Type-Safe AST**: Direct access to strongly-typed abstract syntax tree
- **Flexible**: Easy to extend and modify parser rules
- **Maintainable**: Simpler codebase without grammar compilation step

See [MIGRATION.md](MIGRATION.md) for details on the parser architecture.

**Important:** The LSP requires the tree-sitter-stonescript repository to be in the parent directory during build. Expected structure:
```
parent-dir/
├── stonescript-language-server/
└── tree-sitter-stonescript/
```

You can test your local build with:
```bash
cd stonescript-language-server
./scripts/test-build.sh
```

### Prerequisites

- Rust 1.70 or later
- Cargo
- C compiler (gcc, clang, or MSVC) - required for building tree-sitter parser

## Usage

### Command Line

```bash
stonescript-lsp
```

The language server communicates via stdin/stdout using the Language Server Protocol.

### Editor Integration

#### VSCode

Add to your `settings.json`:

```json
{
  "stonescript.languageServer.path": "/path/to/stonescript-lsp"
}
```

#### Neovim

Using `nvim-lspconfig`:

```lua
require'lspconfig'.stonescript_lsp.setup{
  cmd = { "/path/to/stonescript-lsp" },
  filetypes = { "stonescript" },
  root_dir = function(fname)
    return vim.fn.getcwd()
  end,
}
```

#### Zed (Recommended)

Install the [zed-stonescript](https://github.com/kurbezz/zed-stonescript) extension for automatic LSP installation and configuration. No manual setup required!

## Project Structure

This is a Cargo workspace with two main crates:

- **`stonescript-parser`** - Tree-sitter based parser for StoneScript
- **`stonescript-lsp`** - LSP server implementation

```
stone-script-lsp/
├── crates/
│   ├── stonescript-parser/   # Parser implementation
│   └── stonescript-lsp/       # LSP server
│       ├── src/
│       │   ├── data/          # Game data (abilities, foes, etc.)
│       │   ├── providers/     # LSP feature implementations
│       │   └── utils/         # Helper utilities
│       └── Cargo.toml
└── Cargo.toml
```

## Development

### Building

```bash
cargo build
```

### Testing

```bash
cargo test
```

### Running in Development

```bash
cargo run --bin stonescript-lsp
```

## Language Support

The LSP provides comprehensive support for StoneScript features:

### Built-in Objects
- `foe` - Enemy information
- `item`, `items` - Item management
- `armor`, `helm`, `shield` - Equipment
- `loc` - Location data
- `time`, `totaltime` - Time tracking
- `screen`, `pos` - UI positioning

### Game Data Completion
- **Abilities**: dash, smite, bardiche, etc.
- **Foes**: Poena, Nagaraja, Bolesh, etc.
- **Locations**: Rocky Plateau, Deadwood Canyon, etc.
- **UI Elements**: buffs, debuffs, conditions, etc.
- **Sounds & Music**: All game audio assets

### Type Inference
The LSP includes a basic type inference system to provide better completion suggestions and error detection.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

### Adding New Game Data

Game data is stored in `crates/stonescript-lsp/src/data/`. To add new items:

1. Edit the appropriate file (e.g., `abilities.rs`, `foes.rs`)
2. Add documentation strings for hover information
3. Rebuild the project

## License

MIT License - see LICENSE file for details

## Related Projects

- [tree-sitter-stonescript](https://github.com/kurbezz/tree-sitter-stonescript) - Tree-sitter grammar for StoneScript
- [zed-stonescript](https://github.com/kurbezz/zed-stonescript) - Zed editor extension for StoneScript

## Resources

- [Stone Story RPG](https://stonestoryrpg.com/)
- [StoneScript Documentation](https://stonestoryrpg.com/stonescript/)
- [Language Server Protocol](https://microsoft.github.io/language-server-protocol/)

## Pre-built Binaries

Pre-built binaries are automatically released for each version via GitHub Actions:
- Download from [GitHub Releases](https://github.com/kurbezz/stonescript-language-server/releases)
- Supported platforms: macOS (ARM64/x86_64), Linux (x86_64), Windows (x86_64)
- The Zed extension uses these binaries automatically

## Acknowledgments

Built with:
- [tower-lsp](https://github.com/ebkalderon/tower-lsp) - LSP framework for Rust
- [tree-sitter](https://tree-sitter.github.io/) - Incremental parsing library