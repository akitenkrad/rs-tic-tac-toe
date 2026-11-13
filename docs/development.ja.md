[English](development.md) | **日本語**

# 開発者向け情報

## ビルド

```sh
cargo build --release
cargo run --release                      # play
cargo run --release -- replay FILE       # replay a record
```

rs-tic-tac-toe は Rust edition 2024 で，描画に [ratatui](https://ratatui.rs)，コマンドラインに [clap](https://docs.rs/clap) を使っています．構成は [rs-nim](https://github.com/akitenkrad/rs-nim) にならっています．

## ソース構成

| ファイル | 役割 |
|---|---|
| `src/main.rs` | コマンドラインの解析，イベントループ（33 ms ごとの tick），`replay --print` |
| `src/lib.rs` | 以下のモジュールをライブラリとして公開する（結合テストから使うため） |
| `src/game.rs` | 盤，ルール，マスの表記，メモ化したミニマックス探索（`solve`，`best_moves`）．UI に依存しない |
| `src/ai.rs` | CPU の着手選択（難易度ごとの正答率）と小さな xorshift 乱数 |
| `src/record.rs` | `.tttrec` 棋譜の読み書きと，任意の手数の局面の再構成 |
| `src/app.rs` | 画面遷移，入力，通常モードとリプレイモード，CPU の思考演出，アニメーション，セッション戦績，紙吹雪と雨 |
| `src/ui.rs` | 描画 |

`App` は 2 つのモードのどちらかで動きます．`Mode::Normal`（CPU との対局）と `Mode::Replay`（`Record` の再生）です．リプレイモードは盤の描画を通常モードと共有します．1 手戻るときは着手を取り消すのではなく，棋譜から局面を作り直します．

## テスト

```sh
cargo test                              # unit and integration tests
cargo test -- --ignored --nocapture     # text dump of the screens and a full game against the CPU
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

主なテスト:

- `game` のテストは，8 本のライン，引き分け，不正な手を確かめ，到達可能なすべての局面で `solve` がメモ化なしのミニマックスと一致することを確かめます．
- `ai` のテストは，最強の CPU が X でも O でも，相手のあらゆる着手列に対して一度も負けないこと，最強同士が必ず引き分けになることを確かめます．
- `record` のテストは，2 通りの表記，行番号付きの各エラー，`to_text` → `parse` の往復，`examples/records/` の全ファイルを確かめます．
- `app` と `ui` のテストは，リプレイの前進・後退・先頭・末尾への移動，保存した対局を読み戻せることを確かめ，最小サイズ 80 × 24 で全画面を描画します．
- `tests/regression/` には，テスト中に見つかった境界条件（パーサの端のケースと状態遷移）の結合テストを置いています．
