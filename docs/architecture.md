# システムアーキテクチャ設計書

本ドキュメントでは、`rust-mahjong` および周辺システムの全体アーキテクチャ、コンポーネント構成、データフロー、および設計原則を解説します。

---

## 目次
1. [システム全体像 (System Context)](#1-システム全体像-system-context)
2. [モジュール構成 (Rust Core)](#2-モジュール構成-rust-core)
3. [言語境界 (Rust ⇔ Python FFI)](#3-言語境界-rust--python-ffi)
4. [並列処理 & スレッドモデル](#4-並列処理--スレッドモデル)
5. [ゼロアロケーション & メモリレイアウト](#5-ゼロアロケーション--メモリレイアウト)

---

## 1. システム全体像 (System Context)

システムは、コアとなる高性能数理エンジンを中心に、Python バインディング、Web サービス層、および GUI フロントエンドで構成されています。

```mermaid
graph TD
    subgraph Frontend["フロントエンド (mahjong-ui/frontend)"]
        UI["React 18 + TypeScript + Vite<br>Tailwind CSS / Lucide Icons"]
        HUD["リアルタイム AI HUD"]
        DrillView["何切るドリル画面"]
        ReviewView["牌譜レビュー画面"]
    end

    subgraph Backend["バックエンド (mahjong-ui/backend)"]
        FastAPI["FastAPI / Uvicorn (Python 3.10+)"]
        WS["WebSocket 対局サーバー"]
        MatchManager["MatchManager (局進行・イベント管理)"]
    end

    subgraph Engine["コアエンジン (mahjong/mahjong)"]
        PyO3Layer["Python バインディング (PyO3 / _core.pyd)"]
        RustCore["Rust コアエンジン (libmahjong)"]
    end

    UI <-->|WebSocket (JSON)| WS
    WS --> MatchManager
    MatchManager --> FastAPI
    FastAPI --> PyO3Layer
    PyO3Layer --> RustCore
```

---

## 2. モジュール構成 (Rust Core)

Rust コアエンジン内部は、責務ごとに明確にモジュール分離されています。

```mermaid
graph LR
    subgraph CoreDomain["ドメイン層"]
        Tile["tile.rs<br>牌の定義・分類"]
        Hand["hand.rs<br>手牌・純手牌・副露"]
        River["river.rs<br>河・捨て牌履歴"]
        Wall["wall.rs<br>牌山・王牌・ドラ"]
        Round["round.rs<br>局進行・手番管理"]
    end

    subgraph MathEngine["数理・判定層"]
        Shanten["shanten.rs<br>向聴数計算"]
        SuitTable["suit_table.rs<br>色別LUT"]
        Acceptance["acceptance.rs<br>有効牌・受入れ探索"]
        Yaku["yaku.rs<br>役判定 (YakuSet)"]
        Score["score.rs<br>符計算・点数計算"]
        Dora["dora.rs<br>ドラ計算"]
    end

    subgraph DecisionEngine["AI・評価・学習層"]
        Expectation["expectation.rs<br>局収支EV並列探索"]
        PlacementEV["placement_ev.rs<br>着順分布・順位点モデル"]
        CallAdvisor["call_advisor.rs<br>鳴き期待値判断"]
        Drill["drill.rs<br>何切るドリル生成"]
        Review["review.rs<br>牌譜レビュー・悪手判定"]
        Explanation["explanation.rs<br>思考根拠言語化"]
    end

    subgraph Interop["インターフェース層"]
        PythonAPI["python_api.rs<br>PyO3 バインディング"]
    end

    CoreDomain --> MathEngine
    MathEngine --> DecisionEngine
    DecisionEngine --> Interop
```

---

## 3. 言語境界 (Rust ⇔ Python FFI)

### PyO3 による直接バインディング
Python と Rust の境界において、中間 JSON シリアル化や過剰なオブジェクトアロケーションを避け、メモリ効率と呼び出し速度を両立しています。

- **プリミティブ・Enum のマッピング**:
  - `TileName` などの Rust 列挙型は、PyO3 の `#[pyclass(eq, eq_int)]` により Python 側の Enum / 整数としてゼロコピーで扱われます。
- **データ構造の変換最適化**:
  - ホットループで頻繁に往復する手牌や打牌候補リストは、Rust 側で `CandidateEvaluation` を直接 Python オブジェクトとしてインスタンス化し、Python 側のオーバーヘッドを最小化しています。
- **型定義ファイル (`_core.pyi`)**:
  - すべての PyO3 クラス・メソッドに対し、完全な型ヒント（Type Annotations）を提供。`mypy`, `pyright`, VSCode Pylance による静的型検査と入力補完をフルサポート。

---

## 4. 並列処理 & スレッドモデル

### Rayon マルチスレッド並列化
1回の打牌選択において、手牌の各打牌候補（最大14通り）に対してそれぞれ向聴数・受入れ・和了率・打点期待値をシミュレーションします。

- 各打牌候補の評価は完全に独立（Embarrassingly Parallel）であるため、`rayon::prelude::*` を用いた並列イテレータにより、CPU コア数に応じて自動スケジューリングされます。
- Python の GIL (Global Interpreter Lock) は、Rust 側の計算ループ実行中に解放（`py.allow_threads(...)`）され、Python 側のイベントループ（FastAPI / WebSocket）をブロックしません。

---

## 5. ゼロアロケーション & メモリレイアウト

### ヒープ確保の最小化
麻雀の打牌探索・役判定処理は毎秒数万〜数十万回呼び出されるホットパスです。以下の設計方針を徹底しています：

1. **固定長配列・スタック割り当て**:
   - 牌数カウント配列は `[u8; 35]`（または `[u8; 38]`）のスタック配列を使用。
   - 面子分解や役リストには `ArrayVec` / `SmallVec` を活用し、ヒープアロケーションを完全排除。
2. **ビットマスクによる集合表現**:
   - 役の集合は `u64` 1つのビットマスク（`YakuSet`）として表現。メモリフットプリントは 8 バイトのみ。
3. **インライン最適化**:
   - 単純な境界計算（スート判定、数牌の開始・終了インデックス計算）は数学的共通式 `((tile_idx - 1) / 9) * 9 + 1` に集約し、コンパイラによるインライン化を最大化。
