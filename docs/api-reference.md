# Python API リファレンス

`rust-mahjong` (`mahjong`) パッケージが提供する Python API の詳細リファレンスです。
高速な Rust コアを PyO3 経由で直接操作できる高レベル・低レベルインターフェースを網羅しています。

---

## 目次
- [インポート](#インポート)
- [基本ドメイン型](#基本ドメイン型)
  - [TileName, TileType, TileCategory](#tilename-tiletype-tilecategory)
  - [Tile](#tile)
  - [Meld](#meld)
  - [Hand](#hand)
  - [River](#river)
  - [Wall](#wall)
  - [Round](#round)
- [ルールとコンテキスト](#ルールとコンテキスト)
  - [RuleConfig](#ruleconfig)
  - [MatchContext](#matchcontext)
  - [WinContext](#wincontext)
  - [TableState](#tablestate)
- [計算・評価関数](#計算評価関数)
  - [calculate_shanten](#calculate_shanten)
  - [judge_yaku / get_all_yaku](#judge_yaku--get_all_yaku)
  - [evaluate_hand_discards](#evaluate_hand_discards)
  - [evaluate_placement_discards](#evaluate_placement_discards)
  - [calculate_orasu_conditions](#calculate_orasu_conditions)
  - [advise_call](#advise_call)
  - [generate_drill_problem](#generate_drill_problem)
  - [get_ai_hud_data](#get_ai_hud_data)
  - [ReviewTracker](#reviewtracker)

---

## インポート

```python
import mahjong as mj
# または個別インポート
from mahjong import (
    Hand,
    Meld,
    Round,
    Tile,
    TileName,
    calculate_shanten,
    evaluate_hand_discards,
    judge_yaku,
)
```

---

## 基本ドメイン型

### `TileName`, `TileType`, `TileCategory`

牌の種類・分類を定義する列挙型です。

#### `TileName` (Enum)
牌の一意な識別子（1〜34）。
- 萬子: `OneM` 〜 `NineM` (1〜9)
- 筒子: `OneP` 〜 `NineP` (10〜18)
- 索子: `OneS` 〜 `NineS` (19〜27)
- 字牌（風牌）: `East`, `South`, `West`, `North` (28〜31)
- 字牌（三元牌）: `White` (白), `Green` (發), `Red` (中) (32〜34)

**メソッド・プロパティ**:
- `as_str() -> str`: 日本語表記（例: `"一萬"`, `"東"`, `"白"`）
- `mpsz() -> str`: 短縮記法（例: `"1m"`, `"5p"`, `"1z"`）
- `tile_type -> TileType`: 牌のスート（萬子/筒子/索子/風牌/三元牌）
- `category -> TileCategory`: 么九牌 (`Honors`) または 中張牌 (`Simples`)

#### `TileType` (Enum)
`Characters` (萬子), `Circles` (筒子), `Bamboos` (索子), `Winds` (風牌), `Dragons` (三元牌)

#### `TileCategory` (Enum)
`Simples` (中張牌: 2〜8), `Honors` (么九牌: 1, 9, 字牌)

---

### `Tile`
牌のインスタンスを表現します。

```python
class Tile:
    def __init__(self, name: TileName) -> None: ...
    @property
    def name(self) -> TileName: ...
    @property
    def tile_type(self) -> TileType: ...
    @property
    def category(self) -> TileCategory: ...
    def as_str(self) -> str: ...
    def mpsz(self) -> str: ...
    def to_dict(self) -> dict[str, Any]: ...
```

---

### `Meld`
鳴き（副露: チー、ポン、大明槓、暗槓、加槓）を表現します。

```python
class Meld:
    @staticmethod
    def chii(called: TileName, consumed: list[TileName]) -> Meld: ...
    @staticmethod
    def pon(tile: TileName) -> Meld: ...
    @staticmethod
    def daiminkan(tile: TileName) -> Meld: ...
    @staticmethod
    def ankan(tile: TileName) -> Meld: ...
    @staticmethod
    def kakan(tile: TileName) -> Meld: ...

    @property
    def kind(self) -> str: ...  # "Chii", "Pon", "Daiminkan", "Ankan", "Kakan"
    @property
    def tiles(self) -> list[TileName]: ...
    def to_dict(self) -> dict[str, Any]: ...
```

---

### `Hand`
プレイヤーの手牌（純手牌＋副露）を管理します。

```python
class Hand:
    def __init__(self) -> None: ...
    @property
    def tiles(self) -> list[TileName]: ...       # 純手牌のリスト
    @property
    def open_melds(self) -> list[Meld]: ...      # 副露のリスト

    def push(self, tile: TileName) -> None: ...  # 牌を手牌に加える
    def discard(self, index: int) -> TileName: ... # 指定インデックスの牌を切る
    def call_meld(self, meld: Meld) -> None: ... # 副露を実行して手牌を更新
    def shanten(self) -> ShantenResult: ...      # 向聴数を計算
```

**使用例**:
```python
hand = Hand()
for name in [TileName.OneM, TileName.TwoM, TileName.ThreeM, TileName.East, TileName.East]:
    hand.push(name)

result = hand.shanten()
print(f"向聴数: {result.shanten}")
```

---

### `River`
プレイヤーの河（捨て牌列、ツモ切り/手出し、リーチ宣言牌、鳴かれ状態）を記録します。

```python
class River:
    def __init__(self) -> None: ...
    def push(self, tile: TileName, is_tsumogiri: bool = False, is_riichi: bool = False) -> None: ...
    def pop(self) -> Optional[TileName]: ...
    def mark_last_called(self) -> None: ...
    def to_dict(self) -> dict[str, Any]: ...
```

---

### `Wall`
牌山・王牌・ドラ表示牌を管理します。

```python
class Wall:
    def __init__(self, seed: Optional[int] = None) -> None: ...
    def draw(self) -> Optional[TileName]: ...      # 通常ツモ
    def kan_draw(self) -> Optional[TileName]: ...  # 嶺上牌ツモ
    @property
    def dora_indicators(self) -> list[TileName]: ... # 表ドラ表示牌一覧
    @property
    def remaining(self) -> int: ...                # 残りツモ可能枚数
```

---

### `Round`
1局全体の進行（配牌、各家の手牌・河、ツモ順、局状態）を管理します。

```python
class Round:
    def __init__(self, seed: Optional[int] = None) -> None: ...
    @property
    def current_turn(self) -> int: ... # 現在の手番プレイヤー（0: 東家, 1: 南家, ...）
    def hand(self, player_idx: int) -> Hand: ...
    def river(self, player_idx: int) -> River: ...
    def draw_turn(self) -> Optional[TileName]: ...
    def play_discard(self, player_idx: int, tile_idx: int, is_riichi: bool = False) -> TileName: ...
    def play_meld(self, player_idx: int, meld: Meld) -> None: ...
```

---

## ルールとコンテキスト

### `RuleConfig`
ゲームルール設定（順位点ウマ・オカ、トビ終了有無、切り上げ満貫など）を定義します。

```python
config = RuleConfig(
    uma=[30.0, 10.0, -10.0, -30.0],  # Mリーグルール
    oka=20.0,                         # 25,000点持ち30,000点返し
    tobi_enabled=False,               # トビ終了なし
    head_bump=True                    # 頭ハネ（ダブロンなし）
)
```

### `MatchContext`
半荘全体の状況（現在の局・本場・供託・4家の持ち点）を保持します。

```python
ctx = MatchContext(
    round_wind=TileName.East, # 東場
    round_number=4,           # 東4局
    honba=1,                  # 1本場
    riichi_sticks=1,          # 供託1本
    scores=[32000, 28000, 22000, 18000], # 各家の点棒
    rule_config=config
)
```

### `WinContext`
和了判定時の状況（自風、場風、ツモ/ロン、リーチ、一発、海底、嶺上、槍槓など）を渡すための構造体です。

---

## 計算・評価関数

### `calculate_shanten`
手牌の向聴数、有効牌（受入れ牌）、受入れ枚数を算出します。

```python
def calculate_shanten(hand: Hand) -> ShantenResult: ...
```

**戻り値 `ShantenResult`**:
- `shanten: int`: 向聴数（-1: 和了, 0: 聴牌, 1: 一向聴, ...）
- `acceptance: list[TileName]`: 有効牌一覧
- `total_acceptance_count: int`: 有効牌の残り合計枚数（山＋他家未見牌からの推定）
- `breakdown: dict[TileName, int]`: 牌種ごとの受入れ枚数

---

### `judge_yaku` / `get_all_yaku`
和了形の手牌と `WinContext` から役と飜数を判定します。

```python
def judge_yaku(hand: Hand, win_tile: TileName, ctx: WinContext) -> list[Yaku]: ...
def get_all_yaku() -> list[Yaku]: ...
```

**`Yaku` オブジェクト**:
- `id: YakuId`: 役ID
- `name: str`: 役名（例: `"立直"`, `"断幺九"`, `"門前清模聴"`, `"国士無双"`）
- `han: int`: 飜数（門前時）
- `open_han: int`: 鳴き時の飜数（喰い下がりの場合は -1 など）
- `is_yakuman: bool`: 役満判定

---

### `evaluate_hand_discards`
局収支期待値（和了率、打点期待値、放銃率、局収支EV）に基づいて、すべての打牌選択肢を並列探索・評価します。

```python
def evaluate_hand_discards(
    hand: Hand,
    dora_indicators: list[TileName],
    visible_tiles: list[TileName] = []
) -> list[CandidateEvaluation]: ...
```

**戻り値 `CandidateEvaluation`**:
- `tile: TileName`: 切る牌
- `shanten_after: int`: 打牌後の向聴数
- `acceptance_count: int`: 打牌後の有効牌総数
- `win_prob: float`: 和了確率（0.0 〜 1.0）
- `expected_score: float`: 和了時の期待打点（点数）
- `deal_in_prob: float`: 放銃リスク確率
- `ev: float`: 局収支期待値（点）

---

### `evaluate_placement_discards`
持ち点状況や順位点（ウマ・オカ）を加味した「最終着順期待値（順位EV）」に基づいて打牌を評価します。

```python
def evaluate_placement_discards(
    hand: Hand,
    match_ctx: MatchContext,
    player_idx: int,
    visible_tiles: list[TileName] = []
) -> list[PlacementEvaluation]: ...
```

**戻り値 `PlacementEvaluation`**:
- `tile: TileName`: 切る牌
- `ev: float`: 局収支EV
- `placement_ev: float`: 順位点期待値（ポイント単位）
- `rank_probabilities: list[float]`: 1位〜4位の着順確率分布（例: `[0.45, 0.30, 0.15, 0.10]`）

---

### `calculate_orasu_conditions`
オーラス（南4局）において、指定プレイヤーが逆転トップまたは着順浮上するために必要な和了条件（ツモ・直撃・脇ロン時の必要飜数・符・点数）を逆算します。

```python
def calculate_orasu_conditions(
    match_ctx: MatchContext,
    player_idx: int,
    target_rank: int = 1
) -> list[WinCondition]: ...
```

**戻り値 `WinCondition`**:
- `target_player: Optional[int]`: ロン直撃対象（None の場合はツモ和了）
- `required_points: int`: 逆転に必要な最小和了点
- `required_han: int`: 想定飜数
- `required_fu: int`: 想定符

---

### `advise_call`
他家の捨牌に対して、鳴くべきか（チー、ポン、カン、スルー）の期待値判断とアドバイスを出力します。

```python
def advise_call(
    hand: Hand,
    called_tile: TileName,
    from_seat_relation: str, # "Kamicha", "Toimen", "Shimocha"
    dora_indicators: list[TileName]
) -> CallAdvice: ...
```

**戻り値 `CallAdvice`**:
- `recommended_action: CallChoice`: `Pass`, `Chii`, `Pon`, `Kan`
- `expected_gain: float`: 鳴いた場合の期待値変動差分
- `explanation: str`: 鳴き判断の理由（和了速度向上、打点確定、守備力低下リスクなど）

---

### `generate_drill_problem`
学習用の「何切る」ドリル問題を自動生成します。

```python
def generate_drill_problem(
    difficulty: int = 1, # 1: 初級, 2: 中級, 3: 上級
    problem_type: str = "Nanikiru"
) -> DrillProblem: ...
```

**戻り値 `DrillProblem`**:
- `hand: Hand`: 出題手牌
- `best_discard: TileName`: 最善打牌
- `evaluations: list[CandidateEvaluation]`: 全候補打牌の評価値
- `explanation: str`: 最善打牌に至る思考プロセスの解説

---

### `ReviewTracker`
1局または半荘全体の打牌を記録し、AIの最適打牌と比較して悪手（Blunder）や疑問手を検出・採点する検討エンジンです。

```python
tracker = ReviewTracker()
# 打牌ごとに評価を追記
tracker.record_turn(
    turn_idx=12,
    player_idx=0,
    actual_discard=TileName.OneM,
    evaluations=evaluations,
    threshold_blunder_ev=1000.0 # 1000点以上のEV損失を悪手判定
)

# 終局後にサマリー取得
summary = tracker.summarize()
print(f"悪手回数: {summary['blunder_count']}, 総合レーティング: {summary['rating']}")
```
