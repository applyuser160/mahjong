#[path = "mahjong/tile.rs"]
pub mod tile;

#[path = "mahjong/hand.rs"]
pub mod hand;

#[path = "mahjong/round.rs"]
pub mod round;

#[path = "mahjong/wall.rs"]
pub mod wall;

use pyo3::prelude::*;
use rand::rngs::StdRng;
use rand::SeedableRng;

pub use round::{Round, PLAYER_NUMBER};
pub use tile::{
    Tile, TileCategory, TileName, TileType, TILE_NAME_NUMBER, TILE_PER_KIND, TILE_WALL_CAPACITY,
};
pub use wall::Wall;

#[pyfunction]
pub fn play_once(seed: u64) -> PyResult<Vec<&'static str>> {
    let mut wall = Wall::new();
    let mut rng = StdRng::seed_from_u64(seed);
    wall.shuffle(&mut rng);

    let mut round = Round::new(wall);
    let mut discards = Vec::new();

    while let Some(tile) = round.play_turn(0) {
        discards.push(tile.as_str());
    }

    Ok(discards)
}

#[pymodule]
fn mahjong(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(play_once, m)?)?;
    Ok(())
}
