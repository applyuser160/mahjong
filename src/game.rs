use crate::hand::Hand;
use crate::wall::Wall;

#[allow(dead_code)]
pub const PLAYER_COUNT: usize = 4;

#[allow(dead_code)]
pub struct Game {
    pub wall: Wall,
    pub hands: [Hand; PLAYER_COUNT],
    pub discards: [Vec<Tile>; PLAYER_COUNT],
    pub current_player: usize,
    pub turn: usize,
}

use crate::tile::Tile;

impl Game {
    #[allow(dead_code)]
    pub fn new(is_red: bool) -> Self {
        let mut wall = Wall::from(is_red);
        wall.shuffle();

        let mut hands = [Hand::new(), Hand::new(), Hand::new(), Hand::new()];
        for i in 0..PLAYER_COUNT {
            for _ in 0..13 {
                if let Some(tile) = wall.tiles.pop() {
                    hands[i].tiles.push(tile);
                }
            }
        }

        Self {
            wall,
            hands,
            discards: [Vec::new(), Vec::new(), Vec::new(), Vec::new()],
            current_player: 0,
            turn: 0,
        }
    }

    #[allow(dead_code)]
    pub fn draw_tile(&mut self) -> Option<Tile> {
        self.wall.tiles.pop()
    }

    #[allow(dead_code)]
    pub fn discard_tile(&mut self, player_index: usize, tile_index: usize) {
        if self.hands[player_index].tiles.len() > tile_index {
            let tile = self.hands[player_index].tiles.remove(tile_index);
            self.discards[player_index].push(tile);
        }
    }
}
