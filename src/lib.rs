use pyo3::prelude::*;
use pyo3::wrap_pyfunction;
use game::Game;
use tile::Tile;
mod game;
mod hand;
mod test_game;
mod test_hand;
mod test_tile;
mod test_wall;
mod tile;
mod wall;

/// Formats the sum of two numbers as string.
#[pyfunction]
fn sum_as_string(a: usize, b: usize) -> PyResult<String> {
    Ok((a + b).to_string())
}

/// A Python module implemented in Rust.
#[pyclass]
struct MahjongGame {
    game: Game,
}

#[pymethods]
impl MahjongGame {
    #[new]
    fn new(is_red: bool) -> Self {
        MahjongGame {
            game: Game::new(is_red),
        }
    }

    fn get_hand(&self, player_index: usize) -> Vec<Tile> {
        self.game.hands[player_index].tiles.clone()
    }

    fn draw_tile(&mut self, player_index: usize) -> Option<Tile> {
        if let Some(tile) = self.game.draw_tile() {
            self.game.hands[player_index].draw = Some(tile);
            return Some(tile);
        }
        None
    }

    fn discard_tile(&mut self, player_index: usize, tile_index: usize) {
        self.game.discard_tile(player_index, tile_index);
    }

    fn is_winning(&self, player_index: usize) -> bool {
        self.game.hands[player_index].is_winning()
    }
}

impl IntoPy<PyObject> for Tile {
    fn into_py(self, py: Python) -> PyObject {
        let dict = pyo3::types::PyDict::new_bound(py);
        dict.set_item("name", self.name.to_string()).unwrap();
        dict.set_item("is_red", self.is_red).unwrap();
        dict.into()
    }
}

#[pymodule]
fn mahjong(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(sum_as_string, m)?)?;
    m.add_class::<MahjongGame>()?;
    Ok(())
}
