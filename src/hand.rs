use crate::tile::Tile;

#[allow(dead_code)]
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Hand {
    pub tiles: Vec<Tile>,
}

impl Hand {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self { tiles: Vec::new() }
    }

    #[allow(dead_code)]
    pub fn push_tile(&mut self, tile: Tile) {
        self.tiles.push(tile);
    }

    #[allow(dead_code)]
    pub fn pop_tile(&mut self, index: usize) {
        self.tiles.remove(index);
    }

    #[allow(dead_code)]
    pub fn sort(&mut self) {
        self.tiles.sort_by(|a, b| a.name.cmp(&b.name));
    }

    #[allow(dead_code)]
    pub fn get_delta(&self) -> Vec<u8> {
        let mut result: Vec<u8> = Vec::new();

        for i in 0..self.tiles.len() - 1 {
            let delta = (self.tiles[i + 1].name as u8) - (self.tiles[i].name as u8);
            result.push(delta);
        }

        result
    }
}
