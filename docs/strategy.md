**English** | [日本語](strategy.ja.md)

# Strategy and perfect play

Tic-tac-toe is small enough to solve completely. rs-tic-tac-toe searches the whole game tree in `game::solve`, which powers the hint (`h`), the evaluation panel (`e`) and the CPU.

## The game is a draw

With perfect play from both sides, every game of tic-tac-toe ends in a draw. Every opening move — corner, edge or centre — still allows the first player to draw, and none of them forces a win. You can see this on an empty board by pressing `e`: every cell reads *Draw*.

So the game is won by **mistakes**. The evaluation panel shows exactly which moves are mistakes: any cell marked *Loss* hands your opponent a forced win.

## Forks

The usual way to win is a **fork**: a move that creates two threats (two lines with two of your marks and an empty third cell) at once. Your opponent can block only one of them.

The sample record [`examples/records/x-wins.tttrec`](../examples/records/x-wins.tttrec) shows one:

```text
X . O        X plays a3 and threatens both
. O .   ->   a1-a2-a3 (needs a2) and
. . X        a3-b3-c3 (needs b3)
```

To defend against forks, block a threat early or create your own threat so that your opponent must answer it.

## How the search works

`game::solve` evaluates every empty cell with **minimax**: a move is a win if some reply sequence forces three in a row for the player to move, a loss if the opponent can force one, and a draw otherwise. Results are memoised by board, so the whole tree is searched in a fraction of a second.

Each result also records how many moves remain until the game ends. Among equally good moves, `game::best_moves` prefers faster wins and slower losses, so the CPU finishes won games quickly and holds out as long as possible in lost ones.

## Verification

The tests check the search exhaustively:

- `solve` agrees with a plain, unmemoised minimax on every reachable position.
- The Perfect CPU, playing first or second, never loses against **any** sequence of opponent moves.
- Perfect against Perfect always ends in a draw.
- The empty board is a draw and every opening move draws.
