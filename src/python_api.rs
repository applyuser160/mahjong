use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

use crate::hand::{Hand, Meld};
use crate::river::River;
use crate::round::Round;
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

    pub fn tile_type(&self) -> PyTileType {
        let t: TileName = (*self).into();
        t.tile_type().into()
    }

    pub fn category(&self) -> PyTileCategory {
        let t: TileName = (*self).into();
        t.category().into()
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
