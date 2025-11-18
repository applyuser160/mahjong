#[allow(dead_code)]
pub const TILE_NAME_NUMBER: usize = 34;
pub const TILE_PER_KIND: usize = 4;
pub const TILE_WALL_CAPACITY: usize = TILE_NAME_NUMBER * TILE_PER_KIND;

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[repr(usize)]
pub enum TileName {
    None = 0,

    /* 萬子   　*/ OneM, /* 1m */
    /*         */ TwoM, /* 2m */
    /*         */ ThreeM, /* 3m */
    /*         */ FourM, /* 4m */
    /*         */ FiveM, /* 5m */
    /*         */ SixM, /* 6m */
    /*         */ SevenM, /* 7m */
    /*         */ EightM, /* 8m */
    /*         */ NineM, /* 9m */

    /* 筒子   　*/ OneP, /* 1p */
    /*         */ TwoP, /* 2p */
    /*         */ ThreeP, /* 3p */
    /*         */ FourP, /* 4p */
    /*         */ FiveP, /* 5p */
    /*         */ SixP, /* 6p */
    /*         */ SevenP, /* 7p */
    /*         */ EightP, /* 8p */
    /*         */ NineP, /* 9p */

    /* 索子   　*/ OneS, /* 1s */
    /*         */ TwoS, /* 2s */
    /*         */ ThreeS, /* 3s */
    /*         */ FourS, /* 4s */
    /*         */ FiveS, /* 5s */
    /*         */ SixS, /* 6s */
    /*         */ SevenS, /* 7s */
    /*         */ EightS, /* 8s */
    /*         */ NineS, /* 9s */

    /* 風牌   　*/ East, /* 東 */
    /*         */ South, /* 南 */
    /*         */ West, /* 西 */
    /*         */ North, /* 北 */

    /* 三元牌   */ Red, /* 中 */
    /*         */ Green, /* 発 */
    /*         */ White, /* 白 */
}

impl TileName {
    #[allow(dead_code)]
    pub fn from_usize(n: usize) -> TileName {
        match n {
            0 => TileName::None,
            1 => TileName::OneM,
            2 => TileName::TwoM,
            3 => TileName::ThreeM,
            4 => TileName::FourM,
            5 => TileName::FiveM,
            6 => TileName::SixM,
            7 => TileName::SevenM,
            8 => TileName::EightM,
            9 => TileName::NineM,
            10 => TileName::OneP,
            11 => TileName::TwoP,
            12 => TileName::ThreeP,
            13 => TileName::FourP,
            14 => TileName::FiveP,
            15 => TileName::SixP,
            16 => TileName::SevenP,
            17 => TileName::EightP,
            18 => TileName::NineP,
            19 => TileName::OneS,
            20 => TileName::TwoS,
            21 => TileName::ThreeS,
            22 => TileName::FourS,
            23 => TileName::FiveS,
            24 => TileName::SixS,
            25 => TileName::SevenS,
            26 => TileName::EightS,
            27 => TileName::NineS,
            28 => TileName::East,
            29 => TileName::South,
            30 => TileName::West,
            31 => TileName::North,
            32 => TileName::Red,
            33 => TileName::Green,
            34 => TileName::White,
            _ => TileName::None,
        }
    }

    #[allow(dead_code)]
    pub fn as_str(&self) -> &'static str {
        match self {
            TileName::None => " ",
            TileName::OneM => "1m",
            TileName::TwoM => "2m",
            TileName::ThreeM => "3m",
            TileName::FourM => "4m",
            TileName::FiveM => "5m",
            TileName::SixM => "6m",
            TileName::SevenM => "7m",
            TileName::EightM => "8m",
            TileName::NineM => "9m",
            TileName::OneP => "1p",
            TileName::TwoP => "2p",
            TileName::ThreeP => "3p",
            TileName::FourP => "4p",
            TileName::FiveP => "5p",
            TileName::SixP => "6p",
            TileName::SevenP => "7p",
            TileName::EightP => "8p",
            TileName::NineP => "9p",
            TileName::OneS => "1s",
            TileName::TwoS => "2s",
            TileName::ThreeS => "3s",
            TileName::FourS => "4s",
            TileName::FiveS => "5s",
            TileName::SixS => "6s",
            TileName::SevenS => "7s",
            TileName::EightS => "8s",
            TileName::NineS => "9s",
            TileName::East => "東",
            TileName::South => "南",
            TileName::West => "西",
            TileName::North => "北",
            TileName::Red => "中",
            TileName::Green => "発",
            TileName::White => "白",
        }
    }

    pub const fn tile_type(&self) -> TileType {
        match self {
            TileName::OneM
            | TileName::TwoM
            | TileName::ThreeM
            | TileName::FourM
            | TileName::FiveM
            | TileName::SixM
            | TileName::SevenM
            | TileName::EightM
            | TileName::NineM => TileType::Characters,
            TileName::OneP
            | TileName::TwoP
            | TileName::ThreeP
            | TileName::FourP
            | TileName::FiveP
            | TileName::SixP
            | TileName::SevenP
            | TileName::EightP
            | TileName::NineP => TileType::Circles,
            TileName::OneS
            | TileName::TwoS
            | TileName::ThreeS
            | TileName::FourS
            | TileName::FiveS
            | TileName::SixS
            | TileName::SevenS
            | TileName::EightS
            | TileName::NineS => TileType::Bamboos,
            TileName::East | TileName::South | TileName::West | TileName::North => TileType::Winds,
            TileName::Red | TileName::Green | TileName::White => TileType::Dragons,
            TileName::None => TileType::None,
        }
    }

    pub const fn category(&self) -> TileCategory {
        match self.tile_type() {
            TileType::Characters | TileType::Circles | TileType::Bamboos => TileCategory::Simples,
            TileType::Winds | TileType::Dragons => TileCategory::Honors,
            TileType::None => TileCategory::None,
        }
    }
}

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[repr(usize)]
pub enum TileType {
    None = 0,
    Characters, /* 萬子      */
    Circles,    /* 筒子      */
    Bamboos,    /* 索子      */
    Winds,      /* 風牌      */
    Dragons,    /* 三元牌    */
}

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[repr(usize)]
pub enum TileCategory {
    None = 0,
    Simples, /* 数牌      */
    Honors,  /* 字牌      */
}

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Tile {
    name: TileName,
}

impl Tile {
    pub const fn new(name: TileName) -> Self {
        Self { name }
    }

    pub const fn name(self) -> TileName {
        self.name
    }

    pub const fn tile_type(self) -> TileType {
        self.name.tile_type()
    }

    pub const fn category(self) -> TileCategory {
        self.name.category()
    }

    pub const fn is_red(self) -> bool {
        matches!(self.name, TileName::Red | TileName::Green | TileName::White)
    }
}
