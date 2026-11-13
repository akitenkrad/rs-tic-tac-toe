**English** | [日本語](gameplay.ja.md)

# Playing rs-tic-tac-toe

## Rules

Two players take turns placing their mark on a 3 × 3 board. The first player plays **X**, the second plays **O**. The first to get three of their marks in a row — horizontally, vertically or diagonally — wins. If the board fills up with no line, the game is a draw.

## Cell names

Cells can be named in two ways, both on screen and in [record files](replay.md):

```text
    a    b    c          1  2  3
1   a1   b1   c1         4  5  6
2   a2   b2   c2         7  8  9
3   a3   b3   c3
```

- Coordinates: a column letter `a`–`c` (left to right) and a row number `1`–`3` (top to bottom).
- Numbers: `1`–`9` in reading order, starting at the top left. Note that this is **not** the numeric-keypad layout.

## Starting a game

Run `rs-tic-tac-toe` (or `cargo run --release`) to open the title screen.

| Key | Setting | Values |
|---|---|---|
| `d` | Difficulty | Easy / Normal / Perfect |
| `f` | First player (plays X) | You / CPU |
| `l` | Language | English / 日本語 |
| `Enter` / `Space` | Start | |
| `q` | Quit | |

The CPU picks one of the best moves (see [strategy](strategy.md)) with a probability that depends on the difficulty — 20 % on Easy, 65 % on Normal — and otherwise plays a random empty cell. On Perfect it always plays a best move and never loses.

## Controls during a game

| Key | Action |
|---|---|
| `←` `↑` `↓` `→` | Move the cursor |
| `1`-`9` | Select that cell (press the same number again, or `Enter`, to place) |
| `Enter` / `Space` | Place your mark on the selected cell |
| `h` | Hint: highlight the best move, or tell you when only a draw or a loss is left |
| `e` | Toggle the evaluation panel |
| `s` | Save the current game as a record file (see [replay](replay.md)) |
| `n` | New game |
| `t` / `Esc` | Back to the title screen |
| `q` / `Ctrl-C` | Quit |

The CPU briefly highlights the cell it has chosen before placing its mark, and new marks fade in. When someone wins, the winning line flashes; winning showers the screen with confetti and losing makes it rain. The side panel keeps a win / loss / draw tally for the session.

## The evaluation panel

Press `e` to label every empty cell with the result of playing there, assuming perfect play afterwards, from the point of view of the player to move:

- **Win n** — you win, and the game ends after `n` moves counting this one (`Win 1` is an immediate win).
- **Draw** — best play from both sides leads to a draw.
- **Loss n** — the opponent can force a win, ending the game after `n` moves.

## Language

The interface is in English by default. Switch to Japanese with `l` on the title screen, or start the game with `--lang ja` (this also works for `replay`).

## Terminal requirements

- At least **80 × 24** cells. A smaller terminal shows a message asking you to enlarge it.
- A true-colour (24-bit) terminal is recommended: WezTerm, iTerm2, kitty, Alacritty, …
- Marks are drawn with background-coloured cells, so no special font is required.
- If box-drawing characters look misaligned in a CJK locale, set your terminal's *East Asian Ambiguous width* to 1 (narrow).
