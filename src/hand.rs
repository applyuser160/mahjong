use crate::tile::TileName;

#[derive(Clone, Copy, Debug)]
pub struct Hand {
    tiles: [TileName; 14],
    len: usize,
}

impl Hand {
    pub const fn new() -> Self {
        Self {
            tiles: [TileName::None; 14],
            len: 0,
        }
    }

    pub fn tiles(&self) -> &[TileName] {
        &self.tiles[..self.len]
    }

    pub fn push(&mut self, tile: TileName) {
        assert!(self.len < self.tiles.len());
        self.tiles[self.len] = tile;
        self.len += 1;
    }

    pub fn discard(&mut self, index: usize) -> TileName {
        assert!(index < self.len);
        let removed = self.tiles[index];
        self.tiles.copy_within(index + 1..self.len, index);
        self.len -= 1;
        removed
    }
}
