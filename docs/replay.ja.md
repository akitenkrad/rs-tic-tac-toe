[English](replay.md) | **日本語**

# 棋譜とリプレイ

rs-tic-tac-toe はテキスト形式の棋譜ファイル（`.tttrec`）から対局を再生できます．棋譜は手で書いても，別のプログラムで生成しても，自分の対局から保存してもかまいません．リプレイは録画にも使えるように作ってあり，一定の間隔で自動再生し，最後まで再生したら自動で終了できます．

## 棋譜の再生

```sh
rs-tic-tac-toe replay examples/records/x-wins.tttrec
```

| オプション | 説明 |
|---|---|
| `<FILE>` | 棋譜ファイル．`-` で標準入力から読む |
| `--interval <INTERVAL>` | 自動再生の 1 手あたりの間隔（ミリ秒．既定 `1500`．印が現れるアニメーションの 480 ms より短い値は 480 に切り上げ） |
| `--paused` | 一時停止の状態で開始 |
| `--exit-on-end` | 終局した棋譜の最後まで再生したら，3 秒後に終了 |
| `--print` | TUI を起動せず，再生内容をテキストで出力 |
| `--lang <LANG>` | 表示言語．`en`（既定）か `ja`．`--print` の出力にも効く |

ファイルが読めない場合や棋譜に誤りがある場合は，TUI を起動する前に問題点（行番号付き）を標準エラーに出し，終了コード 1 で終わります．

### リプレイ中の操作

| キー | 動作 |
|---|---|
| `Space` | 再生 / 一時停止 |
| `→` / `l` | 1 手進む（アニメーション付き） |
| `←` / `h` | 1 手戻る |
| `Home` / `g` | 先頭へ |
| `End` / `G` | 末尾へ |
| `+` / `-` | 速く / 遅く（1 段 250 ms） |
| `e` | 評価パネルの表示切替 |
| `q` / `Esc` / `Ctrl-C` | 終了 |

ステータス欄には再生中かどうか，手数（`k/N`），現在の間隔を表示します．最後の手で終局していれば，結果パネルに勝者の名前か引き分けを表示します．途中で終わる棋譜では「棋譜の終わり」と表示します．

### テキスト出力

`--print` は棋譜を検査し，各手と，その手の後の盤面を出力します．ファイルの確認やスクリプトからの利用に便利です．

```sh
rs-tic-tac-toe --lang ja replay examples/records/x-wins.tttrec --print
```

```text
手数 1: Alice X a1
X . .
. . .
. . .
...
手数 7: Alice X b3
X . O
O O .
X X X
勝者: Alice
```

## 自分の対局を保存する

対局中または結果画面で `s` を押すと，現在の対局を `ttt-<unix 秒>.tttrec` として保存します．保存先はカレントディレクトリです．`--record-dir <DIR>` を付けて起動すると変えられます．

```sh
rs-tic-tac-toe --record-dir ~/ttt-records
```

保存した棋譜には既定の名前を書きません．そのため英語では You / CPU，日本語では あなた / CPU と表示されます．名前を付けるには `human` / `cpu` の行を追加します．

## `.tttrec` 形式

棋譜は UTF-8 のテキストで，`key = value` 形式のヘッダの後に，1 行 1 手で着手を並べます．

```text
# rs-tic-tac-toe record v1
first = human      # human | cpu   (default: human; the first player plays X)
human = Alice      # optional display name of the "human" side (default: You / あなた)
cpu   = Bob        # optional display name of the "cpu" side   (default: CPU)

b2                 # move: a cell as a coordinate a1..c3 ...
A1                 # ... case-insensitive ...
9                  # ... or a number 1..9 in reading order (9 = c3)
```

規則:

- `#` 以降はコメント，空行は無視します．
- ヘッダのキーは最初の着手より前にだけ書けます．未知のキー・重複したキーはエラーです．
- 着手は `first` から交互で，`first` が X を持ちます．2 通りの表記は [マスの呼び方](gameplay.ja.md#マスの呼び方) を参照してください．
- `human` と `cpu` は 2 つの陣営を表すだけです．リプレイでは指定した名前で表示するので，任意の 2 人の対局を記録できます．
- 埋まっているマスへの着手と，終局後の着手はエラーです．終局前で終わる棋譜は許されます．

[`examples/records/`](../examples/records/) にサンプル棋譜が 3 つあります．

| ファイル | 内容 |
|---|---|
| `perfect-draw.tttrec` | 双方が最善を尽くして引き分けになる対局 |
| `x-wins.tttrec` | X が両取りで勝つ対局 |
| `unfinished.tttrec` | 途中で終わる棋譜 |

## 録画する

リプレイを使えば同じ対局を何度でも再現できるので，録画のやり直しも簡単です．録画する端末は 80 × 24 以上にしてください．

[asciinema](https://asciinema.org) の場合:

```sh
asciinema rec -c "rs-tic-tac-toe replay game.tttrec --exit-on-end" ttt.cast
```

[VHS](https://github.com/charmbracelet/vhs) の場合は，次のようなテープファイルを使います．

```text
Output ttt.gif
Set Width 1300
Set Height 650
Type "rs-tic-tac-toe replay game.tttrec --interval 1200 --exit-on-end"
Enter
Sleep 15s
```

`--exit-on-end` は最終盤面と結果を 3 秒間表示してから終了するので，録画がきれいに終わります．録画しながら手動で 1 手ずつ進めたい場合は，代わりに `--paused` を使ってください．
