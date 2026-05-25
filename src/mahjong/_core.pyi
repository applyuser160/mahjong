from enum import Enum
from typing import Optional

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

class PyMeld:
    """Represents a Mahjong meld (Chii, Pon, Kan)."""
    @staticmethod
    def chii(called: PyTileName, consumed: list[PyTileName]) -> 'PyMeld': ...
    @staticmethod
    def pon(tile: PyTileName) -> 'PyMeld': ...
    @staticmethod
    def daiminkan(tile: PyTileName) -> 'PyMeld': ...
    @staticmethod
    def ankan(tile: PyTileName) -> 'PyMeld': ...
    @staticmethod
    def kakan(tile: PyTileName) -> 'PyMeld': ...
    @property
    def kind(self) -> str: ...
    @property
    def tiles(self) -> list[PyTileName]: ...

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

class PyRiver:
    """Represents a player's discard river."""
    def __init__(self) -> None: ...
    @property
    def tiles(self) -> list[PyTileName]: ...

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
    tiles: list[PyTileName],
    melds: list[PyMeld],
    context: PyWinContext
) -> list[PyYakuId]:
    """Judges the Yaku present in the given hand and context."""
    ...