# halfguru - AI Agent Guide

## Project Overview
Rust-based GitHub statistics generator that creates terminal-style SVG profile cards. Fetches real-time GitHub stats for a user and generates dark/light mode SVG images with neofetch-inspired formatting for README embedding.

## Tech Stack
- **Language:** Rust 2024 Edition
- **Runtime:** Tokio 1.48 (async)
- **HTTP Client:** reqwest 0.12
- **Serialization:** serde/serde_json
- **Date/Time:** chrono 0.4
- **Error Handling:** anyhow 1.0

## Key Files

| File | Purpose |
|------|---------|
| `src/main.rs` | Entry point: calculates age, fetches GitHub stats, generates SVGs |
| `src/github.rs` | GraphQL API client with retry logic and rate limiting handling (547 lines) |
| `src/svg.rs` | SVG generation engine for dark/light themes (430 lines) |
| `src/age.rs` | Human-readable age calculation (years, months, days) with calendar-aware logic |
| `src/stats.rs` | Statistics data structures (repos, stars, followers, commits, LOC) |
| `src/ascii.rs` | ASCII art constants |
| `src/ascii.txt` | ASCII art content (25 lines) |

## Commands

### Build & Run
- `cargo build --release` - Build release binary
- `cargo run` - Build and run (generates `dark_mode.svg` and `light_mode.svg`)

### Testing & Quality (AI should run after edits)
- `cargo test` - Run tests
- `cargo fmt` - Format code
- `cargo clippy` - Run linter

## Important Patterns

### Async Architecture
- All API calls use async/await with Tokio runtime
- GitHub API client uses `Arc` for thread-safe shared state

### Error Handling
- Uses `anyhow::Result<T>` with context via `.context()`
- Retry logic with exponential backoff for 429 and 5xx responses

### Code Style
- GraphQL queries as inline strings with `format!`
- Theme-based styling using enums and match expressions
- SVG generation uses string templates with manual XML escaping

### Data Flow
1. `main.rs` calculates age from hardcoded birthday
2. Creates GitHub client and fetches stats for user
3. Generates dark and light mode SVGs via `svg.rs`
4. Writes SVG files to disk

## Environment Setup
- **Required:** `ACCESS_TOKEN` environment variable (GitHub personal access token)
- Set with: `export ACCESS_TOKEN=your_token_here`

## Output Files
- `dark_mode.svg` - Dark theme card (~938x560px)
- `light_mode.svg` - Light theme card (~938x560px)
