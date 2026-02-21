# halfguru - AI Agent Guide

## Project Overview
Rust-based GitHub statistics generator that creates terminal-style SVG profile cards. Fetches real-time GitHub stats for a user and generates dark/light mode SVG images with neofetch-inspired formatting for README embedding.

## Tech Stack
- **Language:** Rust
- **Runtime:** Tokio
- **HTTP Client:** reqwest
- **Serialization:** serde/serde_json/toml
- **Date/Time:** chrono
- **Error Handling:** anyhow

## Commands

### Build & Run
```bash
cargo build --release    # Build release binary
cargo run                # Build and run (generates dark_mode.svg and light_mode.svg)
```

### Testing
```bash
cargo test                          # Run all tests
cargo test test_plural              # Run single test by name
cargo test age_string               # Run tests matching pattern
cargo test -- --nocapture           # Run tests with stdout visible
```

### Quality (run after edits)
```bash
cargo fmt                           # Format code
cargo clippy                        # Run linter
cargo clippy -- -D warnings         # Clippy with warnings as errors
```

## Architecture

### Data Flow
1. `main.rs` loads TOML config from `config/profile.toml`
2. Creates `GithubClient` and fetches stats for configured username
3. Builds `Stats` struct with aggregated data
4. Generates SVG via `svg::generate_svg()` for each theme
5. Writes `dark_mode.svg` and `light_mode.svg` to disk

### Key Modules
| Module | Purpose |
|--------|---------|
| `main.rs` | Entry point, orchestrates fetch and generation |
| `github.rs` | GraphQL client with retry logic, deserialization structs |
| `svg.rs` | SVG generation with theme support, layout constants |
| `age.rs` | Calendar-aware age calculation with tests |
| `config.rs` | TOML config parsing with validation tests |

## Environment Setup
- **Config:** `config/profile.toml` (user settings)
- **Required:** `ACCESS_TOKEN` env variable (GitHub PATH)
- Set with: `export ACCESS_TOKEN=your_token_here`

## Output Files
- `dark_mode.svg` - Dark theme (~938x560px)
- `light_mode.svg` - Light theme (~938x560px)
