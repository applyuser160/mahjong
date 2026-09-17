# rust-mahjong (`mahjong`)

[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org)
[![Python](https://img.shields.io/badge/python-3.10%2B-blue.svg)](https://www.python.org)
[![PyO3](https://img.shields.io/badge/PyO3-0.25-yellow.svg)](https://pyo3.rs)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)

超高速な麻雀数理判定・期待値探索・AI学習評価エンジン。
コアロジックを Rust で極限まで最適化し、PyO3 経由で Python から直感的に操作できる高機能ライブラリです。

---

## 主な特徴

- ⚡ **超高速な向聴数・有効牌計算**:
  - 色別ルックアップテーブル（Suit LUT）とビットパッキングによるナノ秒単位の向聴数判定
  - AVX2 / SWAR ベクトル命令によるスーツキー生成 & 七対子判定の高速化
- 🎯 **ゼロアロケーション役・点数判定**:
  - `YakuSet`（64bit ビットマスク）による全40役以上の超高速判定
  - ホットパスでのヒープメモリ確保（`Vec` / `HashMap`）を完全排除
- 📊 **期待値 (EV) & 着順分布モデル**:
  - 打牌候補の和了率・期待打点・放銃リスクを局収支 EV として算出
  - Rayon による打牌候補のマルチコア CPU 並列探索
  - Mリーグルール等のウマ・オカ・点棒状況を考慮した「順位点期待値 (Placement EV)」モデル
- 🏆 **実戦支援 & 学習機能**:
  - **オーラス逆転条件計算**: 着順浮上・逆転トップに必要な役・飜数・直撃/ツモ条件を自動逆算
  - **鳴きアドバイザー**: チー・ポン・カンの期待値差分を評価して推奨手を提示
  - **何切るドリル生成**: 難易度別の牌姿・正解手・解説を自動生成
  - **牌譜レビュー**: 終局後の悪手（Blunder）・疑問手を客観的な EV 損失で検出

---

## パフォーマンス・ベンチマーク

| 処理内容 | 実行速度 / スループット | 最適化技術 |
| :--- | :--- | :--- |
| **向聴数判定 (1手牌)** | **約 15 ns** | Suit LUT + SIMD |
| **役・点数計算 (和了形)** | **約 40 ns** | `YakuSet` ビットマスク + ゼロアロケーション |
| **打牌候補 EV 全探索 (14打牌)** | **< 0.5 ms** | Rayon マルチスレッド並列処理 |
| **有効牌探索 (枝刈り込み)** | **約 120 ns** | 孤立牌・見え牌スキップ枝刈り |

※ Criterion によるベンチマーク測定結果（CPU: AMD Ryzen / Intel Core 環境）

---

## クイックスタート (Python)

### インストール & 開発ビルド
```bash
# リポジトリ直下で仮想環境を構築してビルド
uv venv
uv pip install maturin
uv run maturin develop --release
```

### 基本的な使い方

```python
import mahjong as mj

# 1. 手牌の作成と向聴数・有効牌計算
hand = mj.Hand()
tiles = [
    mj.TileName.OneM, mj.TileName.TwoM, mj.TileName.ThreeM,
    mj.TileName.FourP, mj.TileName.FiveP, mj.TileName.SixP,
    mj.TileName.SevenS, mj.TileName.EightS, mj.TileName.NineS,
    mj.TileName.East, mj.TileName.East,
    mj.TileName.OneP, mj.TileName.NineP, mj.TileName.North,
]
for t in tiles:
    hand.push(t)

result = hand.shanten()
print(f"向聴数: {result.shanten}")  # 0 (聴牌)
print(f"受入れ有効牌: {[t.mpsz() for t in result.acceptance]}")

# 2. 打牌ごとの局収支期待値 (EV) 探索
dora = [mj.TileName.OneM]
evaluations = mj.evaluate_hand_discards(hand, dora)
best = evaluations[0]
print(f"推奨打牌: {best.tile.mpsz()}, EV: {best.ev:.1f}, 和了率: {best.win_prob*100:.1f}%")

# 3. 役判定
# 和了牌を加えた Hand と WinContext から成立役を判定
# win_ctx = mj.WinContext(...)
# yaku_list = mj.judge_yaku(hand, mj.TileName.OneP, win_ctx)
```

---

## プロジェクト構成

```
mahjong/
├── src/
│   ├── lib.rs              # クレートルート・エクスポート
│   ├── python_api.rs       # PyO3 Python バインディング
│   └── mahjong/            # 各種エンジンモジュール
│       ├── tile.rs         # 牌の定義
│       ├── hand.rs         # 手牌
│       ├── shanten.rs      # 向聴数計算
│       ├── suit_table.rs   # スーツLUT
│       ├── yaku.rs         # 役判定 (YakuSet)
│       ├── score.rs        # 符・点数計算
│       ├── expectation.rs  # 期待値探索 (Rayon)
│       ├── placement_ev.rs # 順位点・着順分布モデル
│       ├── call_advisor.rs # 鳴きアドバイザー
│       ├── drill.rs        # 何切るドリル生成
│       ├── review.rs       # 牌譜検討エンジン
│       └── _core.pyi       # Python型ヒント定義
├── tests/                  # Rust ユニットテスト群
├── tests_python/           # Python 結合テスト群
├── benches/                # Criterion ベンチマークスイート
└── docs/                   # 詳細ドキュメント
    ├── api-reference.md    # Python API 完全リファレンス
    ├── algorithms.md       # 数理モデル & 高速化アルゴリズム仕様
    ├── architecture.md     # システムアーキテクチャ設計書
    └── developer-guide.md  # 開発者ガイド & コントリビューション規約
```

---

## ドキュメント一覧

- 📖 [**Python API リファレンス**](docs/api-reference.md): クラス・関数の完全な型定義・仕様とコード例
- 📐 [**数理 & アルゴリズム仕様書**](docs/algorithms.md): LUT、SIMD、YakuSet、EVモデル、順位点分布の数式・設計
- 🏛️ [**システムアーキテクチャ**](docs/architecture.md): 全体構成図、データフロー、FFI境界、並列モデル
- 🛠️ [**開発者ガイド**](docs/developer-guide.md): ビルド、テスト、ベンチマーク、品質規約 (Clippy / Fmt)

---

## ライセンス

本プロジェクトは [MIT License](LICENSE) のもとで公開されています。
