**English** | [日本語](development.ja.md)

# Development

## Building

```sh
cargo build --release
cargo run --release                      # play
cargo run --release -- replay FILE       # replay a record
```

rs-tic-tac-toe uses Rust edition 2024, [ratatui](https://ratatui.rs) for drawing and [clap](https://docs.rs/clap) for the command line. Its structure follows [rs-nim](https://github.com/akitenkrad/rs-nim).

## Source layout

| File | Role |
|---|---|
| `src/main.rs` | Command-line parsing, the event loop (33 ms tick) and `replay --print` |
| `src/lib.rs` | Exposes the modules below as a library, so integration tests can use them |
| `src/game.rs` | Board, rules, cell notation, and the memoised minimax solver (`solve`, `best_moves`). Independent of the UI |
| `src/ai.rs` | CPU move selection (per-difficulty accuracy) and a small xorshift RNG |
| `src/record.rs` | Reading and writing `.tttrec` records, and rebuilding the position after any number of moves |
| `src/app.rs` | Screens, input, normal and replay modes, CPU "thinking", animations, session tally, confetti and rain |
| `src/ui.rs` | Drawing |

`App` runs in one of two modes: `Mode::Normal` (play against the CPU) and `Mode::Replay` (play back a `Record`). Replay mode reuses the same board drawing; stepping backwards rebuilds the position from the record instead of undoing moves.

## Tests

```sh
cargo test                              # unit and integration tests
cargo test -- --ignored --nocapture     # text dump of the screens and a full game against the CPU
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

Highlights:

- `game` tests check all eight lines, draws, illegal moves, and that `solve` agrees with an unmemoised minimax on every reachable position.
- `ai` tests check that the Perfect CPU never loses, as X or O, against every possible sequence of opponent moves, and that Perfect against Perfect always draws.
- `record` tests cover both notations, every error kind with its line number, the `to_text` → `parse` round trip and every file in `examples/records/`.
- `app` and `ui` tests drive replay forward, backward, to the start and to the end, check that saved games parse back, and render every screen at the minimum 80 × 24 size.
- `tests/regression/` holds integration tests for boundary cases found while testing: parser edge cases and state transitions.
