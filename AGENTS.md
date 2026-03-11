# AGENTS.md - Coding Agent Guidelines

## Workflow Orchestration

### 1. Plan Mode Default
- Enter plan mode for ANY non-trivial task (3+ steps or architectural decisions)
- If something goes sideways, STOP and re-plan immediately - don't keep pushing
- Use plan mode for verification steps, not just building
- Write detailed specs upfront to reduce ambiguity

### 2. Self-Improvement Loop
- After ANY correction from the user: update `tasks/lessons.md` with the pattern
- Write rules for yourself that prevent the same mistake
- Ruthlessly iterate on these lessons until mistake rate drops
- Review lessons at session start for relevant project

### 3. Verification Before Done
- Never mark a task complete without proving it works
- Diff behavior between main and your changes when relevant
- Ask yourself: "Would a staff engineer approve this?"
- Run tests, check logs, demonstrate correctness

### 4. Demand Elegance (Balanced)
- For non-trivial changes: pause and ask "is there a more elegant way?"
- If a fix feels hacky: "Knowing everything I know now, implement the elegant solution"
- Skip this for simple, obvious fixes - don't over-engineer
- Challenge your own work before presenting it

### 5. Autonomous Bug Fixing
- When given a bug report: just fix it. Don't ask for hand-holding
- Point at logs, errors, failing tests then resolve them
- Zero context switching required from the user
- Go fix failing CI tests without being told how

## Task Management

1. **Plan First**: Write plan to `tasks/todo.md` with checkable items
2. **Verify Plan**: Check in before starting implementation
3. **Track Progress**: Mark items complete as you go
4. **Explain Changes**: High-level summary at each step
5. **Document Results**: Add review section to plan `tasks/todo.md`
6. **Capture Lessons**: Update `tasks/lessons.md` after corrections

## Core Principles

- **Simplicity First**: Make every change as simple as possible. Impact minimal code.
- **No Laziness**: Find root causes. No temporary fixes. Senior developer standards.
- **Minimal Impact**: Changes should only touch what's necessary. Avoid introducing bugs.

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

## Attribution

Inspired by [Andrew6rant](https://github.com/Andrew6rant/Andrew6rant) - neofetch-style GitHub profile card generator.
