use crate::tile::TileName;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Meld {
    Chii(TileName),
    Pon(TileName),
    Kan(TileName),
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

    pub fn call_meld(
        &mut self,
        meld: Meld,
        consumed_from_hand: &[TileName],
    ) -> Result<(), &'static str> {
        if let Meld::Chii(tile) = meld {
            let info = crate::yaku::is_number_tile(tile).ok_or("Invalid tile for Chii")?;
            if info.1 > 7 {
                return Err("Chii sequence exceeds rank 9");
            }

            if consumed_from_hand.len() != 2 {
                return Err("Chii requires exactly 2 consumed tiles");
            }

            let mut ranks = vec![info.1];
            for &consumed in consumed_from_hand {
                let consumed_info = crate::yaku::is_number_tile(consumed)
                    .ok_or("Invalid consumed tile for Chii")?;
                if consumed_info.0 != info.0 {
                    return Err("Cross-suit Chii is not allowed");
                }
                ranks.push(consumed_info.1);
            }

            ranks.sort_unstable();
            if ranks[1] != ranks[0] + 1 || ranks[2] != ranks[1] + 1 {
                return Err("Tiles do not form a consecutive sequence");
            }
        }

        // Verify that we have the consumed tiles, accounting for duplicates
        let mut available_tiles = self.tiles[..self.len].to_vec();
        for &t in consumed_from_hand {
            if let Some(pos) = available_tiles.iter().position(|&x| x == t) {
                available_tiles.remove(pos);
            } else {
                return Err("Missing required tiles in hand to call meld");
            }
        }

        // Remove the consumed tiles
        for &t in consumed_from_hand {
            if let Some(pos) = self.tiles[..self.len].iter().position(|&x| x == t) {
                self.discard(pos);
            }
        }

        self.open_melds.push(meld);
        Ok(())
    }
}
