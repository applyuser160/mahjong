use rand::seq::SliceRandom;
use rand::Rng;

use crate::tile::{TileName, TILE_NAME_NUMBER, TILE_PER_KIND, TILE_WALL_CAPACITY};

#[derive(Clone, Debug)]
pub struct Wall {
    tiles: [TileName; TILE_WALL_CAPACITY],
    cursor: usize,
}

impl Wall {
    pub fn new() -> Self {
        let mut tiles = [TileName::None; TILE_WALL_CAPACITY];

        for name_index in 1..=TILE_NAME_NUMBER {
            let tile_name = TileName::from_usize(name_index);
            let offset = (name_index - 1) * TILE_PER_KIND;
            tiles[offset..offset + TILE_PER_KIND].fill(tile_name);
        }

        Self { tiles, cursor: 0 }
    }

    pub fn shuffle<R: Rng + ?Sized>(&mut self, rng: &mut R) {
        self.tiles.shuffle(rng);
        self.cursor = 0;
    }

    pub fn draw(&mut self) -> Option<TileName> {
        let tile = self.tiles.get(self.cursor).copied();
        if tile.is_some() {
            self.cursor += 1;
        }
        tile
    }

    pub fn remaining(&self) -> usize {
        TILE_WALL_CAPACITY.saturating_sub(self.cursor)
    }

    pub fn tiles(&self) -> &[TileName] {
        &self.tiles
    }
}

impl Default for Wall {
    fn default() -> Self {
        Self::new()
    }
}
