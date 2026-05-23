use crate::tile::TileName;

#[derive(Debug, Clone, Default)]
pub struct River {
    tiles: Vec<TileName>,
}

impl River {
    pub fn new() -> Self {
        Self { tiles: Vec::new() }
    }

    pub fn tiles(&self) -> &[TileName] {
        &self.tiles
    }

    pub fn push(&mut self, tile: TileName) {
        self.tiles.push(tile);
    }
}
