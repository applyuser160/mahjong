from enum import Enum
from typing import Any, Optional

class PyTileType(Enum):
    """Represents the type of a Mahjong tile."""

    None_ = 0
    Characters = 1
    Circles = 2
    Bamboos = 3
    Winds = 4
    Dragons = 5

class PyTileCategory(Enum):
    """Represents the category of a Mahjong tile."""

    None_ = 0
    Simples = 1
    Honors = 2

class PyTileName(Enum):
    """Represents the exact name of a Mahjong tile."""

    None_ = 0
    OneM = 1
    TwoM = 2
    ThreeM = 3
    FourM = 4
    FiveM = 5
    SixM = 6
    SevenM = 7
    EightM = 8
    NineM = 9
    OneP = 10
    TwoP = 11
    ThreeP = 12
    FourP = 13
    FiveP = 14
    SixP = 15
    SevenP = 16
    EightP = 17
    NineP = 18
    OneS = 19
    TwoS = 20
    ThreeS = 21
    FourS = 22
    FiveS = 23
    SixS = 24
    SevenS = 25
    EightS = 26
    NineS = 27
    East = 28
    South = 29
    West = 30
    North = 31
    Red = 32
    Green = 33
    White = 34

    def as_str(self) -> str: ...
    def mpsz(self) -> str: ...
    @property
    def tile_type(self) -> PyTileType: ...
    @property
    def category(self) -> PyTileCategory: ...

class PyTile:
    """Represents a Mahjong tile with name, type, and category."""

    def __init__(self, name: PyTileName) -> None: ...
    @property
    def name(self) -> PyTileName: ...
    @property
    def tile_type(self) -> PyTileType: ...
    @property
    def category(self) -> PyTileCategory: ...
    def as_str(self) -> str: ...
    def mpsz(self) -> str: ...
    def to_dict(self) -> dict[str, Any]: ...

class PyMeld:
    """Represents a Mahjong meld (Chii, Pon, Kan)."""

    @staticmethod
    def chii(called: PyTileName, consumed: list[PyTileName]) -> "PyMeld": ...
    @staticmethod
    def pon(tile: PyTileName) -> "PyMeld": ...
    @staticmethod
    def daiminkan(tile: PyTileName) -> "PyMeld": ...
    @staticmethod
    def ankan(tile: PyTileName) -> "PyMeld": ...
    @staticmethod
    def kakan(tile: PyTileName) -> "PyMeld": ...
    @property
    def kind(self) -> str: ...
    @property
    def tiles(self) -> list[PyTileName]: ...
    def to_dict(self) -> dict[str, Any]: ...

class PyHand:
    """Represents a player's hand."""

    def __init__(self) -> None: ...
    @property
    def tiles(self) -> list[PyTileName]: ...
    @property
    def open_melds(self) -> list[PyMeld]: ...
    def push(self, tile: PyTileName) -> None: ...
    def discard(self, index: int) -> PyTileName:
        """Discards a tile by index. Can raise ValueError."""
        ...

    def call_meld(self, meld: PyMeld) -> None:
        """Calls a meld, updating the hand. Can raise ValueError."""
        ...

    def shanten(self) -> "PyShantenResult":
        """Calculates the shanten number of the hand."""
        ...

    def to_dict(self) -> dict[str, Any]: ...

class PyRiver:
    """Represents a player's river (discard pile)."""

    def __init__(self) -> None: ...
    @property
    def tiles(self) -> list[PyTileName]: ...
    def to_dict(self) -> dict[str, Any]: ...

class PyWall:
    """Represents the Mahjong wall."""

    def __init__(self) -> None: ...
    def shuffle(self, seed: int) -> None: ...
    def draw(self) -> Optional[PyTileName]: ...
    def draw_replacement(self) -> Optional[PyTileName]: ...
    def remaining(self) -> int: ...

class PyRound:
    """Represents a round of Mahjong."""

    def __init__(self, wall: PyWall) -> None: ...
    def turn(self) -> int: ...
    def hand(self, index: int) -> list[PyTileName]: ...
    def river(self, index: int) -> PyRiver: ...
    def draw_tile(self) -> Optional[PyTileName]: ...
    def discard_tile(self, index: int) -> PyTileName:
        """Discards a tile for the current turn. Can raise ValueError."""
        ...

    def play_meld(self, player_index: int, meld: PyMeld) -> None:
        """Plays a meld. Can raise ValueError."""
        ...

class PyYakuId(Enum):
    """Enum representing all possible Yaku IDs."""

    Riichi = 0
    MenzenTsumo = 1
    Tanyao = 2
    Pinfu = 3
    Ipeiko = 4
    YakuhaiHaku = 5
    YakuhaiHatsu = 6
    YakuhaiChun = 7
    YakuhaiJikaze = 8
    YakuhaiBakaze = 9
    Chitoitsu = 10
    Toitoi = 11
    Sanankou = 12
    Shousangen = 13
    Chantaiyao = 14
    Ryanpeiko = 15
    SanshokuDoujun = 16
    SanshokuDoukou = 17
    Honitsu = 18
    Junchan = 19
    Chinitsu = 20
    Chinroutou = 21
    Honroutou = 22
    Sankantsu = 23
    KokushiMusou = 24
    Suuankou = 25
    Daisangen = 26
    Shousuushi = 27
    Daisuushi = 28
    Suukantsu = 29
    Tsuuiisou = 30
    Ryuuiisou = 31
    ChuurenPoutou = 32
    Tenhou = 33
    Chiihou = 34
    RinshanKaihou = 35
    Chankan = 36
    HaiteiRaoyue = 37
    HouteiRaoyui = 38
    DoubleRiichi = 39
    Ippatsu = 40

class PyYaku:
    """Represents a Yaku with its properties."""

    @property
    def id(self) -> PyYakuId: ...
    @property
    def name_ja(self) -> str: ...
    @property
    def name_kana(self) -> str: ...
    @property
    def han_closed(self) -> int: ...
    @property
    def han_open(self) -> int: ...
    @property
    def yakuman(self) -> bool: ...

def get_all_yaku() -> list[PyYaku]:
    """Returns a list of all defined Yaku."""
    ...

class PyWinContext:
    """Context required to judge Yaku."""

    is_closed: bool
    is_tsumo: bool
    seat_wind: Optional[PyTileName]
    round_wind: Optional[PyTileName]
    riichi: bool
    kan_count: int
    tenhou: bool
    chiihou: bool
    win_tile: Optional[PyTileName]
    is_rinshan: bool
    is_chankan: bool
    is_haitei: bool
    is_houtei: bool
    is_double_riichi: bool
    is_ippatsu: bool

    def __init__(
        self,
        is_closed: bool = True,
        is_tsumo: bool = True,
        seat_wind: Optional[PyTileName] = None,
        round_wind: Optional[PyTileName] = None,
        riichi: bool = False,
        kan_count: int = 0,
        tenhou: bool = False,
        chiihou: bool = False,
        win_tile: Optional[PyTileName] = None,
        is_rinshan: bool = False,
        is_chankan: bool = False,
        is_haitei: bool = False,
        is_houtei: bool = False,
        is_double_riichi: bool = False,
        is_ippatsu: bool = False,
    ) -> None: ...

def py_judge_yaku(
    tiles: list[PyTileName], melds: list[PyMeld], context: PyWinContext
) -> list[PyYakuId]:
    """Judges the Yaku present in the given hand and context."""
    ...

class PyShantenResult:
    """Represents the shanten (minimum steps to ready hand) calculation result."""

    min_shanten: int
    normal: int
    chitoitsu: int
    kokushi: int

def py_calculate_shanten(
    tiles: list[PyTileName], open_melds_count: int = 0
) -> PyShantenResult:
    """Calculates the shanten number from a list of tiles and open melds count."""
    ...

class PyCandidateEvaluation:
    """Evaluation of a discard candidate tile."""

    discard_tile: PyTileName
    shanten_after: int
    ev: float
    remaining_count: int
    expected_score: float
    expected_han: float
    risk_score: float
    is_safe: bool
    primary_yaku: list[str]

    def to_dict(self) -> dict[str, Any]: ...

def py_evaluate_hand_discards(
    tiles: list[PyTileName],
    is_dealer: bool = True,
    dora_indicators: Optional[list[PyTileName]] = None,
    turn_number: Optional[int] = None,
    remaining_wall_tiles: Optional[int] = None,
    seat_wind: Optional[PyTileName] = None,
    round_wind: Optional[PyTileName] = None,
    visible_tiles: Optional[list[PyTileName]] = None,
) -> list[PyCandidateEvaluation]:
    """Evaluates all possible discards from the hand based on EV, acceptance, and value."""
    ...

class PyReviewTracker:
    """Tracks discard decisions and generates post-match review reports."""

    def __init__(self) -> None: ...
    def record_decision(
        self,
        turn: int,
        chosen_tile: PyTileName,
        candidates: list[PyCandidateEvaluation],
    ) -> None: ...
    def get_accuracy_rate(self) -> float: ...
    def get_total_ev_loss(self) -> float: ...
    def format_report(self) -> str: ...
    def to_dict(self) -> dict[str, Any]: ...
    def get_blunders_dict(
        self, threshold: Optional[float] = None
    ) -> list[dict[str, Any]]: ...

class PyCallChoice:
    """A call action choice evaluation."""

    action: str
    post_shanten: int
    post_acceptance: int
    estimated_score: float
    ev: float

    def to_dict(self) -> dict[str, Any]: ...

class PyCallAdvice:
    """Comprehensive call advice for a discarded tile."""

    target_tile: PyTileName
    is_kamicha: bool
    best_action: str
    recommendation: str
    rationale: str
    choices: list[PyCallChoice]

    def to_dict(self) -> dict[str, Any]: ...

def py_advise_call(
    tiles: list[PyTileName],
    target_tile: PyTileName,
    is_kamicha: bool = True,
    dora_indicators: Optional[list[PyTileName]] = None,
) -> Optional[PyCallAdvice]:
    """Generates advice on whether to call (chii, pon, kan) or pass."""
    ...

class PyDrillProblem:
    """A generated what-to-discard drill problem."""

    tiles: list[PyTileName]
    dora_indicator: PyTileName
    turn_number: int
    best_tile: PyTileName
    rationale: str
    candidates: list[PyCandidateEvaluation]

    def to_dict(self) -> dict[str, Any]: ...

def py_generate_drill_problem(
    target_shanten: Optional[int] = None,
) -> Optional[PyDrillProblem]:
    """Generates a what-to-discard drill problem for the specified shanten level."""
    ...

class PyRuleConfig:
    """Point and placement rule configuration (Uma, Oka)."""

    origin_score: int
    return_score: int
    uma: list[int]
    oka: int

    def __init__(
        self,
        origin_score: int = 25000,
        return_score: int = 30000,
        uma: Optional[list[int]] = None,
        oka: int = 0,
    ) -> None: ...
    @staticmethod
    def mleague() -> "PyRuleConfig": ...
    @staticmethod
    def general() -> "PyRuleConfig": ...
    @staticmethod
    def tenhou_dan() -> "PyRuleConfig": ...
    def to_dict(self) -> dict[str, Any]: ...

class PyMatchContext:
    """Current match context including player scores, round, and rules."""

    scores: list[int]
    round_wind: PyTileName
    round_number: int
    honba: int
    riichi_sticks: int
    dealer_idx: int
    rule: PyRuleConfig

    def __init__(
        self,
        scores: Optional[list[int]] = None,
        round_wind: Optional[PyTileName] = None,
        round_number: int = 1,
        honba: int = 0,
        riichi_sticks: int = 0,
        dealer_idx: int = 0,
        rule: Optional[PyRuleConfig] = None,
    ) -> None: ...
    def current_ranks(self) -> list[int]: ...
    def score_diff(self, p: int, target: int) -> int: ...
    def is_orasu(self) -> bool: ...
    def remaining_rounds(self) -> int: ...
    def to_dict(self) -> dict[str, Any]: ...

class PyWinCondition:
    """Final round (orasu) win condition requirement to overturn rank."""

    target_rank: int
    target_player: int
    diff: int
    ron_direct_req: Optional[int]
    tsumo_req: Optional[int]
    ron_other_req: Optional[int]
    summary: str

    def to_dict(self) -> dict[str, Any]: ...

class PyPlacementEvaluation:
    """Discard candidate evaluation incorporating placement EV and rank odds."""

    discard_tile: PyTileName
    raw_ev: float
    placement_ev: float
    expected_rank: float
    rank_probabilities: list[float]
    situational_note: str
    shanten_after: int
    remaining_count: int
    expected_score: float
    risk_score: float
    is_safe: bool

    def to_dict(self) -> dict[str, Any]: ...

class PyTableState:
    """Full table snapshot for graphical UI rendering."""

    round_wind: PyTileName
    round_number: int
    honba: int
    riichi_sticks: int
    dealer_idx: int
    current_turn: int
    dora_indicators: list[PyTileName]
    remaining_wall_tiles: int
    scores: list[int]
    is_riichi: list[bool]
    hands: list[list[PyTileName]]
    melds: list[list[PyMeld]]
    rivers: list[list[PyTileName]]

    def __init__(
        self,
        round_wind: PyTileName,
        round_number: int,
        honba: int,
        riichi_sticks: int,
        dealer_idx: int,
        current_turn: int,
        dora_indicators: list[PyTileName],
        remaining_wall_tiles: int,
        scores: list[int],
        is_riichi: list[bool],
        hands: list[list[PyTileName]],
        melds: list[list[PyMeld]],
        rivers: list[list[PyTileName]],
    ) -> None: ...
    def to_dict(self, reveal_all: bool = False) -> dict[str, Any]: ...

def py_evaluate_placement_discards(
    tiles: list[PyTileName],
    match_context: PyMatchContext,
    player_idx: int = 0,
    is_dealer: Optional[bool] = None,
    dora_indicators: Optional[list[PyTileName]] = None,
    turn_number: Optional[int] = None,
    remaining_wall_tiles: Optional[int] = None,
    seat_wind: Optional[PyTileName] = None,
    visible_tiles: Optional[list[PyTileName]] = None,
) -> list[PyPlacementEvaluation]:
    """Evaluates all discards taking placement expectations and rank points into account."""
    ...

def py_calculate_orasu_conditions(
    match_context: PyMatchContext, player_idx: int = 0
) -> list[PyWinCondition]:
    """Calculates win condition requirements for orasu."""
    ...

def py_get_ai_hud_data(
    tiles: list[PyTileName],
    match_context: PyMatchContext,
    player_idx: int = 0,
    is_dealer: Optional[bool] = None,
    dora_indicators: Optional[list[PyTileName]] = None,
    turn_number: Optional[int] = None,
    remaining_wall_tiles: Optional[int] = None,
    seat_wind: Optional[PyTileName] = None,
    visible_tiles: Optional[list[PyTileName]] = None,
) -> dict[str, Any]:
    """Provides a unified dictionary with all HUD cards and AI metrics for UI rendering."""
    ...
