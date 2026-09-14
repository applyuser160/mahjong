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

#[path = "mahjong/shanten.rs"]
pub mod shanten;

#[path = "mahjong/acceptance.rs"]
pub mod acceptance;

#[path = "mahjong/dora.rs"]
pub mod dora;

#[path = "mahjong/score.rs"]
pub mod score;

#[path = "mahjong/expectation.rs"]
pub mod expectation;

#[path = "mahjong/explanation.rs"]
pub mod explanation;

#[path = "mahjong/review.rs"]
pub mod review;

#[path = "mahjong/drill.rs"]
pub mod drill;

#[path = "mahjong/call_advisor.rs"]
pub mod call_advisor;

#[path = "mahjong/placement_ev.rs"]
pub mod placement_ev;

use pyo3::prelude::*;
use rand::rngs::SmallRng;
use rand::SeedableRng;

pub use acceptance::{
    analyze_all_discards, calculate_acceptance, AcceptanceResult, DiscardAnalysis, WaitTile,
};
pub use call_advisor::{CallAction, CallAdvice, CallAdvisor, CallChoice, CallRecommendation};
pub use dora::{count_dora, indicator_to_dora};
pub use drill::{DrillAnswerResult, DrillEngine, DrillProblem, DrillSession, DrillSessionReport};
pub use expectation::{
    evaluate_hand_discards, evaluate_standing_hand, AnalysisContext, CandidateEvaluation,
    SafetyMetric, SpeedMetric, StandingHandEvaluation, ValueMetric,
};
pub use explanation::Explainer;
pub use placement_ev::{
    calculate_orasu_conditions, evaluate_hand_discards_with_placement, MatchContext,
    PlacementCandidateEvaluation, RuleConfig, WinCondition,
};
pub use review::{
    BlunderRecord, BlunderSeverity, MatchReviewReport, ReviewTracker, TurnDecisionRecord,
};
pub use round::{Round, PLAYER_NUMBER};
pub use score::{calculate_fu, calculate_score, PlayerSeat, ScoreResult};
pub use shanten::{calculate_shanten, calculate_shanten_from_counts, ShantenResult};
pub use tile::{
    Tile, TileCategory, TileName, TileType, TILE_NAME_NUMBER, TILE_PER_KIND, TILE_WALL_CAPACITY,
};
pub use wall::Wall;
pub use yaku::{judge_yaku, WinContext, Yaku, YakuId, ALL_YAKU};

pub mod python_api;

#[pyfunction]
pub fn play_once(seed: u64) -> PyResult<Vec<&'static str>> {
    let mut wall = Wall::new();
    let mut rng = SmallRng::seed_from_u64(seed);
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
fn _core(m: &Bound<'_, PyModule>) -> PyResult<()> {
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
    m.add_class::<python_api::PyShantenResult>()?;
    m.add_class::<python_api::PyCandidateEvaluation>()?;
    m.add_class::<python_api::PyReviewTracker>()?;
    m.add_class::<python_api::PyCallChoice>()?;
    m.add_class::<python_api::PyCallAdvice>()?;
    m.add_class::<python_api::PyDrillProblem>()?;
    m.add_class::<python_api::PyRuleConfig>()?;
    m.add_class::<python_api::PyMatchContext>()?;
    m.add_class::<python_api::PyWinCondition>()?;
    m.add_class::<python_api::PyPlacementEvaluation>()?;
    m.add_class::<python_api::PyTableState>()?;
    m.add_function(wrap_pyfunction!(python_api::get_all_yaku, m)?)?;
    m.add_function(wrap_pyfunction!(python_api::py_judge_yaku, m)?)?;
    m.add_function(wrap_pyfunction!(python_api::py_calculate_shanten, m)?)?;
    m.add_function(wrap_pyfunction!(python_api::py_evaluate_hand_discards, m)?)?;
    m.add_function(wrap_pyfunction!(python_api::py_advise_call, m)?)?;
    m.add_function(wrap_pyfunction!(python_api::py_generate_drill_problem, m)?)?;
    m.add_function(wrap_pyfunction!(
        python_api::py_evaluate_placement_discards,
        m
    )?)?;
    m.add_function(wrap_pyfunction!(
        python_api::py_calculate_orasu_conditions,
        m
    )?)?;
    m.add_function(wrap_pyfunction!(python_api::py_get_ai_hud_data, m)?)?;
    Ok(())
}
