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
}
