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

    /// 順子を探す
    #[allow(dead_code)]
    pub fn get_chows(&self) -> Vec<usize> {
        let deltas = self.get_delta();
        let mut result: Vec<usize> = Vec::new();

        for i in 0..deltas.len() - 1 {
            if deltas[i] == 1 && deltas[i + 1] == 1 {
                result.push(i);
            }
        }

        result
    }

    /// 刻子を探す
    #[allow(dead_code)]
    pub fn get_pungs(&self) -> Vec<usize> {
        let deltas = self.get_delta();
        let mut result: Vec<usize> = Vec::new();

        for i in 0..deltas.len() - 1 {
            if deltas[i] == 0 && deltas[i + 1] == 0 {
                result.push(i);
            }
        }

        result
    }

    /// カンを探す
    #[allow(dead_code)]
    pub fn get_kongs(&self) -> Vec<usize> {
        let deltas = self.get_delta();
        let mut result: Vec<usize> = Vec::new();

        for i in 0..deltas.len() - 2 {
            if deltas[i] == 0 && deltas[i + 1] == 0 && deltas[i + 2] == 0 {
                result.push(i);
            }
        }

        result
    }

    /// 対子を探す
    #[allow(dead_code)]
    pub fn get_pairs(&self) -> Vec<usize> {
        let deltas = self.get_delta();
        let mut result: Vec<usize> = Vec::new();

        for i in 0..deltas.len() - 1 {
            if deltas[i] == 0 {
                result.push(i);
            }
        }

        result
    }
}
