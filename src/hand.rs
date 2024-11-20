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
}
