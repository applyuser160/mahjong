use crate::tile::TileName;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

#[derive(Clone, Debug)]
pub struct Hand {
    tiles: [TileName; 14],
    len: usize,
    pub open_melds: Vec<Meld>,
}

impl Default for Hand {
    fn default() -> Self {
        Self::new()
    }
}

impl Hand {
    pub const fn new() -> Self {
        Self {
            tiles: [TileName::None; 14],
            len: 0,
            open_melds: Vec::new(),
        }
    }

    pub fn tiles(&self) -> &[TileName] {
        &self.tiles[..self.len]
    }

    pub fn push(&mut self, tile: TileName) {
        self.tiles[self.len] = tile;
        self.len += 1;
    }

    pub fn discard(&mut self, index: usize) -> TileName {
        let removed = self.tiles[index];
        self.tiles.copy_within(index + 1..self.len, index);
        self.len -= 1;
        removed
    }

    pub fn call_meld(&mut self, meld: Meld) -> Result<(), &'static str> {
        let consumed_from_hand: Vec<TileName> = match meld {
            Meld::Chii { called, consumed } => {
                let info = crate::yaku::is_number_tile(called).ok_or("Invalid tile for Chii")?;

                let mut ranks = vec![info.1];
                for &c in &consumed {
                    let consumed_info =
                        crate::yaku::is_number_tile(c).ok_or("Invalid consumed tile for Chii")?;
                    if consumed_info.0 != info.0 {
                        return Err("Cross-suit Chii is not allowed");
                    }
                    ranks.push(consumed_info.1);
                }

                ranks.sort_unstable();
                if ranks[1] != ranks[0] + 1 || ranks[2] != ranks[1] + 1 {
                    return Err("Tiles do not form a consecutive sequence");
                }
                consumed.to_vec()
            }
            Meld::Pon(tile) => vec![tile, tile],
            Meld::Daiminkan(tile) => vec![tile, tile, tile],
            Meld::Ankan(tile) => vec![tile, tile, tile, tile],
            Meld::Kakan(tile) => {
                // Must have a corresponding Pon
                if !self.open_melds.contains(&Meld::Pon(tile)) {
                    return Err("Cannot call Kakan without an existing Pon");
                }
                vec![tile]
            }
        };

        // Verify that we have the consumed tiles, accounting for duplicates
        let mut available_tiles = self.tiles[..self.len].to_vec();
        for &t in &consumed_from_hand {
            if let Some(pos) = available_tiles.iter().position(|&x| x == t) {
                available_tiles.remove(pos);
            } else {
                return Err("Missing required tiles in hand to call meld");
            }
        }

        // Remove the consumed tiles
        for &t in &consumed_from_hand {
            if let Some(pos) = self.tiles[..self.len].iter().position(|&x| x == t) {
                self.discard(pos);
            }
        }

        if let Meld::Kakan(tile) = meld {
            if let Some(pos) = self.open_melds.iter().position(|&m| m == Meld::Pon(tile)) {
                self.open_melds[pos] = meld;
            }
        } else {
            self.open_melds.push(meld);
        }

        Ok(())
    }
}
