#[allow(dead_code)]
pub const TILE_NAME_NUMBER: usize = 34;
pub const TILE_PER_KIND: usize = 4;
pub const TILE_WALL_CAPACITY: usize = TILE_NAME_NUMBER * TILE_PER_KIND;

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
#[repr(u8)]
/// 牌の種類（名前）を表す列挙型です。
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
    #[inline(always)]
    #[allow(dead_code)]
    pub const fn from_usize(n: usize) -> TileName {
        if n <= 34 {
            // SAFETY: TileName は #[repr(u8)] で定義されており、
            // 0 から 34 のすべての値に対して有効なバリアント (None=0, OneM..=White=1..=34) が
            // 連続して定義されているため、transmute は健全です。
            unsafe { std::mem::transmute::<u8, TileName>(n as u8) }
        } else {
            TileName::None
        }
    }

    #[inline]
    #[allow(dead_code)]
    pub fn as_str(&self) -> &'static str {
        const TILE_STRS: [&str; 35] = [
            " ", "1m", "2m", "3m", "4m", "5m", "6m", "7m", "8m", "9m", "1p", "2p", "3p", "4p",
            "5p", "6p", "7p", "8p", "9p", "1s", "2s", "3s", "4s", "5s", "6s", "7s", "8s", "9s",
            "東", "南", "西", "北", "中", "発", "白",
        ];
        let idx = *self as usize;
        if idx < TILE_STRS.len() {
            TILE_STRS[idx]
        } else {
            " "
        }
    }

    #[inline]
    pub const fn tile_type(&self) -> TileType {
        let idx = *self as u8;
        match idx {
            1..=9 => TileType::Characters,
            10..=18 => TileType::Circles,
            19..=27 => TileType::Bamboos,
            28..=31 => TileType::Winds,
            32..=34 => TileType::Dragons,
            _ => TileType::None,
        }
    }

    #[inline]
    pub const fn category(&self) -> TileCategory {
        let idx = *self as u8;
        match idx {
            1..=27 => TileCategory::Simples,
            28..=34 => TileCategory::Honors,
            _ => TileCategory::None,
        }
    }
}

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[repr(u8)]
/// 牌の種類（数牌の各スーツ、風牌、三元牌）を表す列挙型です。
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
#[repr(u8)]
/// 牌のカテゴリ（数牌か字牌か）を表す列挙型です。
pub enum TileCategory {
    None = 0,
    Simples, /* 数牌      */
    Honors,  /* 字牌      */
}

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
/// 牌を表す構造体です。
pub struct Tile {
    name: TileName,
}

impl Tile {
    #[inline]
    pub const fn new(name: TileName) -> Self {
        Self { name }
    }

    #[inline]
    pub const fn name(self) -> TileName {
        self.name
    }

    #[inline]
    pub const fn tile_type(self) -> TileType {
        self.name.tile_type()
    }

    #[inline]
    pub const fn category(self) -> TileCategory {
        self.name.category()
    }
}
