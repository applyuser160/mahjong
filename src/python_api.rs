use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

use crate::hand::{Hand, Meld};
use crate::placement_ev::{
    calculate_orasu_conditions, evaluate_hand_discards_with_placement, MatchContext,
    PlacementCandidateEvaluation, RuleConfig, WinCondition,
};
use crate::river::River;
use crate::round::Round;
use crate::shanten::{calculate_shanten, calculate_shanten_from_counts, ShantenResult};
use crate::tile::{Tile, TileCategory, TileName, TileType};
use crate::wall::Wall;
use crate::yaku::{judge_yaku, WinContext, Yaku, YakuId, ALL_YAKU};

// ==========================================
// 1. Tile related wrappers
// ==========================================

#[pyclass(eq, eq_int)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum PyTileType {
    None = 0,
    Characters,
    Circles,
    Bamboos,
    Winds,
    Dragons,
}

impl PyTileType {
    pub fn as_str(&self) -> &'static str {
        match self {
            PyTileType::None => "None",
            PyTileType::Characters => "Characters",
            PyTileType::Circles => "Circles",
            PyTileType::Bamboos => "Bamboos",
            PyTileType::Winds => "Winds",
            PyTileType::Dragons => "Dragons",
        }
    }
}

impl From<TileType> for PyTileType {
    fn from(t: TileType) -> Self {
        match t {
            TileType::None => PyTileType::None,
            TileType::Characters => PyTileType::Characters,
            TileType::Circles => PyTileType::Circles,
            TileType::Bamboos => PyTileType::Bamboos,
            TileType::Winds => PyTileType::Winds,
            TileType::Dragons => PyTileType::Dragons,
        }
    }
}

impl From<PyTileType> for TileType {
    fn from(val: PyTileType) -> Self {
        match val {
            PyTileType::None => TileType::None,
            PyTileType::Characters => TileType::Characters,
            PyTileType::Circles => TileType::Circles,
            PyTileType::Bamboos => TileType::Bamboos,
            PyTileType::Winds => TileType::Winds,
            PyTileType::Dragons => TileType::Dragons,
        }
    }
}

#[pyclass(eq, eq_int)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum PyTileCategory {
    None = 0,
    Simples,
    Honors,
}

impl PyTileCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            PyTileCategory::None => "None",
            PyTileCategory::Simples => "Simples",
            PyTileCategory::Honors => "Honors",
        }
    }
}

impl From<TileCategory> for PyTileCategory {
    fn from(c: TileCategory) -> Self {
        match c {
            TileCategory::None => PyTileCategory::None,
            TileCategory::Simples => PyTileCategory::Simples,
            TileCategory::Honors => PyTileCategory::Honors,
        }
    }
}

impl From<PyTileCategory> for TileCategory {
    fn from(val: PyTileCategory) -> Self {
        match val {
            PyTileCategory::None => TileCategory::None,
            PyTileCategory::Simples => TileCategory::Simples,
            PyTileCategory::Honors => TileCategory::Honors,
        }
    }
}

#[pyclass(eq, eq_int)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub enum PyTileName {
    None = 0,
    OneM,
    TwoM,
    ThreeM,
    FourM,
    FiveM,
    SixM,
    SevenM,
    EightM,
    NineM,
    OneP,
    TwoP,
    ThreeP,
    FourP,
    FiveP,
    SixP,
    SevenP,
    EightP,
    NineP,
    OneS,
    TwoS,
    ThreeS,
    FourS,
    FiveS,
    SixS,
    SevenS,
    EightS,
    NineS,
    East,
    South,
    West,
    North,
    Red,
    Green,
    White,
}

impl From<TileName> for PyTileName {
    fn from(t: TileName) -> Self {
        match t {
            TileName::None => PyTileName::None,
            TileName::OneM => PyTileName::OneM,
            TileName::TwoM => PyTileName::TwoM,
            TileName::ThreeM => PyTileName::ThreeM,
            TileName::FourM => PyTileName::FourM,
            TileName::FiveM => PyTileName::FiveM,
            TileName::SixM => PyTileName::SixM,
            TileName::SevenM => PyTileName::SevenM,
            TileName::EightM => PyTileName::EightM,
            TileName::NineM => PyTileName::NineM,
            TileName::OneP => PyTileName::OneP,
            TileName::TwoP => PyTileName::TwoP,
            TileName::ThreeP => PyTileName::ThreeP,
            TileName::FourP => PyTileName::FourP,
            TileName::FiveP => PyTileName::FiveP,
            TileName::SixP => PyTileName::SixP,
            TileName::SevenP => PyTileName::SevenP,
            TileName::EightP => PyTileName::EightP,
            TileName::NineP => PyTileName::NineP,
            TileName::OneS => PyTileName::OneS,
            TileName::TwoS => PyTileName::TwoS,
            TileName::ThreeS => PyTileName::ThreeS,
            TileName::FourS => PyTileName::FourS,
            TileName::FiveS => PyTileName::FiveS,
            TileName::SixS => PyTileName::SixS,
            TileName::SevenS => PyTileName::SevenS,
            TileName::EightS => PyTileName::EightS,
            TileName::NineS => PyTileName::NineS,
            TileName::East => PyTileName::East,
            TileName::South => PyTileName::South,
            TileName::West => PyTileName::West,
            TileName::North => PyTileName::North,
            TileName::Red => PyTileName::Red,
            TileName::Green => PyTileName::Green,
            TileName::White => PyTileName::White,
        }
    }
}

impl From<PyTileName> for TileName {
    fn from(val: PyTileName) -> Self {
        match val {
            PyTileName::None => TileName::None,
            PyTileName::OneM => TileName::OneM,
            PyTileName::TwoM => TileName::TwoM,
            PyTileName::ThreeM => TileName::ThreeM,
            PyTileName::FourM => TileName::FourM,
            PyTileName::FiveM => TileName::FiveM,
            PyTileName::SixM => TileName::SixM,
            PyTileName::SevenM => TileName::SevenM,
            PyTileName::EightM => TileName::EightM,
            PyTileName::NineM => TileName::NineM,
            PyTileName::OneP => TileName::OneP,
            PyTileName::TwoP => TileName::TwoP,
            PyTileName::ThreeP => TileName::ThreeP,
            PyTileName::FourP => TileName::FourP,
            PyTileName::FiveP => TileName::FiveP,
            PyTileName::SixP => TileName::SixP,
            PyTileName::SevenP => TileName::SevenP,
            PyTileName::EightP => TileName::EightP,
            PyTileName::NineP => TileName::NineP,
            PyTileName::OneS => TileName::OneS,
            PyTileName::TwoS => TileName::TwoS,
            PyTileName::ThreeS => TileName::ThreeS,
            PyTileName::FourS => TileName::FourS,
            PyTileName::FiveS => TileName::FiveS,
            PyTileName::SixS => TileName::SixS,
            PyTileName::SevenS => TileName::SevenS,
            PyTileName::EightS => TileName::EightS,
            PyTileName::NineS => TileName::NineS,
            PyTileName::East => TileName::East,
            PyTileName::South => TileName::South,
            PyTileName::West => TileName::West,
            PyTileName::North => TileName::North,
            PyTileName::Red => TileName::Red,
            PyTileName::Green => TileName::Green,
            PyTileName::White => TileName::White,
        }
    }
}

#[pymethods]
impl PyTileName {
    pub fn as_str(&self) -> &'static str {
        let t: TileName = (*self).into();
        t.as_str()
    }

    pub fn mpsz(&self) -> &'static str {
        match self {
            PyTileName::None => "",
            PyTileName::OneM => "1m",
            PyTileName::TwoM => "2m",
            PyTileName::ThreeM => "3m",
            PyTileName::FourM => "4m",
            PyTileName::FiveM => "5m",
            PyTileName::SixM => "6m",
            PyTileName::SevenM => "7m",
            PyTileName::EightM => "8m",
            PyTileName::NineM => "9m",
            PyTileName::OneP => "1p",
            PyTileName::TwoP => "2p",
            PyTileName::ThreeP => "3p",
            PyTileName::FourP => "4p",
            PyTileName::FiveP => "5p",
            PyTileName::SixP => "6p",
            PyTileName::SevenP => "7p",
            PyTileName::EightP => "8p",
            PyTileName::NineP => "9p",
            PyTileName::OneS => "1s",
            PyTileName::TwoS => "2s",
            PyTileName::ThreeS => "3s",
            PyTileName::FourS => "4s",
            PyTileName::FiveS => "5s",
            PyTileName::SixS => "6s",
            PyTileName::SevenS => "7s",
            PyTileName::EightS => "8s",
            PyTileName::NineS => "9s",
            PyTileName::East => "1z",
            PyTileName::South => "2z",
            PyTileName::West => "3z",
            PyTileName::North => "4z",
            PyTileName::White => "5z",
            PyTileName::Green => "6z",
            PyTileName::Red => "7z",
        }
    }

    pub fn tile_type(&self) -> PyTileType {
        let t: TileName = (*self).into();
        t.tile_type().into()
    }

    pub fn category(&self) -> PyTileCategory {
        let t: TileName = (*self).into();
        t.category().into()
    }

    fn __str__(&self) -> &'static str {
        self.as_str()
    }

    fn __repr__(&self) -> String {
        format!("TileName({}: {})", self.mpsz(), self.as_str())
    }
}

#[pyclass]
#[derive(Clone, Debug)]
pub struct PyTile {
    tile: Tile,
}

#[pymethods]
impl PyTile {
    #[new]
    #[allow(clippy::too_many_arguments)]
    pub fn new(name: PyTileName) -> Self {
        Self {
            tile: Tile::new(name.into()),
        }
    }

    #[getter]
    pub fn name(&self) -> PyTileName {
        self.tile.name().into()
    }

    #[getter]
    pub fn tile_type(&self) -> PyTileType {
        self.tile.tile_type().into()
    }

    #[getter]
    pub fn category(&self) -> PyTileCategory {
        self.tile.category().into()
    }

    pub fn as_str(&self) -> &'static str {
        self.name().as_str()
    }

    pub fn mpsz(&self) -> &'static str {
        self.name().mpsz()
    }

    pub fn to_dict(&self, py: Python<'_>) -> PyResult<Py<PyDict>> {
        let dict = PyDict::new(py);
        dict.set_item("name", self.name().as_str())?;
        dict.set_item("mpsz", self.name().mpsz())?;
        dict.set_item("type", self.tile_type().as_str())?;
        dict.set_item("category", self.category().as_str())?;
        Ok(dict.unbind())
    }

    fn __str__(&self) -> &'static str {
        self.as_str()
    }

    fn __repr__(&self) -> String {
        format!("Tile({}: {})", self.mpsz(), self.as_str())
    }
}

// ==========================================
// 2. Meld and Hand wrappers
// ==========================================

#[pyclass]
#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct PyMeld {
    meld: Meld,
}

impl From<Meld> for PyMeld {
    fn from(m: Meld) -> Self {
        Self { meld: m }
    }
}

impl From<PyMeld> for Meld {
    fn from(val: PyMeld) -> Self {
        val.meld
    }
}

impl From<&PyMeld> for Meld {
    fn from(val: &PyMeld) -> Self {
        val.meld
    }
}

#[pymethods]
impl PyMeld {
    #[staticmethod]
    pub fn chii(called: PyTileName, consumed: [PyTileName; 2]) -> Self {
        Self {
            meld: Meld::Chii {
                called: called.into(),
                consumed: [consumed[0].into(), consumed[1].into()],
            },
        }
    }

    #[staticmethod]
    pub fn pon(tile: PyTileName) -> Self {
        Self {
            meld: Meld::Pon(tile.into()),
        }
    }

    #[staticmethod]
    pub fn daiminkan(tile: PyTileName) -> Self {
        Self {
            meld: Meld::Daiminkan(tile.into()),
        }
    }

    #[staticmethod]
    pub fn ankan(tile: PyTileName) -> Self {
        Self {
            meld: Meld::Ankan(tile.into()),
        }
    }

    #[staticmethod]
    pub fn kakan(tile: PyTileName) -> Self {
        Self {
            meld: Meld::Kakan(tile.into()),
        }
    }

    #[getter]
    pub fn kind(&self) -> String {
        match self.meld {
            Meld::Chii { .. } => "chii".to_string(),
            Meld::Pon(_) => "pon".to_string(),
            Meld::Daiminkan(_) => "daiminkan".to_string(),
            Meld::Ankan(_) => "ankan".to_string(),
            Meld::Kakan(_) => "kakan".to_string(),
        }
    }

    #[getter]
    pub fn tiles(&self) -> Vec<PyTileName> {
        match self.meld {
            Meld::Chii { called, consumed } => {
                vec![called.into(), consumed[0].into(), consumed[1].into()]
            }
            Meld::Pon(t) => vec![t.into(), t.into(), t.into()],
            Meld::Daiminkan(t) => vec![t.into(), t.into(), t.into(), t.into()],
            Meld::Ankan(t) => vec![t.into(), t.into(), t.into(), t.into()],
            Meld::Kakan(t) => vec![t.into(), t.into(), t.into(), t.into()],
        }
    }

    pub fn to_dict(&self, py: Python<'_>) -> PyResult<Py<PyDict>> {
        let dict = PyDict::new(py);
        dict.set_item("kind", self.kind())?;
        let tiles_str = PyList::new(py, self.tiles().iter().map(|t| t.as_str()))?;
        let tiles_mpsz = PyList::new(py, self.tiles().iter().map(|t| t.mpsz()))?;
        dict.set_item("tiles", tiles_str)?;
        dict.set_item("mpsz", tiles_mpsz)?;
        Ok(dict.unbind())
    }

    fn __repr__(&self) -> String {
        let mpsz: Vec<&'static str> = self.tiles().iter().map(|t| t.mpsz()).collect();
        format!("Meld({}: {:?})", self.kind(), mpsz)
    }
}

#[pyclass]
#[derive(Clone, Debug, Default)]
pub struct PyHand {
    pub(crate) hand: Hand,
}

#[pymethods]
impl PyHand {
    #[new]
    #[allow(clippy::too_many_arguments)]
    pub fn new() -> Self {
        Self { hand: Hand::new() }
    }

    #[getter]
    pub fn tiles(&self) -> Vec<PyTileName> {
        self.hand.tiles().iter().map(|&t| t.into()).collect()
    }

    #[getter]
    pub fn open_melds(&self) -> Vec<PyMeld> {
        self.hand.open_melds.iter().map(|&m| m.into()).collect()
    }

    pub fn push(&mut self, tile: PyTileName) {
        self.hand.push(tile.into());
    }

    pub fn discard(&mut self, index: usize) -> PyResult<PyTileName> {
        match self.hand.discard(index) {
            Ok(t) => Ok(t.into()),
            Err(e) => Err(PyValueError::new_err(e.to_string())),
        }
    }

    pub fn call_meld(&mut self, meld: PyMeld) -> PyResult<()> {
        match self.hand.call_meld(meld.into()) {
            Ok(_) => Ok(()),
            Err(e) => Err(PyValueError::new_err(e.to_string())),
        }
    }

    pub fn shanten(&self) -> PyShantenResult {
        calculate_shanten(&self.hand).into()
    }

    pub fn to_dict(&self, py: Python<'_>) -> PyResult<Py<PyDict>> {
        let dict = PyDict::new(py);
        let tiles_str = PyList::new(py, self.tiles().iter().map(|t| t.as_str()))?;
        let tiles_mpsz = PyList::new(py, self.tiles().iter().map(|t| t.mpsz()))?;
        dict.set_item("tiles", tiles_str)?;
        dict.set_item("mpsz", tiles_mpsz)?;

        let melds_list = PyList::empty(py);
        for m in self.open_melds() {
            melds_list.append(m.to_dict(py)?)?;
        }
        dict.set_item("open_melds", melds_list)?;
        dict.set_item("shanten", self.shanten().min_shanten)?;
        Ok(dict.unbind())
    }

    fn __repr__(&self) -> String {
        let mpsz: Vec<&'static str> = self.tiles().iter().map(|t| t.mpsz()).collect();
        format!("Hand(tiles={:?}, melds={})", mpsz, self.open_melds().len())
    }
}

// ==========================================
// 3. River, Wall, and Round wrappers
// ==========================================

#[pyclass]
#[derive(Clone, Debug, Default)]
pub struct PyRiver {
    pub(crate) river: River,
}

#[pymethods]
impl PyRiver {
    #[new]
    #[allow(clippy::too_many_arguments)]
    pub fn new() -> Self {
        Self {
            river: River::new(),
        }
    }

    #[getter]
    pub fn tiles(&self) -> Vec<PyTileName> {
        self.river.tiles().iter().map(|&t| t.into()).collect()
    }

    pub fn to_dict(&self, py: Python<'_>) -> PyResult<Py<PyDict>> {
        let dict = PyDict::new(py);
        let tiles_str = PyList::new(py, self.tiles().iter().map(|t| t.as_str()))?;
        let tiles_mpsz = PyList::new(py, self.tiles().iter().map(|t| t.mpsz()))?;
        dict.set_item("tiles", tiles_str)?;
        dict.set_item("mpsz", tiles_mpsz)?;
        Ok(dict.unbind())
    }

    fn __repr__(&self) -> String {
        let mpsz: Vec<&'static str> = self.tiles().iter().map(|t| t.mpsz()).collect();
        format!("River({:?})", mpsz)
    }
}

#[pyclass]
#[derive(Clone, Debug, Default)]
pub struct PyWall {
    pub(crate) wall: Wall,
}

#[pymethods]
impl PyWall {
    #[new]
    #[allow(clippy::too_many_arguments)]
    pub fn new() -> Self {
        Self { wall: Wall::new() }
    }

    pub fn shuffle(&mut self, seed: u64) {
        use rand::rngs::StdRng;
        use rand::SeedableRng;
        let mut rng = StdRng::seed_from_u64(seed);
        self.wall.shuffle(&mut rng);
    }

    pub fn draw(&mut self) -> Option<PyTileName> {
        self.wall.draw().map(|t| t.into())
    }

    pub fn draw_replacement(&mut self) -> Option<PyTileName> {
        self.wall.draw_replacement().map(|t| t.into())
    }

    pub fn remaining(&self) -> usize {
        self.wall.remaining()
    }
}

#[pyclass]
#[derive(Debug)]
pub struct PyRound {
    round: Round,
}

#[pymethods]
impl PyRound {
    #[new]
    #[allow(clippy::too_many_arguments)]
    pub fn new(mut wall: PyRefMut<'_, PyWall>) -> Self {
        Self {
            round: Round::new(std::mem::take(&mut wall.wall)),
        }
    }

    pub fn turn(&self) -> usize {
        self.round.turn()
    }

    pub fn hand(&self, index: usize) -> Vec<PyTileName> {
        self.round.hand(index).iter().map(|&t| t.into()).collect()
    }

    pub fn river(&self, index: usize) -> PyRiver {
        PyRiver {
            river: self.round.river(index).clone(),
        }
    }

    pub fn draw_tile(&mut self) -> Option<PyTileName> {
        self.round.draw_tile().map(|t| t.into())
    }

    pub fn discard_tile(&mut self, index: usize) -> PyResult<PyTileName> {
        match self.round.discard_tile(index) {
            Ok(t) => Ok(t.into()),
            Err(e) => Err(PyValueError::new_err(e.to_string())),
        }
    }

    pub fn play_meld(&mut self, player_index: usize, meld: &PyMeld) -> PyResult<()> {
        match self.round.play_meld(player_index, meld.into()) {
            Ok(_) => Ok(()),
            Err(e) => Err(PyValueError::new_err(e.to_string())),
        }
    }
}

// ==========================================
// 4. Yaku and Yaku judgement wrappers
// ==========================================

#[pyclass(eq, eq_int)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub enum PyYakuId {
    Riichi,
    MenzenTsumo,
    Tanyao,
    Pinfu,
    Ipeiko,
    YakuhaiHaku,
    YakuhaiHatsu,
    YakuhaiChun,
    YakuhaiJikaze,
    YakuhaiBakaze,
    Chitoitsu,
    Toitoi,
    Sanankou,
    Shousangen,
    Chantaiyao,
    Ryanpeiko,
    SanshokuDoujun,
    SanshokuDoukou,
    Honitsu,
    Junchan,
    Chinitsu,
    Chinroutou,
    Honroutou,
    Sankantsu,
    KokushiMusou,
    Suuankou,
    Daisangen,
    Shousuushi,
    Daisuushi,
    Suukantsu,
    Tsuuiisou,
    Ryuuiisou,
    ChuurenPoutou,
    Tenhou,
    Chiihou,
    RinshanKaihou,
    Chankan,
    HaiteiRaoyue,
    HouteiRaoyui,
    DoubleRiichi,
    Ippatsu,
}

impl From<YakuId> for PyYakuId {
    fn from(y: YakuId) -> Self {
        match y {
            YakuId::Riichi => PyYakuId::Riichi,
            YakuId::MenzenTsumo => PyYakuId::MenzenTsumo,
            YakuId::Tanyao => PyYakuId::Tanyao,
            YakuId::Pinfu => PyYakuId::Pinfu,
            YakuId::Ipeiko => PyYakuId::Ipeiko,
            YakuId::YakuhaiHaku => PyYakuId::YakuhaiHaku,
            YakuId::YakuhaiHatsu => PyYakuId::YakuhaiHatsu,
            YakuId::YakuhaiChun => PyYakuId::YakuhaiChun,
            YakuId::YakuhaiJikaze => PyYakuId::YakuhaiJikaze,
            YakuId::YakuhaiBakaze => PyYakuId::YakuhaiBakaze,
            YakuId::Chitoitsu => PyYakuId::Chitoitsu,
            YakuId::Toitoi => PyYakuId::Toitoi,
            YakuId::Sanankou => PyYakuId::Sanankou,
            YakuId::Shousangen => PyYakuId::Shousangen,
            YakuId::Chantaiyao => PyYakuId::Chantaiyao,
            YakuId::Ryanpeiko => PyYakuId::Ryanpeiko,
            YakuId::SanshokuDoujun => PyYakuId::SanshokuDoujun,
            YakuId::SanshokuDoukou => PyYakuId::SanshokuDoukou,
            YakuId::Honitsu => PyYakuId::Honitsu,
            YakuId::Junchan => PyYakuId::Junchan,
            YakuId::Chinitsu => PyYakuId::Chinitsu,
            YakuId::Chinroutou => PyYakuId::Chinroutou,
            YakuId::Honroutou => PyYakuId::Honroutou,
            YakuId::Sankantsu => PyYakuId::Sankantsu,
            YakuId::KokushiMusou => PyYakuId::KokushiMusou,
            YakuId::Suuankou => PyYakuId::Suuankou,
            YakuId::Daisangen => PyYakuId::Daisangen,
            YakuId::Shousuushi => PyYakuId::Shousuushi,
            YakuId::Daisuushi => PyYakuId::Daisuushi,
            YakuId::Suukantsu => PyYakuId::Suukantsu,
            YakuId::Tsuuiisou => PyYakuId::Tsuuiisou,
            YakuId::Ryuuiisou => PyYakuId::Ryuuiisou,
            YakuId::ChuurenPoutou => PyYakuId::ChuurenPoutou,
            YakuId::Tenhou => PyYakuId::Tenhou,
            YakuId::Chiihou => PyYakuId::Chiihou,
            YakuId::RinshanKaihou => PyYakuId::RinshanKaihou,
            YakuId::Chankan => PyYakuId::Chankan,
            YakuId::HaiteiRaoyue => PyYakuId::HaiteiRaoyue,
            YakuId::HouteiRaoyui => PyYakuId::HouteiRaoyui,
            YakuId::DoubleRiichi => PyYakuId::DoubleRiichi,
            YakuId::Ippatsu => PyYakuId::Ippatsu,
        }
    }
}

impl From<PyYakuId> for YakuId {
    fn from(val: PyYakuId) -> Self {
        match val {
            PyYakuId::Riichi => YakuId::Riichi,
            PyYakuId::MenzenTsumo => YakuId::MenzenTsumo,
            PyYakuId::Tanyao => YakuId::Tanyao,
            PyYakuId::Pinfu => YakuId::Pinfu,
            PyYakuId::Ipeiko => YakuId::Ipeiko,
            PyYakuId::YakuhaiHaku => YakuId::YakuhaiHaku,
            PyYakuId::YakuhaiHatsu => YakuId::YakuhaiHatsu,
            PyYakuId::YakuhaiChun => YakuId::YakuhaiChun,
            PyYakuId::YakuhaiJikaze => YakuId::YakuhaiJikaze,
            PyYakuId::YakuhaiBakaze => YakuId::YakuhaiBakaze,
            PyYakuId::Chitoitsu => YakuId::Chitoitsu,
            PyYakuId::Toitoi => YakuId::Toitoi,
            PyYakuId::Sanankou => YakuId::Sanankou,
            PyYakuId::Shousangen => YakuId::Shousangen,
            PyYakuId::Chantaiyao => YakuId::Chantaiyao,
            PyYakuId::Ryanpeiko => YakuId::Ryanpeiko,
            PyYakuId::SanshokuDoujun => YakuId::SanshokuDoujun,
            PyYakuId::SanshokuDoukou => YakuId::SanshokuDoukou,
            PyYakuId::Honitsu => YakuId::Honitsu,
            PyYakuId::Junchan => YakuId::Junchan,
            PyYakuId::Chinitsu => YakuId::Chinitsu,
            PyYakuId::Chinroutou => YakuId::Chinroutou,
            PyYakuId::Honroutou => YakuId::Honroutou,
            PyYakuId::Sankantsu => YakuId::Sankantsu,
            PyYakuId::KokushiMusou => YakuId::KokushiMusou,
            PyYakuId::Suuankou => YakuId::Suuankou,
            PyYakuId::Daisangen => YakuId::Daisangen,
            PyYakuId::Shousuushi => YakuId::Shousuushi,
            PyYakuId::Daisuushi => YakuId::Daisuushi,
            PyYakuId::Suukantsu => YakuId::Suukantsu,
            PyYakuId::Tsuuiisou => YakuId::Tsuuiisou,
            PyYakuId::Ryuuiisou => YakuId::Ryuuiisou,
            PyYakuId::ChuurenPoutou => YakuId::ChuurenPoutou,
            PyYakuId::Tenhou => YakuId::Tenhou,
            PyYakuId::Chiihou => YakuId::Chiihou,
            PyYakuId::RinshanKaihou => YakuId::RinshanKaihou,
            PyYakuId::Chankan => YakuId::Chankan,
            PyYakuId::HaiteiRaoyue => YakuId::HaiteiRaoyue,
            PyYakuId::HouteiRaoyui => YakuId::HouteiRaoyui,
            PyYakuId::DoubleRiichi => YakuId::DoubleRiichi,
            PyYakuId::Ippatsu => YakuId::Ippatsu,
        }
    }
}

#[pyclass]
#[derive(Clone, Debug)]
pub struct PyYaku {
    yaku: Yaku,
}

impl From<Yaku> for PyYaku {
    fn from(y: Yaku) -> Self {
        Self { yaku: y }
    }
}

#[pymethods]
impl PyYaku {
    #[getter]
    pub fn id(&self) -> PyYakuId {
        self.yaku.id.into()
    }

    #[getter]
    pub fn name_ja(&self) -> String {
        self.yaku.name_ja.to_string()
    }

    #[getter]
    pub fn name_kana(&self) -> String {
        self.yaku.name_kana.to_string()
    }

    #[getter]
    pub fn han_closed(&self) -> i8 {
        self.yaku.han_closed
    }

    #[getter]
    pub fn han_open(&self) -> i8 {
        self.yaku.han_open
    }

    #[getter]
    pub fn yakuman(&self) -> bool {
        self.yaku.yakuman
    }
}

#[pyfunction]
pub fn get_all_yaku() -> Vec<PyYaku> {
    ALL_YAKU.iter().map(|y| (*y).into()).collect()
}

#[pyclass]
#[derive(Clone, Debug)]
pub struct PyWinContext {
    #[pyo3(get, set)]
    pub is_closed: bool,
    #[pyo3(get, set)]
    pub is_tsumo: bool,
    #[pyo3(get, set)]
    pub seat_wind: Option<PyTileName>,
    #[pyo3(get, set)]
    pub round_wind: Option<PyTileName>,
    #[pyo3(get, set)]
    pub riichi: bool,
    #[pyo3(get, set)]
    pub kan_count: usize,
    #[pyo3(get, set)]
    pub tenhou: bool,
    #[pyo3(get, set)]
    pub chiihou: bool,
    #[pyo3(get, set)]
    pub win_tile: Option<PyTileName>,
    #[pyo3(get, set)]
    pub is_rinshan: bool,
    #[pyo3(get, set)]
    pub is_chankan: bool,
    #[pyo3(get, set)]
    pub is_haitei: bool,
    #[pyo3(get, set)]
    pub is_houtei: bool,
    #[pyo3(get, set)]
    pub is_double_riichi: bool,
    #[pyo3(get, set)]
    pub is_ippatsu: bool,
}

#[pymethods]
impl PyWinContext {
    #[new]
    #[allow(clippy::too_many_arguments)]
    #[pyo3(signature = (is_closed=true, is_tsumo=true, seat_wind=None, round_wind=None, riichi=false, kan_count=0, tenhou=false, chiihou=false, win_tile=None, is_rinshan=false, is_chankan=false, is_haitei=false, is_houtei=false, is_double_riichi=false, is_ippatsu=false))]
    pub fn new(
        is_closed: bool,
        is_tsumo: bool,
        seat_wind: Option<PyTileName>,
        round_wind: Option<PyTileName>,
        riichi: bool,
        kan_count: usize,
        tenhou: bool,
        chiihou: bool,
        win_tile: Option<PyTileName>,
        is_rinshan: bool,
        is_chankan: bool,
        is_haitei: bool,
        is_houtei: bool,
        is_double_riichi: bool,
        is_ippatsu: bool,
    ) -> Self {
        Self {
            is_closed,
            is_tsumo,
            seat_wind,
            round_wind,
            riichi,
            kan_count,
            tenhou,
            chiihou,
            win_tile,
            is_rinshan,
            is_chankan,
            is_haitei,
            is_houtei,
            is_double_riichi,
            is_ippatsu,
        }
    }
}

impl From<PyWinContext> for WinContext {
    fn from(val: PyWinContext) -> Self {
        WinContext {
            is_closed: val.is_closed,
            is_tsumo: val.is_tsumo,
            seat_wind: val.seat_wind.map(|t| t.into()),
            round_wind: val.round_wind.map(|t| t.into()),
            riichi: val.riichi,
            kan_count: val.kan_count,
            tenhou: val.tenhou,
            chiihou: val.chiihou,
            win_tile: val.win_tile.map(|t| t.into()),
            is_rinshan: val.is_rinshan,
            is_chankan: val.is_chankan,
            is_haitei: val.is_haitei,
            is_houtei: val.is_houtei,
            is_double_riichi: val.is_double_riichi,
            is_ippatsu: val.is_ippatsu,
        }
    }
}

#[pyfunction]
pub fn py_judge_yaku(
    tiles: Vec<PyTileName>,
    melds: Vec<PyMeld>,
    context: PyWinContext,
) -> Vec<PyYakuId> {
    let mut closed_counts = [0u8; 35];
    for py_tile in tiles {
        let rs_tile: TileName = py_tile.into();
        let idx = rs_tile as usize;
        if idx < closed_counts.len() {
            closed_counts[idx] += 1;
        }
    }
    let rs_melds: &[Meld] =
        unsafe { std::slice::from_raw_parts(melds.as_ptr() as *const Meld, melds.len()) };
    let rs_context: WinContext = context.into();

    let result = judge_yaku(&closed_counts, rs_melds, rs_context);
    result.into_iter().map(|y| y.into()).collect()
}

// ==========================================
// 6. Shanten wrappers
// ==========================================

#[pyclass]
#[derive(Clone, Debug)]
pub struct PyShantenResult {
    #[pyo3(get)]
    pub min_shanten: i8,
    #[pyo3(get)]
    pub normal: i8,
    #[pyo3(get)]
    pub chitoitsu: i8,
    #[pyo3(get)]
    pub kokushi: i8,
}

#[pymethods]
impl PyShantenResult {
    fn __repr__(&self) -> String {
        format!(
            "ShantenResult(min={}, normal={}, chitoitsu={}, kokushi={})",
            self.min_shanten, self.normal, self.chitoitsu, self.kokushi
        )
    }
}

impl From<ShantenResult> for PyShantenResult {
    fn from(res: ShantenResult) -> Self {
        Self {
            min_shanten: res.min_shanten,
            normal: res.normal,
            chitoitsu: res.chitoitsu,
            kokushi: res.kokushi,
        }
    }
}

#[pyfunction]
#[pyo3(signature = (tiles, open_melds_count=0))]
pub fn py_calculate_shanten(
    py: Python<'_>,
    tiles: Vec<PyTileName>,
    open_melds_count: usize,
) -> PyShantenResult {
    let mut counts = [0u8; 35];
    for py_tile in tiles {
        let rs_tile: TileName = py_tile.into();
        let idx = rs_tile as usize;
        if idx < counts.len() {
            counts[idx] += 1;
        }
    }
    py.allow_threads(|| calculate_shanten_from_counts(&counts, open_melds_count))
        .into()
}

// ==========================================
// 7. Expectation & Review wrappers
// ==========================================

#[pyclass]
#[derive(Clone, Debug)]
pub struct PyCandidateEvaluation {
    #[pyo3(get)]
    pub discard_tile: PyTileName,
    #[pyo3(get)]
    pub shanten_after: i8,
    #[pyo3(get)]
    pub ev: f64,
    #[pyo3(get)]
    pub remaining_count: usize,
    #[pyo3(get)]
    pub expected_score: f64,
    #[pyo3(get)]
    pub expected_han: f64,
    #[pyo3(get)]
    pub risk_score: f64,
    #[pyo3(get)]
    pub is_safe: bool,
    #[pyo3(get)]
    pub primary_yaku: Vec<String>,
}

#[pymethods]
impl PyCandidateEvaluation {
    pub fn to_dict(&self, py: Python<'_>) -> PyResult<Py<PyDict>> {
        let dict = PyDict::new(py);
        dict.set_item("discard_tile", self.discard_tile.as_str())?;
        dict.set_item("mpsz", self.discard_tile.mpsz())?;
        dict.set_item("shanten_after", self.shanten_after)?;
        dict.set_item("ev", self.ev)?;
        dict.set_item("remaining_count", self.remaining_count)?;
        dict.set_item("expected_score", self.expected_score)?;
        dict.set_item("expected_han", self.expected_han)?;
        dict.set_item("risk_score", self.risk_score)?;
        dict.set_item("is_safe", self.is_safe)?;
        dict.set_item("primary_yaku", self.primary_yaku.clone())?;
        Ok(dict.unbind())
    }

    fn __repr__(&self) -> String {
        format!(
            "CandidateEvaluation(discard={:?}, shanten={}, ev={:.0}, rem={}, score={:.0})",
            self.discard_tile,
            self.shanten_after,
            self.ev,
            self.remaining_count,
            self.expected_score
        )
    }
}

impl From<crate::expectation::CandidateEvaluation> for PyCandidateEvaluation {
    fn from(c: crate::expectation::CandidateEvaluation) -> Self {
        Self {
            discard_tile: c.discard_tile.into(),
            shanten_after: c.shanten_after,
            ev: c.ev,
            remaining_count: c.speed.remaining_count,
            expected_score: c.value.expected_score,
            expected_han: c.value.expected_han,
            risk_score: c.safety.risk_score,
            is_safe: c.safety.is_safe,
            primary_yaku: c.value.primary_yaku.iter().map(|s| s.to_string()).collect(),
        }
    }
}

impl From<&PyCandidateEvaluation> for crate::expectation::CandidateEvaluation {
    fn from(c: &PyCandidateEvaluation) -> Self {
        Self {
            discard_tile: c.discard_tile.into(),
            shanten_after: c.shanten_after,
            ev: c.ev,
            speed: crate::expectation::SpeedMetric {
                accepted_tiles: Vec::new(),
                remaining_count: c.remaining_count,
                win_probability: 0.5,
            },
            value: crate::expectation::ValueMetric {
                expected_score: c.expected_score,
                expected_han: c.expected_han,
                primary_yaku: Vec::new(),
                has_high_value_potential: false,
            },
            safety: crate::expectation::SafetyMetric {
                risk_score: c.risk_score,
                is_safe: c.is_safe,
            },
        }
    }
}

#[pyfunction]
#[allow(clippy::too_many_arguments)] // PyO3 entry point exposing keyword arguments for granular analysis context
#[pyo3(signature = (
    tiles,
    is_dealer=true,
    dora_indicators=None,
    turn_number=None,
    remaining_wall_tiles=None,
    seat_wind=None,
    round_wind=None,
    visible_tiles=None,
    riichi_status=None,
    player_rivers=None,
    player_melds=None,
    player_is_dealer=None,
))]
pub fn py_evaluate_hand_discards(
    py: Python<'_>,
    tiles: Vec<PyTileName>,
    is_dealer: bool,
    dora_indicators: Option<Vec<PyTileName>>,
    turn_number: Option<usize>,
    remaining_wall_tiles: Option<usize>,
    seat_wind: Option<PyTileName>,
    round_wind: Option<PyTileName>,
    visible_tiles: Option<Vec<PyTileName>>,
    riichi_status: Option<[bool; 4]>,
    player_rivers: Option<Vec<Vec<PyTileName>>>,
    player_melds: Option<Vec<Vec<PyMeld>>>,
    player_is_dealer: Option<[bool; 4]>,
) -> Vec<PyCandidateEvaluation> {
    let mut hand = Hand::new();
    for t in tiles {
        hand.push(t.into());
    }

    let dora_vec: Vec<TileName> = dora_indicators
        .unwrap_or_else(|| vec![PyTileName::OneM])
        .into_iter()
        .map(|t| t.into())
        .collect();

    let riichi_arr = riichi_status.unwrap_or([false; 4]);
    let dealer_arr = player_is_dealer.unwrap_or([is_dealer, false, false, false]);

    let converted_rivers: Vec<Vec<TileName>> = player_rivers
        .unwrap_or_default()
        .into_iter()
        .map(|river| river.into_iter().map(|t| t.into()).collect())
        .collect();
    let river_slices: Vec<&[TileName]> = converted_rivers.iter().map(|r| r.as_slice()).collect();

    let converted_melds: Vec<Vec<Meld>> = player_melds
        .unwrap_or_default()
        .into_iter()
        .map(|melds| melds.into_iter().map(|m| m.into()).collect())
        .collect();
    let meld_slices: Vec<&[Meld]> = converted_melds.iter().map(|m| m.as_slice()).collect();

    let ctx = crate::expectation::AnalysisContext {
        turn_number: turn_number.unwrap_or(6),
        remaining_wall_tiles: remaining_wall_tiles.unwrap_or(50),
        seat_wind: seat_wind.map(|w| w.into()).or(Some(TileName::East)),
        round_wind: round_wind.map(|w| w.into()).or(Some(TileName::East)),
        dora_indicators: &dora_vec,
        is_dealer,
        target_player: 0,
        riichi_status: riichi_arr,
        player_rivers: &river_slices,
        player_melds: &meld_slices,
        player_is_dealer: dealer_arr,
    };

    let mut visible_counts = [0u8; 35];
    let visible_opt = if let Some(v_tiles) = visible_tiles {
        for t in v_tiles {
            let idx = TileName::from(t) as usize;
            if idx <= 34 {
                visible_counts[idx] = visible_counts[idx].saturating_add(1);
            }
        }
        Some(&visible_counts)
    } else {
        None
    };

    let evs =
        py.allow_threads(|| crate::expectation::evaluate_hand_discards(&hand, visible_opt, &ctx));
    evs.into_iter().map(|e| e.into()).collect()
}

#[pyclass]
#[derive(Default, Clone)]
pub struct PyReviewTracker {
    inner: crate::review::ReviewTracker,
}

#[pymethods]
impl PyReviewTracker {
    #[new]
    pub fn new() -> Self {
        Self {
            inner: crate::review::ReviewTracker::new(),
        }
    }

    pub fn record_decision(
        &mut self,
        turn: usize,
        chosen_tile: PyTileName,
        candidates: Vec<PyCandidateEvaluation>,
    ) {
        let rs_cands: Vec<crate::expectation::CandidateEvaluation> =
            candidates.iter().map(|c| c.into()).collect();
        self.inner
            .record_decision(turn, chosen_tile.into(), &rs_cands);
    }

    pub fn get_accuracy_rate(&self) -> f64 {
        self.inner.generate_report().accuracy_rate
    }

    pub fn get_total_ev_loss(&self) -> f64 {
        self.inner.generate_report().total_ev_loss
    }

    pub fn format_report(&self) -> String {
        let report = self.inner.generate_report();
        self.inner.format_report(&report)
    }

    pub fn to_dict(&self, py: Python<'_>) -> PyResult<Py<PyDict>> {
        let report = self.inner.generate_report();
        let dict = PyDict::new(py);
        dict.set_item("total_turns", report.total_turns)?;
        dict.set_item("optimal_picks_count", report.optimal_picks_count)?;
        dict.set_item("accuracy_rate", report.accuracy_rate)?;
        dict.set_item("total_ev_loss", report.total_ev_loss)?;
        dict.set_item("average_ev_loss", report.average_ev_loss)?;

        let blunders_list = PyList::empty(py);
        for b in report.blunders {
            let b_dict = PyDict::new(py);
            b_dict.set_item("turn", b.turn)?;
            b_dict.set_item("chosen_tile", b.chosen_tile.as_str())?;
            b_dict.set_item("chosen_mpsz", PyTileName::from(b.chosen_tile).mpsz())?;
            b_dict.set_item("chosen_ev", b.chosen_ev)?;
            b_dict.set_item("best_tile", b.best_tile.as_str())?;
            b_dict.set_item("best_mpsz", PyTileName::from(b.best_tile).mpsz())?;
            b_dict.set_item("best_ev", b.best_ev)?;
            b_dict.set_item("ev_loss", b.ev_loss)?;
            b_dict.set_item("severity", b.severity.label_ja())?;
            b_dict.set_item("explanation", b.explanation)?;
            blunders_list.append(b_dict)?;
        }
        dict.set_item("blunders", blunders_list)?;
        Ok(dict.unbind())
    }

    #[pyo3(signature = (threshold=None))]
    pub fn get_blunders_dict(
        &self,
        py: Python<'_>,
        threshold: Option<f64>,
    ) -> PyResult<Py<PyList>> {
        let report = self.inner.generate_report();
        let min_loss = threshold.unwrap_or(0.0);
        let list = PyList::empty(py);
        for b in report.blunders {
            if b.ev_loss >= min_loss {
                let b_dict = PyDict::new(py);
                b_dict.set_item("turn", b.turn)?;
                b_dict.set_item("chosen_tile", b.chosen_tile.as_str())?;
                b_dict.set_item("chosen_mpsz", PyTileName::from(b.chosen_tile).mpsz())?;
                b_dict.set_item("chosen_ev", b.chosen_ev)?;
                b_dict.set_item("best_tile", b.best_tile.as_str())?;
                b_dict.set_item("best_mpsz", PyTileName::from(b.best_tile).mpsz())?;
                b_dict.set_item("best_ev", b.best_ev)?;
                b_dict.set_item("ev_loss", b.ev_loss)?;
                b_dict.set_item("severity", b.severity.label_ja())?;
                b_dict.set_item("explanation", b.explanation)?;
                list.append(b_dict)?;
            }
        }
        Ok(list.unbind())
    }
}

// ==========================================
// 8. Drill & Call Advisor wrappers
// ==========================================

#[pyclass]
#[derive(Clone, Debug)]
pub struct PyCallChoice {
    #[pyo3(get)]
    pub action: String,
    #[pyo3(get)]
    pub post_shanten: i8,
    #[pyo3(get)]
    pub post_acceptance: usize,
    #[pyo3(get)]
    pub estimated_score: f64,
    #[pyo3(get)]
    pub ev: f64,
}

#[pymethods]
impl PyCallChoice {
    pub fn to_dict(&self, py: Python<'_>) -> PyResult<Py<PyDict>> {
        let dict = PyDict::new(py);
        dict.set_item("action", &self.action)?;
        dict.set_item("post_shanten", self.post_shanten)?;
        dict.set_item("post_acceptance", self.post_acceptance)?;
        dict.set_item("estimated_score", self.estimated_score)?;
        dict.set_item("ev", self.ev)?;
        Ok(dict.unbind())
    }
}

#[pyclass]
#[derive(Clone, Debug)]
pub struct PyCallAdvice {
    #[pyo3(get)]
    pub target_tile: PyTileName,
    #[pyo3(get)]
    pub is_kamicha: bool,
    #[pyo3(get)]
    pub best_action: String,
    #[pyo3(get)]
    pub recommendation: String,
    #[pyo3(get)]
    pub rationale: String,
    #[pyo3(get)]
    pub choices: Vec<PyCallChoice>,
}

#[pymethods]
impl PyCallAdvice {
    pub fn to_dict(&self, py: Python<'_>) -> PyResult<Py<PyDict>> {
        let dict = PyDict::new(py);
        dict.set_item("target_tile", self.target_tile.as_str())?;
        dict.set_item("target_mpsz", self.target_tile.mpsz())?;
        dict.set_item("is_kamicha", self.is_kamicha)?;
        dict.set_item("best_action", &self.best_action)?;
        dict.set_item("recommendation", &self.recommendation)?;
        dict.set_item("rationale", &self.rationale)?;
        let choices_list = PyList::empty(py);
        for c in &self.choices {
            choices_list.append(c.to_dict(py)?)?;
        }
        dict.set_item("choices", choices_list)?;
        Ok(dict.unbind())
    }
}

#[pyfunction]
#[pyo3(signature = (tiles, target_tile, is_kamicha=true, dora_indicators=None))]
pub fn py_advise_call(
    py: Python<'_>,
    tiles: Vec<PyTileName>,
    target_tile: PyTileName,
    is_kamicha: bool,
    dora_indicators: Option<Vec<PyTileName>>,
) -> Option<PyCallAdvice> {
    let mut hand = Hand::new();
    for t in tiles {
        hand.push(t.into());
    }

    let dora_vec: Vec<TileName> = dora_indicators
        .unwrap_or_else(|| vec![PyTileName::OneM])
        .into_iter()
        .map(|t| t.into())
        .collect();

    let ctx = crate::expectation::AnalysisContext {
        turn_number: 6,
        remaining_wall_tiles: 50,
        seat_wind: Some(TileName::East),
        round_wind: Some(TileName::East),
        dora_indicators: &dora_vec,
        is_dealer: true,
        ..Default::default()
    };

    let advice = py.allow_threads(|| {
        crate::call_advisor::CallAdvisor::advise_call(&hand, target_tile.into(), is_kamicha, &ctx)
    })?;

    let choices = advice
        .choices
        .into_iter()
        .map(|c| PyCallChoice {
            action: c.action.label_ja(),
            post_shanten: c.post_shanten,
            post_acceptance: c.post_acceptance,
            estimated_score: c.estimated_score,
            ev: c.ev,
        })
        .collect();

    Some(PyCallAdvice {
        target_tile,
        is_kamicha,
        best_action: advice.best_action.label_ja(),
        recommendation: advice.recommendation.label_ja().to_string(),
        rationale: advice.rationale,
        choices,
    })
}

#[pyclass]
#[derive(Clone, Debug)]
pub struct PyDrillProblem {
    #[pyo3(get)]
    pub tiles: Vec<PyTileName>,
    #[pyo3(get)]
    pub dora_indicator: PyTileName,
    #[pyo3(get)]
    pub turn_number: usize,
    #[pyo3(get)]
    pub best_tile: PyTileName,
    #[pyo3(get)]
    pub rationale: String,
    #[pyo3(get)]
    pub candidates: Vec<PyCandidateEvaluation>,
}

#[pymethods]
impl PyDrillProblem {
    pub fn to_dict(&self, py: Python<'_>) -> PyResult<Py<PyDict>> {
        let dict = PyDict::new(py);
        let tiles_str = PyList::new(py, self.tiles.iter().map(|t| t.as_str()))?;
        let tiles_mpsz = PyList::new(py, self.tiles.iter().map(|t| t.mpsz()))?;
        dict.set_item("tiles", tiles_str)?;
        dict.set_item("mpsz", tiles_mpsz)?;
        dict.set_item("dora_indicator", self.dora_indicator.as_str())?;
        dict.set_item("dora_mpsz", self.dora_indicator.mpsz())?;
        dict.set_item("turn_number", self.turn_number)?;
        dict.set_item("best_tile", self.best_tile.as_str())?;
        dict.set_item("best_mpsz", self.best_tile.mpsz())?;
        dict.set_item("rationale", &self.rationale)?;
        let cand_list = PyList::empty(py);
        for c in &self.candidates {
            cand_list.append(c.to_dict(py)?)?;
        }
        dict.set_item("candidates", cand_list)?;
        Ok(dict.unbind())
    }
}

use rand::rngs::SmallRng;
use rand::SeedableRng;

#[pyfunction]
#[pyo3(signature = (target_shanten=None))]
pub fn py_generate_drill_problem(target_shanten: Option<i8>) -> Option<PyDrillProblem> {
    let mut rng = SmallRng::from_entropy();
    let problem = crate::drill::DrillEngine::generate_problem(target_shanten, 100, &mut rng)?;

    let tiles = problem.hand.tiles().iter().map(|&t| t.into()).collect();
    let candidates = problem.candidates.into_iter().map(|c| c.into()).collect();

    Some(PyDrillProblem {
        tiles,
        dora_indicator: problem.dora_indicator.into(),
        turn_number: problem.turn_number,
        best_tile: problem.best_tile.into(),
        rationale: problem.rationale,
        candidates,
    })
}

// ==========================================
// 9. Placement EV, MatchContext & TableState wrappers
// ==========================================

#[pyclass]
#[derive(Clone, Debug, PartialEq)]
pub struct PyRuleConfig {
    #[pyo3(get, set)]
    pub origin_score: i32,
    #[pyo3(get, set)]
    pub return_score: i32,
    #[pyo3(get, set)]
    pub uma: [i32; 4],
    #[pyo3(get, set)]
    pub oka: i32,
}

#[pymethods]
impl PyRuleConfig {
    #[new]
    #[pyo3(signature = (origin_score=25000, return_score=30000, uma=None, oka=0))]
    pub fn new(origin_score: i32, return_score: i32, uma: Option<[i32; 4]>, oka: i32) -> Self {
        let uma = uma.unwrap_or([50, 10, -10, -30]);
        Self {
            origin_score,
            return_score,
            uma,
            oka,
        }
    }

    #[staticmethod]
    pub fn mleague() -> Self {
        let r = RuleConfig::mleague();
        Self {
            origin_score: r.origin_score,
            return_score: r.return_score,
            uma: r.uma,
            oka: r.oka,
        }
    }

    #[staticmethod]
    pub fn general() -> Self {
        let r = RuleConfig::general();
        Self {
            origin_score: r.origin_score,
            return_score: r.return_score,
            uma: r.uma,
            oka: r.oka,
        }
    }

    #[staticmethod]
    pub fn tenhou_dan() -> Self {
        let r = RuleConfig::tenhou_dan();
        Self {
            origin_score: r.origin_score,
            return_score: r.return_score,
            uma: r.uma,
            oka: r.oka,
        }
    }

    pub fn to_dict(&self, py: Python<'_>) -> PyResult<Py<PyDict>> {
        let dict = PyDict::new(py);
        dict.set_item("origin_score", self.origin_score)?;
        dict.set_item("return_score", self.return_score)?;
        dict.set_item("uma", self.uma.to_vec())?;
        dict.set_item("oka", self.oka)?;
        Ok(dict.unbind())
    }

    fn __repr__(&self) -> String {
        format!(
            "RuleConfig(origin={}, return={}, uma={:?}, oka={})",
            self.origin_score, self.return_score, self.uma, self.oka
        )
    }
}

impl From<PyRuleConfig> for RuleConfig {
    fn from(r: PyRuleConfig) -> Self {
        RuleConfig {
            origin_score: r.origin_score,
            return_score: r.return_score,
            uma: r.uma,
            oka: r.oka,
        }
    }
}

impl From<RuleConfig> for PyRuleConfig {
    fn from(r: RuleConfig) -> Self {
        PyRuleConfig {
            origin_score: r.origin_score,
            return_score: r.return_score,
            uma: r.uma,
            oka: r.oka,
        }
    }
}

#[pyclass]
#[derive(Clone, Debug)]
pub struct PyMatchContext {
    #[pyo3(get, set)]
    pub scores: [i32; 4],
    #[pyo3(get, set)]
    pub round_wind: PyTileName,
    #[pyo3(get, set)]
    pub round_number: u8,
    #[pyo3(get, set)]
    pub honba: u8,
    #[pyo3(get, set)]
    pub riichi_sticks: u8,
    pub dealer_idx: usize,
    #[pyo3(get, set)]
    pub rule: PyRuleConfig,
}

#[pymethods]
impl PyMatchContext {
    #[new]
    #[pyo3(signature = (scores=None, round_wind=None, round_number=1, honba=0, riichi_sticks=0, dealer_idx=0, rule=None))]
    pub fn new(
        scores: Option<[i32; 4]>,
        round_wind: Option<PyTileName>,
        round_number: u8,
        honba: u8,
        riichi_sticks: u8,
        dealer_idx: usize,
        rule: Option<PyRuleConfig>,
    ) -> PyResult<Self> {
        if dealer_idx >= 4 {
            return Err(PyValueError::new_err("dealer_idx must be in range 0..4"));
        }
        Ok(Self {
            scores: scores.unwrap_or([25000, 25000, 25000, 25000]),
            round_wind: round_wind.unwrap_or(PyTileName::East),
            round_number,
            honba,
            riichi_sticks,
            dealer_idx,
            rule: rule.unwrap_or_else(PyRuleConfig::mleague),
        })
    }

    #[getter]
    pub fn dealer_idx(&self) -> usize {
        self.dealer_idx
    }

    #[setter]
    pub fn set_dealer_idx(&mut self, idx: usize) -> PyResult<()> {
        if idx >= 4 {
            return Err(PyValueError::new_err("dealer_idx must be in range 0..4"));
        }
        self.dealer_idx = idx;
        Ok(())
    }

    pub fn current_ranks(&self) -> [usize; 4] {
        MatchContext::from(self).current_ranks()
    }

    pub fn score_diff(&self, p: usize, target: usize) -> PyResult<i32> {
        if p >= 4 || target >= 4 {
            return Err(PyValueError::new_err("player index must be in range 0..4"));
        }
        Ok(MatchContext::from(self).score_diff(p, target))
    }

    pub fn is_orasu(&self) -> bool {
        MatchContext::from(self).is_orasu()
    }

    pub fn remaining_rounds(&self) -> usize {
        MatchContext::from(self).remaining_rounds()
    }

    pub fn to_dict(&self, py: Python<'_>) -> PyResult<Py<PyDict>> {
        if self.dealer_idx >= 4 {
            return Err(PyValueError::new_err("dealer_idx must be in range 0..4"));
        }
        let dict = PyDict::new(py);
        dict.set_item("scores", self.scores.to_vec())?;
        dict.set_item("round_wind", self.round_wind.as_str())?;
        dict.set_item("round_wind_mpsz", self.round_wind.mpsz())?;
        dict.set_item("round_number", self.round_number)?;
        dict.set_item("honba", self.honba)?;
        dict.set_item("riichi_sticks", self.riichi_sticks)?;
        dict.set_item("dealer_idx", self.dealer_idx)?;
        dict.set_item("ranks", self.current_ranks().to_vec())?;
        dict.set_item("is_orasu", self.is_orasu())?;
        dict.set_item("remaining_rounds", self.remaining_rounds())?;
        dict.set_item("rule", self.rule.to_dict(py)?)?;
        Ok(dict.unbind())
    }

    fn __repr__(&self) -> String {
        format!(
            "MatchContext({}{}局 {}本場, 供託:{}, 親:{}, 点数:{:?})",
            self.round_wind.as_str(),
            self.round_number,
            self.honba,
            self.riichi_sticks,
            self.dealer_idx,
            self.scores
        )
    }
}

impl From<PyMatchContext> for MatchContext {
    fn from(c: PyMatchContext) -> Self {
        Self {
            scores: c.scores,
            round_wind: c.round_wind.into(),
            round_number: c.round_number,
            honba: c.honba,
            riichi_sticks: c.riichi_sticks,
            dealer_idx: c.dealer_idx,
            rule: c.rule.into(),
        }
    }
}

impl From<MatchContext> for PyMatchContext {
    fn from(c: MatchContext) -> Self {
        Self {
            scores: c.scores,
            round_wind: c.round_wind.into(),
            round_number: c.round_number,
            honba: c.honba,
            riichi_sticks: c.riichi_sticks,
            dealer_idx: c.dealer_idx,
            rule: c.rule.into(),
        }
    }
}

impl From<&PyMatchContext> for MatchContext {
    fn from(c: &PyMatchContext) -> Self {
        Self {
            scores: c.scores,
            round_wind: c.round_wind.into(),
            round_number: c.round_number,
            honba: c.honba,
            riichi_sticks: c.riichi_sticks,
            dealer_idx: c.dealer_idx,
            rule: c.rule.clone().into(),
        }
    }
}

#[pyclass]
#[derive(Clone, Debug)]
pub struct PyWinCondition {
    #[pyo3(get)]
    pub target_rank: usize,
    #[pyo3(get)]
    pub target_player: usize,
    #[pyo3(get)]
    pub diff: i32,
    #[pyo3(get)]
    pub ron_direct_req: Option<i32>,
    #[pyo3(get)]
    pub tsumo_req: Option<i32>,
    #[pyo3(get)]
    pub ron_other_req: Option<i32>,
    #[pyo3(get)]
    pub summary: String,
}

#[pymethods]
impl PyWinCondition {
    pub fn to_dict(&self, py: Python<'_>) -> PyResult<Py<PyDict>> {
        let dict = PyDict::new(py);
        dict.set_item("target_rank", self.target_rank)?;
        dict.set_item("target_player", self.target_player)?;
        dict.set_item("diff", self.diff)?;
        dict.set_item("ron_direct_req", self.ron_direct_req)?;
        dict.set_item("tsumo_req", self.tsumo_req)?;
        dict.set_item("ron_other_req", self.ron_other_req)?;
        dict.set_item("summary", &self.summary)?;
        Ok(dict.unbind())
    }

    fn __repr__(&self) -> String {
        format!("WinCondition({}: {})", self.target_rank, self.summary)
    }
}

impl From<WinCondition> for PyWinCondition {
    fn from(w: WinCondition) -> Self {
        Self {
            target_rank: w.target_rank,
            target_player: w.target_player,
            diff: w.diff,
            ron_direct_req: w.ron_direct_req,
            tsumo_req: w.tsumo_req,
            ron_other_req: w.ron_other_req,
            summary: w.summary,
        }
    }
}

#[pyclass]
#[derive(Clone, Debug)]
pub struct PyPlacementEvaluation {
    #[pyo3(get)]
    pub discard_tile: PyTileName,
    #[pyo3(get)]
    pub raw_ev: f64,
    #[pyo3(get)]
    pub placement_ev: f64,
    #[pyo3(get)]
    pub expected_rank: f64,
    #[pyo3(get)]
    pub rank_probabilities: [f64; 4],
    #[pyo3(get)]
    pub situational_note: String,
    #[pyo3(get)]
    pub shanten_after: i8,
    #[pyo3(get)]
    pub remaining_count: usize,
    #[pyo3(get)]
    pub expected_score: f64,
    #[pyo3(get)]
    pub risk_score: f64,
    #[pyo3(get)]
    pub is_safe: bool,
}

#[pymethods]
impl PyPlacementEvaluation {
    pub fn to_dict(&self, py: Python<'_>) -> PyResult<Py<PyDict>> {
        let dict = PyDict::new(py);
        dict.set_item("discard_tile", self.discard_tile.as_str())?;
        dict.set_item("mpsz", self.discard_tile.mpsz())?;
        dict.set_item("raw_ev", self.raw_ev)?;
        dict.set_item("placement_ev", self.placement_ev)?;
        dict.set_item("expected_rank", self.expected_rank)?;
        dict.set_item("rank_probabilities", self.rank_probabilities.to_vec())?;
        dict.set_item("situational_note", &self.situational_note)?;
        dict.set_item("shanten_after", self.shanten_after)?;
        dict.set_item("remaining_count", self.remaining_count)?;
        dict.set_item("expected_score", self.expected_score)?;
        dict.set_item("risk_score", self.risk_score)?;
        dict.set_item("is_safe", self.is_safe)?;
        Ok(dict.unbind())
    }

    fn __repr__(&self) -> String {
        format!(
            "PlacementEvaluation(discard={}, pt_ev={:.2}, exp_rank={:.2}, note={})",
            self.discard_tile.as_str(),
            self.placement_ev,
            self.expected_rank,
            self.situational_note
        )
    }
}

impl From<PlacementCandidateEvaluation> for PyPlacementEvaluation {
    fn from(p: PlacementCandidateEvaluation) -> Self {
        Self {
            discard_tile: p.base.discard_tile.into(),
            raw_ev: p.base.ev,
            placement_ev: p.placement_ev,
            expected_rank: p.expected_rank,
            rank_probabilities: p.rank_probabilities,
            situational_note: p.situational_note,
            shanten_after: p.base.shanten_after,
            remaining_count: p.base.speed.remaining_count,
            expected_score: p.base.value.expected_score,
            risk_score: p.base.safety.risk_score,
            is_safe: p.base.safety.is_safe,
        }
    }
}

#[pyclass]
#[derive(Clone, Debug)]
pub struct PyTableState {
    #[pyo3(get, set)]
    pub round_wind: PyTileName,
    #[pyo3(get, set)]
    pub round_number: u8,
    #[pyo3(get, set)]
    pub honba: u8,
    #[pyo3(get, set)]
    pub riichi_sticks: u8,
    pub dealer_idx: usize,
    pub current_turn: usize,
    #[pyo3(get, set)]
    pub dora_indicators: Vec<PyTileName>,
    #[pyo3(get, set)]
    pub remaining_wall_tiles: usize,
    #[pyo3(get, set)]
    pub scores: [i32; 4],
    #[pyo3(get, set)]
    pub is_riichi: [bool; 4],
    #[pyo3(get, set)]
    pub hands: Vec<Vec<PyTileName>>,
    #[pyo3(get, set)]
    pub melds: Vec<Vec<PyMeld>>,
    #[pyo3(get, set)]
    pub rivers: Vec<Vec<PyTileName>>,
}

#[pymethods]
impl PyTableState {
    #[new]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        round_wind: PyTileName,
        round_number: u8,
        honba: u8,
        riichi_sticks: u8,
        dealer_idx: usize,
        current_turn: usize,
        dora_indicators: Vec<PyTileName>,
        remaining_wall_tiles: usize,
        scores: [i32; 4],
        is_riichi: [bool; 4],
        hands: Vec<Vec<PyTileName>>,
        melds: Vec<Vec<PyMeld>>,
        rivers: Vec<Vec<PyTileName>>,
    ) -> PyResult<Self> {
        if dealer_idx >= 4 {
            return Err(PyValueError::new_err("dealer_idx must be in range 0..4"));
        }
        if current_turn >= 4 {
            return Err(PyValueError::new_err("current_turn must be in range 0..4"));
        }
        Ok(Self {
            round_wind,
            round_number,
            honba,
            riichi_sticks,
            dealer_idx,
            current_turn,
            dora_indicators,
            remaining_wall_tiles,
            scores,
            is_riichi,
            hands,
            melds,
            rivers,
        })
    }

    #[getter]
    pub fn dealer_idx(&self) -> usize {
        self.dealer_idx
    }

    #[setter]
    pub fn set_dealer_idx(&mut self, idx: usize) -> PyResult<()> {
        if idx >= 4 {
            return Err(PyValueError::new_err("dealer_idx must be in range 0..4"));
        }
        self.dealer_idx = idx;
        Ok(())
    }

    #[getter]
    pub fn current_turn(&self) -> usize {
        self.current_turn
    }

    #[setter]
    pub fn set_current_turn(&mut self, turn: usize) -> PyResult<()> {
        if turn >= 4 {
            return Err(PyValueError::new_err("current_turn must be in range 0..4"));
        }
        self.current_turn = turn;
        Ok(())
    }

    #[pyo3(signature = (reveal_all=false))]
    pub fn to_dict(&self, py: Python<'_>, reveal_all: bool) -> PyResult<Py<PyDict>> {
        if self.dealer_idx >= 4 {
            return Err(PyValueError::new_err("dealer_idx must be in range 0..4"));
        }
        if self.current_turn >= 4 {
            return Err(PyValueError::new_err("current_turn must be in range 0..4"));
        }
        let dict = PyDict::new(py);
        dict.set_item("round_wind", self.round_wind.as_str())?;
        dict.set_item("round_wind_mpsz", self.round_wind.mpsz())?;
        dict.set_item("round_number", self.round_number)?;
        dict.set_item("honba", self.honba)?;
        dict.set_item("riichi_sticks", self.riichi_sticks)?;
        dict.set_item("dealer_idx", self.dealer_idx)?;
        dict.set_item("current_turn", self.current_turn)?;

        let dora_str = PyList::new(py, self.dora_indicators.iter().map(|t| t.as_str()))?;
        let dora_mpsz = PyList::new(py, self.dora_indicators.iter().map(|t| t.mpsz()))?;
        dict.set_item("dora_indicators", dora_str)?;
        dict.set_item("dora_indicators_mpsz", dora_mpsz)?;
        dict.set_item("remaining_wall_tiles", self.remaining_wall_tiles)?;

        let players_list = PyList::empty(py);
        let winds = ["East", "South", "West", "North"];
        let winds_ja = ["東", "南", "西", "北"];
        for i in 0..4 {
            let p_dict = PyDict::new(py);
            p_dict.set_item("seat", i)?;
            let rel_wind = (i + 4 - self.dealer_idx) % 4;
            p_dict.set_item("seat_wind", winds[rel_wind])?;
            p_dict.set_item("seat_wind_ja", winds_ja[rel_wind])?;
            p_dict.set_item("score", self.scores[i])?;
            p_dict.set_item("is_riichi", self.is_riichi[i])?;
            p_dict.set_item("is_dealer", i == self.dealer_idx)?;

            // Hand tiles (seat 0 visible by default; others hidden unless reveal_all)
            let hand_list = PyList::empty(py);
            let mpsz_list = PyList::empty(py);
            if i == 0 || reveal_all {
                if let Some(h) = self.hands.get(i) {
                    for t in h {
                        hand_list.append(t.as_str())?;
                        mpsz_list.append(t.mpsz())?;
                    }
                }
            } else if let Some(h) = self.hands.get(i) {
                for _ in 0..h.len() {
                    hand_list.append("?")?;
                    mpsz_list.append("?")?;
                }
            }
            p_dict.set_item("hand", hand_list)?;
            p_dict.set_item("hand_mpsz", mpsz_list)?;

            // Melds
            let melds_list = PyList::empty(py);
            if let Some(m_vec) = self.melds.get(i) {
                for m in m_vec {
                    melds_list.append(m.to_dict(py)?)?;
                }
            }
            p_dict.set_item("melds", melds_list)?;

            // River
            let river_list = PyList::empty(py);
            let river_mpsz_list = PyList::empty(py);
            if let Some(r_vec) = self.rivers.get(i) {
                for t in r_vec {
                    river_list.append(t.as_str())?;
                    river_mpsz_list.append(t.mpsz())?;
                }
            }
            p_dict.set_item("river", river_list)?;
            p_dict.set_item("river_mpsz", river_mpsz_list)?;

            players_list.append(p_dict)?;
        }
        dict.set_item("players", players_list)?;
        Ok(dict.unbind())
    }
}

#[pyfunction]
#[allow(clippy::too_many_arguments)] // PyO3 entry point exposing keyword arguments for granular analysis context
#[pyo3(signature = (
    tiles,
    match_context,
    player_idx=0,
    is_dealer=None,
    dora_indicators=None,
    turn_number=None,
    remaining_wall_tiles=None,
    seat_wind=None,
    visible_tiles=None,
    riichi_status=None,
    player_rivers=None,
    player_melds=None,
    player_is_dealer=None,
))]
pub fn py_evaluate_placement_discards(
    py: Python<'_>,
    tiles: Vec<PyTileName>,
    match_context: &PyMatchContext,
    player_idx: usize,
    is_dealer: Option<bool>,
    dora_indicators: Option<Vec<PyTileName>>,
    turn_number: Option<usize>,
    remaining_wall_tiles: Option<usize>,
    seat_wind: Option<PyTileName>,
    visible_tiles: Option<Vec<PyTileName>>,
    riichi_status: Option<[bool; 4]>,
    player_rivers: Option<Vec<Vec<PyTileName>>>,
    player_melds: Option<Vec<Vec<PyMeld>>>,
    player_is_dealer: Option<[bool; 4]>,
) -> PyResult<Vec<PyPlacementEvaluation>> {
    if player_idx >= 4 {
        return Err(PyValueError::new_err("player_idx must be in range 0..4"));
    }
    if match_context.dealer_idx >= 4 {
        return Err(PyValueError::new_err("dealer_idx must be in range 0..4"));
    }
    let mut hand = Hand::new();
    for t in tiles {
        hand.push(t.into());
    }

    let dora_vec: Vec<TileName> = dora_indicators
        .unwrap_or_else(|| vec![PyTileName::OneM])
        .into_iter()
        .map(|t| t.into())
        .collect();

    let dealer = is_dealer.unwrap_or(player_idx == match_context.dealer_idx);

    let calculated_seat_wind = match (player_idx + 4 - match_context.dealer_idx) % 4 {
        0 => TileName::East,
        1 => TileName::South,
        2 => TileName::West,
        _ => TileName::North,
    };
    let s_wind = seat_wind.map(|w| w.into()).unwrap_or(calculated_seat_wind);

    let riichi_arr = riichi_status.unwrap_or([false; 4]);
    let dealer_arr = player_is_dealer.unwrap_or([
        match_context.dealer_idx == 0,
        match_context.dealer_idx == 1,
        match_context.dealer_idx == 2,
        match_context.dealer_idx == 3,
    ]);

    let converted_rivers: Vec<Vec<TileName>> = player_rivers
        .unwrap_or_default()
        .into_iter()
        .map(|river| river.into_iter().map(|t| t.into()).collect())
        .collect();
    let river_slices: Vec<&[TileName]> = converted_rivers.iter().map(|r| r.as_slice()).collect();

    let converted_melds: Vec<Vec<Meld>> = player_melds
        .unwrap_or_default()
        .into_iter()
        .map(|melds| melds.into_iter().map(|m| m.into()).collect())
        .collect();
    let meld_slices: Vec<&[Meld]> = converted_melds.iter().map(|m| m.as_slice()).collect();

    let ctx = crate::expectation::AnalysisContext {
        turn_number: turn_number.unwrap_or(6),
        remaining_wall_tiles: remaining_wall_tiles.unwrap_or(50),
        seat_wind: Some(s_wind),
        round_wind: Some(match_context.round_wind.into()),
        dora_indicators: &dora_vec,
        is_dealer: dealer,
        target_player: player_idx,
        riichi_status: riichi_arr,
        player_rivers: &river_slices,
        player_melds: &meld_slices,
        player_is_dealer: dealer_arr,
    };

    let mut visible_counts = [0u8; 35];
    let visible_opt = if let Some(v_tiles) = visible_tiles {
        for t in v_tiles {
            let idx = TileName::from(t) as usize;
            if idx <= 34 {
                visible_counts[idx] = visible_counts[idx].saturating_add(1);
            }
        }
        Some(&visible_counts)
    } else {
        None
    };

    let rs_match: MatchContext = match_context.into();
    let evs = py.allow_threads(|| {
        evaluate_hand_discards_with_placement(&hand, visible_opt, &ctx, &rs_match, player_idx)
    });
    Ok(evs.into_iter().map(|e| e.into()).collect())
}

#[pyfunction]
#[pyo3(signature = (match_context, player_idx=0))]
pub fn py_calculate_orasu_conditions(
    match_context: &PyMatchContext,
    player_idx: usize,
) -> PyResult<Vec<PyWinCondition>> {
    if player_idx >= 4 {
        return Err(PyValueError::new_err("player_idx must be in range 0..4"));
    }
    if match_context.dealer_idx >= 4 {
        return Err(PyValueError::new_err("dealer_idx must be in range 0..4"));
    }
    let rs_match: MatchContext = match_context.into();
    let conds = calculate_orasu_conditions(&rs_match, player_idx);
    Ok(conds.into_iter().map(|c| c.into()).collect())
}

#[pyfunction]
#[allow(clippy::too_many_arguments)] // PyO3 entry point exposing keyword arguments for granular analysis context
#[pyo3(signature = (
    tiles,
    match_context,
    player_idx=0,
    is_dealer=None,
    dora_indicators=None,
    turn_number=None,
    remaining_wall_tiles=None,
    seat_wind=None,
    visible_tiles=None,
    riichi_status=None,
    player_rivers=None,
    player_melds=None,
    player_is_dealer=None,
))]
pub fn py_get_ai_hud_data(
    py: Python<'_>,
    tiles: Vec<PyTileName>,
    match_context: &PyMatchContext,
    player_idx: usize,
    is_dealer: Option<bool>,
    dora_indicators: Option<Vec<PyTileName>>,
    turn_number: Option<usize>,
    remaining_wall_tiles: Option<usize>,
    seat_wind: Option<PyTileName>,
    visible_tiles: Option<Vec<PyTileName>>,
    riichi_status: Option<[bool; 4]>,
    player_rivers: Option<Vec<Vec<PyTileName>>>,
    player_melds: Option<Vec<Vec<PyMeld>>>,
    player_is_dealer: Option<[bool; 4]>,
) -> PyResult<Py<PyDict>> {
    if player_idx >= 4 {
        return Err(PyValueError::new_err("player_idx must be in range 0..4"));
    }
    if match_context.dealer_idx >= 4 {
        return Err(PyValueError::new_err("dealer_idx must be in range 0..4"));
    }
    let evs = py_evaluate_placement_discards(
        py,
        tiles,
        match_context,
        player_idx,
        is_dealer,
        dora_indicators,
        turn_number,
        remaining_wall_tiles,
        seat_wind,
        visible_tiles,
        riichi_status,
        player_rivers,
        player_melds,
        player_is_dealer,
    )?;
    let dict = PyDict::new(py);

    let ranks = match_context.current_ranks();
    dict.set_item("current_rank", ranks[player_idx])?;
    dict.set_item("current_score", match_context.scores[player_idx])?;
    dict.set_item("is_orasu", match_context.is_orasu())?;

    let cand_list = PyList::empty(py);
    for e in &evs {
        cand_list.append(e.to_dict(py)?)?;
    }
    dict.set_item("candidates", cand_list)?;

    if let Some(best) = evs.first() {
        dict.set_item("best_tile", best.discard_tile.as_str())?;
        dict.set_item("best_mpsz", best.discard_tile.mpsz())?;
        dict.set_item("best_placement_ev", best.placement_ev)?;
        dict.set_item("best_raw_ev", best.raw_ev)?;
        dict.set_item("best_note", &best.situational_note)?;
    }

    if match_context.is_orasu() {
        let conds = py_calculate_orasu_conditions(match_context, player_idx)?;
        let cond_list = PyList::empty(py);
        for c in conds {
            cond_list.append(c.to_dict(py)?)?;
        }
        dict.set_item("orasu_conditions", cond_list)?;
    }

    Ok(dict.unbind())
}
