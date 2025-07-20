use crate::tile::{Tile, TileName, TILE_NAME_NUMBER};
use rand::seq::SliceRandom;
use rand::thread_rng;

#[allow(dead_code)]
pub const SAME_TILE_NUMBER: usize = 4;

#[allow(dead_code)]
pub const RED_TILE: [u8; 3] = [5, 14, 23];

#[allow(dead_code)]
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Wall {
    pub tiles: Vec<Tile>,
}

impl Wall {
    #[allow(dead_code)]
    pub fn new() -> Self {
        let mut tiles = Vec::new();
        for n in 1..=TILE_NAME_NUMBER {
            for _i in 0..4 {
                let name = TileName::from_usize(n);
                let tile = Tile::from_name(name, false);
                tiles.push(tile);
            }
        }
        Self { tiles }
    }

    #[allow(dead_code)]
    pub fn from(is_red: bool) -> Self {
        let mut tiles = Vec::new();
        for n in 1..=TILE_NAME_NUMBER {
            for i in 0..4 {
                let is_red_value = RED_TILE.contains(&(n as u8)) && i == 0 && is_red;
                let name = TileName::from_usize(n);
                let tile = Tile::from_name(name, is_red_value);
                tiles.push(tile);
            }
        }
        Self { tiles }
    }

    #[allow(dead_code)]
    pub fn from_custom(tile_numbers: [u8; TILE_NAME_NUMBER], is_red: bool) -> Self {
        let mut tiles = Vec::new();
        for (i, n) in tile_numbers.iter().enumerate() {
            for _ in 0..*n {
                let is_red_value = RED_TILE.contains(&(i as u8)) && is_red;
                let name = TileName::from_usize(i + 1); // iは0から始まるため、1を足して正しいタイル番号を取得
                let tile = Tile::from_name(name, is_red_value);
                tiles.push(tile);
            }
        }
        Self { tiles }
    }

    #[allow(dead_code)]
    pub fn shuffle(&mut self) {
        let mut rng = thread_rng();
        self.tiles.shuffle(&mut rng);
    }
}
