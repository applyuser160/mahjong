use crate::tile::TileName;
use arrayvec::ArrayVec;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// 鳴き（副露）の種類を表す列挙型です。
pub enum Meld {
    Chii {
        called: TileName,
        consumed: [TileName; 2],
    },
    Pon(TileName),
    Daiminkan(TileName),
    Ankan(TileName),
    Kakan(TileName),
}

impl Meld {
    /// この副露が門前を崩す公開副露（チー・ポン・大明槓・加槓）かどうかを判定します。
    /// 暗槓（Ankan）は手牌内で完結し門前を崩さないため false を返します。
    #[inline]
    pub fn is_open(&self) -> bool {
        !matches!(self, Meld::Ankan(_))
    }
}

/// 指定された面子リストに門前を崩す副露が含まれているかを判定します。
#[inline]
pub fn has_open_meld(melds: &[Meld]) -> bool {
    melds.iter().any(|m| m.is_open())
}

/// 指定された面子リストが門前清（暗槓のみ、または副露なし）かどうかを判定します。
#[inline]
pub fn is_menzen(melds: &[Meld]) -> bool {
    !has_open_meld(melds)
}

#[derive(Clone, Debug)]
/// プレイヤーの手牌を表す構造体です。
pub struct Hand {
    tiles: [TileName; 14],
    len: usize,
    pub open_melds: ArrayVec<Meld, 4>,
    pub counts: [u8; 35],
}

impl Default for Hand {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl Hand {
    /// 手牌が門前清（暗槓のみ、または副露なし）かどうかを判定します。
    #[inline]
    pub fn is_menzen(&self) -> bool {
        is_menzen(&self.open_melds)
    }

    /// 手牌に門前を崩す副露（チー・ポン・大明槓・加槓）があるかを判定します。
    #[inline]
    pub fn has_open_meld(&self) -> bool {
        has_open_meld(&self.open_melds)
    }

    #[inline]
    pub const fn new() -> Self {
        Self {
            tiles: [TileName::None; 14],
            len: 0,
            open_melds: ArrayVec::new_const(),
            counts: [0; 35],
        }
    }

    #[inline]
    pub fn tiles(&self) -> &[TileName] {
        &self.tiles[..self.len]
    }

    #[inline]
    pub fn push(&mut self, tile: TileName) {
        self.tiles[self.len] = tile;
        self.len += 1;
        self.counts[tile as usize] += 1;
    }

    #[inline]
    pub fn discard(&mut self, index: usize) -> Result<TileName, &'static str> {
        if index >= self.len {
            return Err("Index out of bounds");
        }
        let removed = self.tiles[index];
        self.tiles.copy_within(index + 1..self.len, index);
        self.len -= 1;
        self.counts[removed as usize] -= 1;
        Ok(removed)
    }

    #[inline]
    pub fn call_meld(&mut self, meld: Meld) -> Result<(), &'static str> {
        let mut consumed_from_hand = [TileName::None; 4];
        let consumed_len;

        match meld {
            Meld::Chii { called, consumed } => {
                let info = crate::yaku::is_number_tile(called).ok_or("Invalid tile for Chii")?;

                let mut ranks = [info.1, 0, 0];
                let mut i = 1;
                for &c in &consumed {
                    let consumed_info =
                        crate::yaku::is_number_tile(c).ok_or("Invalid consumed tile for Chii")?;
                    if consumed_info.0 != info.0 {
                        return Err("Cross-suit Chii is not allowed");
                    }
                    ranks[i] = consumed_info.1;
                    i += 1;
                }

                ranks.sort_unstable();
                if ranks[1] != ranks[0] + 1 || ranks[2] != ranks[1] + 1 {
                    return Err("Tiles do not form a consecutive sequence");
                }

                consumed_from_hand[0] = consumed[0];
                consumed_from_hand[1] = consumed[1];
                consumed_len = 2;
            }
            Meld::Pon(tile) => {
                consumed_from_hand[0] = tile;
                consumed_from_hand[1] = tile;
                consumed_len = 2;
            }
            Meld::Daiminkan(tile) => {
                consumed_from_hand[0] = tile;
                consumed_from_hand[1] = tile;
                consumed_from_hand[2] = tile;
                consumed_len = 3;
            }
            Meld::Ankan(tile) => {
                consumed_from_hand[0] = tile;
                consumed_from_hand[1] = tile;
                consumed_from_hand[2] = tile;
                consumed_from_hand[3] = tile;
                consumed_len = 4;
            }
            Meld::Kakan(tile) => {
                // 対応するポンがすでに存在している必要があります
                if !self.open_melds.contains(&Meld::Pon(tile)) {
                    return Err("Cannot call Kakan without an existing Pon");
                }
                consumed_from_hand[0] = tile;
                consumed_len = 1;
            }
        }

        // 手牌に必要な牌が含まれているか、重複を考慮して検証します
        let mut available_counts = self.counts;
        for &t in &consumed_from_hand[..consumed_len] {
            let idx = t as usize;
            if available_counts[idx] > 0 {
                available_counts[idx] -= 1;
            } else {
                return Err("Missing required tiles in hand to call meld");
            }
        }

        // 消費された牌を手牌から取り除きます
        for &t in &consumed_from_hand[..consumed_len] {
            if let Some(pos) = self.tiles[..self.len].iter().position(|&x| x == t) {
                self.discard(pos)?;
            }
        }

        if let Meld::Kakan(tile) = meld {
            if let Some(pos) = self.open_melds.iter().position(|&m| m == Meld::Pon(tile)) {
                self.open_melds[pos] = meld;
            }
        } else {
            if self.open_melds.len() >= 4 {
                return Err("Cannot have more than 4 melds");
            }
            self.open_melds.push(meld);
        }

        Ok(())
    }
}
