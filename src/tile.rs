#[allow(dead_code)]
pub const TILE_NAME_NUMBER: usize = 34;

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
    pub fn to_string(&self) -> String {
        match self {
            TileName::None => String::from(" "),
            TileName::OneM => String::from("1m"),
            TileName::TwoM => String::from("2m"),
            TileName::ThreeM => String::from("3m"),
            TileName::FourM => String::from("4m"),
            TileName::FiveM => String::from("5m"),
            TileName::SixM => String::from("6m"),
            TileName::SevenM => String::from("7m"),
            TileName::EightM => String::from("8m"),
            TileName::NineM => String::from("9m"),
            TileName::OneP => String::from("1p"),
            TileName::TwoP => String::from("2p"),
            TileName::ThreeP => String::from("3p"),
            TileName::FourP => String::from("4p"),
            TileName::FiveP => String::from("5p"),
            TileName::SixP => String::from("6p"),
            TileName::SevenP => String::from("7p"),
            TileName::EightP => String::from("8p"),
            TileName::NineP => String::from("9p"),
            TileName::OneS => String::from("1s"),
            TileName::TwoS => String::from("2s"),
            TileName::ThreeS => String::from("3s"),
            TileName::FourS => String::from("4s"),
            TileName::FiveS => String::from("5s"),
            TileName::SixS => String::from("6s"),
            TileName::SevenS => String::from("7s"),
            TileName::EightS => String::from("8s"),
            TileName::NineS => String::from("9s"),
            TileName::East => String::from("東"),
            TileName::South => String::from("南"),
            TileName::West => String::from("西"),
            TileName::North => String::from("北"),
            TileName::Red => String::from("中"),
            TileName::Green => String::from("発"),
            TileName::White => String::from("白"),
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

impl TileType {
    #[allow(dead_code)]
    pub fn from_tile_name(name: TileName) -> Self {
        if name == TileName::None {
            return Self::None;
        } else if name <= TileName::NineM {
            Self::Characters
        } else if name <= TileName::NineP {
            Self::Circles
        } else if name <= TileName::NineS {
            Self::Bamboos
        } else if name <= TileName::North {
            Self::Winds
        } else if name <= TileName::White {
            Self::Dragons
        } else {
            Self::None
        }
    }
}

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[repr(usize)]
pub enum TileCategory {
    None = 0,
    Sinples, /* 数牌      */
    Honors,  /* 字牌      */
}

impl TileCategory {
    #[allow(dead_code)]
    pub fn from_tile_name(name: TileName) -> Self {
        if name == TileName::None {
            return Self::None;
        } else if name <= TileName::NineS {
            Self::Sinples
        } else if name <= TileName::White {
            Self::Honors
        } else {
            Self::None
        }
    }

    #[allow(dead_code)]
    pub fn from_tile_type(tile_type: TileType) -> Self {
        if tile_type == TileType::None {
            return Self::None;
        } else if tile_type <= TileType::Bamboos {
            Self::Sinples
        } else if tile_type <= TileType::Dragons {
            Self::Honors
        } else {
            Self::None
        }
    }
}

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Tile {
    pub name: TileName,
    pub tile_type: TileType,
    pub category: TileCategory,
    pub is_red: bool,
}

impl Tile {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            name: TileName::None,
            tile_type: TileType::None,
            category: TileCategory::None,
            is_red: false,
        }
    }

    #[allow(dead_code)]
    pub fn from(name: TileName, tile_type: TileType, category: TileCategory, is_red: bool) -> Self {
        Self {
            name,
            tile_type,
            category,
            is_red,
        }
    }

    #[allow(dead_code)]
    pub fn to_u8(&self) -> u8 {
        let mut result = self.name as u8;
        result += (self.is_red as u8) << 7;
        return result;
    }
}
