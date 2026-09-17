# 開発者ガイド & コントリビューション規約

本ドキュメントでは、`rust-mahjong` の開発環境セットアップ、ビルド・テスト手順、コード品質基準、およびコントリビューションの流れを解説します。

---

## 目次
1. [必要要件 & 開発環境セットアップ](#1-必要要件--開発環境セットアップ)
2. [ビルド & パッケージング手順](#2-ビルド--パッケージング手順)
3. [テストの実行](#3-テストの実行)
4. [ベンチマークの測定](#4-ベンチマークの測定)
5. [コード品質基準 & 静的解析](#5-コード品質基準--静的解析)
6. [コーディング規約 (Rust / Python)](#6-コーディング規約-rust--python)

---

## 1. 必要要件 & 開発環境セットアップ

### 前提ツール
- **Rust**: 1.75 以上（最新 stable を推奨）
- **Python**: 3.10 以上
- **パッケージマネージャー**:
  - Rust: `cargo`
  - Python: `uv`（推奨）または `pip`
- **Maturin**: PyO3 ビルド用ツール（`uv pip install maturin`）

### 初期セットアップ
```bash
# リポジトリ直下（mahjong/mahjong）で仮想環境を作成
uv venv
# 仮想環境を有効化（Windows PowerShell の場合）
.venv\Scripts\Activate.ps1

# maturin および開発用依存パッケージをインストール
uv pip install maturin pytest ruff mypy
```

---

## 2. ビルド & パッケージング手順

### Rust クレート単体のビルド
```bash
# デバッグビルド
cargo build

# 最適化リリースビルド
cargo build --release
```

### Python 拡張モジュールとしての開発ビルド
`maturin develop` を使用して、仮想環境内に即座にインポート可能な共有ライブラリをビルド・リンクします。

```bash
# デバッグビルド（高速なビルド）
uv run maturin develop

# 最適化リリースビルド（高速な実行速度・ベンチマーク用）
uv run maturin develop --release
```

ビルドが完了すると、Python から `import mahjong` で呼び出せるようになります。

---

## 3. テストの実行

### Rust ユニットテスト
```bash
cargo test
```

特定のテストモジュールのみを実行する場合：
```bash
cargo test --test test_placement_ev
cargo test --test test_yaku
```

### Python 側テスト
`maturin develop` 実行後、Python 側の結合テストを実行します：
```bash
uv run pytest tests_python/
```

---

## 4. ベンチマークの測定

Criterion を使用したベンチマークスイートが用意されています。

```bash
# 全ベンチマークを実行
cargo bench

# 特定のベンチマークのみを実行
cargo bench --bench mahjong_benchmark
cargo bench --bench simd_benchmark
cargo bench --bench python_api_benchmark
```

測定結果レポートは `target/criterion/report/index.html` に生成され、HTML グラフでパフォーマンス推移や統計分布を確認できます。

---

## 5. コード品質基準 & 静的解析

PR（プルリクエスト）提出前に、以下の品質ゲートを**すべてエラー・警告ゼロ**で通過する必要があります。

### 1. Clippy (Rust Linter)
```bash
cargo clippy --all-targets -- -D warnings
```
※ 警告はすべてエラーとして扱われます。

### 2. Rustfmt (Rust フォーマッター)
```bash
cargo fmt --all -- --check
```
フォーマットを自動適用する場合は `cargo fmt --all` を実行します。

### 3. Ruff & Mypy (Python 静的解析)
```bash
uv run ruff check .
uv run mypy src/mahjong
```

---

## 6. コーディング規約 (Rust / Python)

### Rust 固有の最適化ルール
1. **Clippy 警告の排除**:
   - `for i in 1..=34` などの配列走査でインデックスのみを添字参照に用いる場合、`clippy::needless_range_loop` を回避するため `.iter().enumerate().take(35).skip(1)` などのイテレータ構文を採用する。
2. **不要な中間配列・コピーの排除**:
   - コンテキスト走査（例: 各プレイヤーのリーチ状態）では中間バッファ配列（`[bool; 4]` 等）を経由せず、直接イテレーションで条件判定と集計を行う。
3. **数牌スーツ境界の数式共通化**:
   - 萬子・筒子・索子の境界計算は `suit_start = ((tile_idx - 1) / 9) * 9 + 1` および `suit_end = suit_start + 8` により、条件分岐を大幅に削減して単一ロジックに集約する。
4. **ホットパスでのヒープアロケーション禁止**:
   - 向聴数判定、役判定、有効牌計算の内部で `Vec::new()` や `HashMap` を生成しない。スタック配列または `ArrayVec` / `SmallVec` を使用する。

### コマンド実行時の注意（CI / CLI）
- 複数のコマンドを1行に連結（`;`, `&&`, `||`, `|` 等）しないこと。必ず1行ずつ個別に実行すること。
