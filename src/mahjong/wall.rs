use rand::seq::SliceRandom;
use rand::Rng;

use crate::tile::{TileName, TILE_NAME_NUMBER, TILE_PER_KIND, TILE_WALL_CAPACITY};

#[derive(Clone, Debug)]
/// 山（壁）を表す構造体です。
pub struct Wall {
    tiles: [TileName; TILE_WALL_CAPACITY],
    cursor: usize,
    kan_count: usize,
}

impl Wall {
    pub fn new() -> Self {
        let mut tiles = [TileName::None; TILE_WALL_CAPACITY];

        for name_index in 1..=TILE_NAME_NUMBER {
            let tile_name = TileName::from_u8(name_index as u8);
            let offset = (name_index - 1) * TILE_PER_KIND;
            tiles[offset..offset + TILE_PER_KIND].fill(tile_name);
        }

        Self {
            tiles,
            cursor: 0,
            kan_count: 0,
        }
    }

    /// 牌山をシャッフルし、ツモの位置を初期化します。
    pub fn shuffle<R: Rng + ?Sized>(&mut self, rng: &mut R) {
        self.tiles.shuffle(rng);
        self.cursor = 0;
        self.kan_count = 0;
    }

    /// 通常のツモを行います。
    /// 王牌（ワンパイ）の14枚を除いた残りの山から牌を引きます。
    pub fn draw(&mut self) -> Option<TileName> {
        let limit = TILE_WALL_CAPACITY - 14 - self.kan_count;
        if self.cursor >= limit {
            return None;
        }

        let tile = self.tiles.get(self.cursor).copied();
        if tile.is_some() {
            self.cursor += 1;
        }
        tile
    }

    /// 嶺上牌が引けるか（カンが4回未満か）を返します。
    pub fn can_draw_replacement(&self) -> bool {
        self.kan_count < 4
    }

    /// カンが発生した際に、嶺上牌（リンシャンハイ）を引きます。
    pub fn draw_replacement(&mut self) -> Option<TileName> {
        if self.kan_count >= 4 {
            return None;
        }

        let tile = self
            .tiles
            .get(TILE_WALL_CAPACITY - 1 - self.kan_count)
            .copied();
        if tile.is_some() {
            self.kan_count += 1;
        }
        tile
    }

    /// 山の残り枚数を返します。
    pub fn remaining(&self) -> usize {
        let limit = TILE_WALL_CAPACITY - 14 - self.kan_count;
        limit.saturating_sub(self.cursor)
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
