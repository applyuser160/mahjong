#[path = "mahjong/tile.rs"]
pub mod tile;

#[path = "mahjong/hand.rs"]
pub mod hand;

#[path = "mahjong/round.rs"]
pub mod round;

#[path = "mahjong/wall.rs"]
pub mod wall;

#[path = "mahjong/yaku.rs"]
pub mod yaku;

#[path = "mahjong/river.rs"]
pub mod river;

use pyo3::prelude::*;
use rand::rngs::StdRng;
use rand::SeedableRng;

pub use round::{Round, PLAYER_NUMBER};
pub use tile::{
    Tile, TileCategory, TileName, TileType, TILE_NAME_NUMBER, TILE_PER_KIND, TILE_WALL_CAPACITY,
};
pub use wall::Wall;
pub use yaku::{judge_yaku, WinContext, Yaku, YakuId, ALL_YAKU};

pub mod python_api;

#[pyfunction]
pub fn play_once(seed: u64) -> PyResult<Vec<&'static str>> {
    let mut wall = Wall::new();
    let mut rng = StdRng::seed_from_u64(seed);
    wall.shuffle(&mut rng);

    let mut round = Round::new(wall);
    let mut discards = Vec::new();

    while round.draw_tile().is_some() {
        if let Ok(tile) = round.discard_tile(0) {
            discards.push(tile.as_str());
        } else {
            break;
        }
    }

    Ok(discards)
}

#[pymodule]
fn mahjong(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(play_once, m)?)?;
    m.add_class::<python_api::PyTileType>()?;
    m.add_class::<python_api::PyTileCategory>()?;
    m.add_class::<python_api::PyTileName>()?;
    m.add_class::<python_api::PyTile>()?;
    m.add_class::<python_api::PyMeld>()?;
    m.add_class::<python_api::PyHand>()?;
    m.add_class::<python_api::PyRiver>()?;
    m.add_class::<python_api::PyWall>()?;
    m.add_class::<python_api::PyRound>()?;
    m.add_class::<python_api::PyYakuId>()?;
    m.add_class::<python_api::PyYaku>()?;
    m.add_class::<python_api::PyWinContext>()?;
    m.add_function(wrap_pyfunction!(python_api::get_all_yaku, m)?)?;
    m.add_function(wrap_pyfunction!(python_api::py_judge_yaku, m)?)?;
    Ok(())
}
