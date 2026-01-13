# halfguru - AI Agent Guide

## Project Overview
Rust-based GitHub statistics generator that creates terminal-style SVG profile cards. Fetches real-time GitHub stats for a user and generates dark/light mode SVG images with neofetch-inspired formatting for README embedding.

## Tech Stack
- **Language:** Rust 2024 Edition
- **Runtime:** Tokio 1.48 (async)
- **HTTP Client:** reqwest 0.12
- **Serialization:** serde/serde_json/toml
- **Date/Time:** chrono 0.4
- **Error Handling:** anyhow 1.0
- **Testing:** tempfile (for config tests)

## Key Files

| File | Purpose | Lines |
|------|---------|-------|
| `src/main.rs` | Entry point: loads config, fetches GitHub stats, generates SVGs | 39 |
| `src/github.rs` | GraphQL API client with retry logic and repeated deserialization structs | 546 |
| `src/svg.rs` | SVG generation engine with self-documenting layout constants | 441 |
| `src/age.rs` | Human-readable age calculation with calendar-aware logic (tests included) | 190 |
| `src/config.rs` | TOML configuration loader with validation tests | 182 |
| `src/ascii.txt` | ASCII art content (25 lines) | 25 |

## Commands

### Build & Run
- `cargo build --release` - Build release binary
- `cargo run` - Build and run (generates `dark_mode.svg` and `light_mode.svg`)

### Testing & Quality (AI should run after edits)
- `cargo test` - Run tests
- `cargo fmt` - Format code
- `cargo clippy` - Run linter

## Important Patterns

### Modularity & Clarity
- Age calculation self-contained in `age.rs` with public `age_from_birthday()` API
- SVG constants use self-documenting names (e.g., `TEXT_TOP_MARGIN_PX`) that explain visual impact

### Async Architecture
- All API calls use async/await with Tokio runtime
- GitHub API client uses `Arc` for thread-safe shared state
- Monolithic `graphql()` method handles retries with exponential backoff for 429 and 5xx responses

### Error Handling
- Uses `anyhow::Result<T>` with context via `.context()`
- Rate limiting honors Retry-After header, 5xx errors use exponential backoff
- Errors in `total_loc` are logged to stderr without failing entire operation

### Code Style
- GraphQL queries as inline strings with `format!`
- Theme-based styling using enums and match expressions
- SVG generation uses string templates with manual XML escaping
- Use `if let` with let-chains instead of nested if statements

### Data Flow
1. `main.rs` loads TOML config file
2. Creates GitHub client and fetches stats for username from config
3. Generates dark and light mode SVGs via `svg.rs` (age calculated internally)
4. Writes SVG files to disk

## Testing

### Test Coverage
- `config.rs` tests: config parsing, missing required fields, invalid birthday format
- `age.rs` tests: leap year rules, month lengths, plural logic, age calculation edge cases
- Tests use `tempfile` for isolated file I/O testing

### Running Tests
- `cargo test` - Run all tests
- Tests verify core logic without requiring GitHub API calls or mocking

## Environment Setup
- **Config:** `config/profile.toml` contains user settings (birthday, GitHub username, display info)
- **Required:** `ACCESS_TOKEN` environment variable (GitHub personal access token)
- Set with: `export ACCESS_TOKEN=your_token_here`

## Output Files
- `dark_mode.svg` - Dark theme card (~938x560px)
- `light_mode.svg` - Light theme card (~938x560px)
