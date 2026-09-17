use criterion::{black_box, criterion_group, criterion_main, Criterion};
use pyo3::prelude::*;

use mahjong::python_api::{
    py_get_ai_hud_data, py_judge_yaku, PyMatchContext, PyMeld, PyRuleConfig, PyTileName,
    PyWinContext,
};

fn bench_py_judge_yaku(c: &mut Criterion) {
    let tiles = vec![
        PyTileName::OneM,
        PyTileName::OneM,
        PyTileName::OneM,
        PyTileName::TwoM,
        PyTileName::ThreeM,
        PyTileName::FourM,
        PyTileName::FiveM,
        PyTileName::SixM,
        PyTileName::SevenM,
        PyTileName::EightM,
        PyTileName::NineM,
        PyTileName::NineM,
        PyTileName::NineM,
        PyTileName::OneM, // win tile
    ];
    let melds = vec![
        PyMeld::pon(PyTileName::East),
        PyMeld::chii(PyTileName::TwoP, [PyTileName::ThreeP, PyTileName::FourP]),
        PyMeld::chii(PyTileName::FiveP, [PyTileName::SixP, PyTileName::SevenP]),
        PyMeld::pon(PyTileName::North),
    ];
    let context = PyWinContext::new(
        true,                   // is_closed
        true,                   // is_tsumo
        None,                   // seat_wind
        None,                   // round_wind
        false,                  // riichi
        0,                      // kan_count
        false,                  // tenhou
        false,                  // chiihou
        Some(PyTileName::OneM), // win_tile
        false,                  // is_rinshan
        false,                  // is_chankan
        false,                  // is_haitei
        false,                  // is_houtei
        false,                  // is_double_riichi
        false,                  // is_ippatsu
    );

    c.bench_function("py_judge_yaku/heavy_melds", |b| {
        b.iter(|| {
            py_judge_yaku(
                black_box(tiles.clone()),
                black_box(melds.clone()),
                black_box(context.clone()),
            )
        })
    });
}

fn bench_py_get_ai_hud_data(c: &mut Criterion) {
    pyo3::prepare_freethreaded_python();

    let hand = vec![
        PyTileName::OneM,
        PyTileName::TwoM,
        PyTileName::ThreeM,
        PyTileName::FourP,
        PyTileName::FiveP,
        PyTileName::SixP,
        PyTileName::SevenS,
        PyTileName::EightS,
        PyTileName::NineS,
        PyTileName::East,
        PyTileName::East,
        PyTileName::White,
        PyTileName::White,
        PyTileName::NineM,
    ];

    let match_ctx = PyMatchContext::new(
        Some([31000, 25000, 24000, 20000]),
        Some(PyTileName::South),
        4, // 南4局 (オーラス)
        1, // 1本場
        1, // 供託1
        0, // 親
        Some(PyRuleConfig::mleague()),
    )
    .unwrap();

    c.bench_function("py_get_ai_hud_data/orasu_14_tiles", |b| {
        b.iter(|| {
            Python::with_gil(|py| {
                py_get_ai_hud_data(
                    py,
                    black_box(hand.clone()),
                    black_box(&match_ctx),
                    black_box(1),
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                )
                .unwrap();
            })
        })
    });
}

criterion_group!(benches, bench_py_judge_yaku, bench_py_get_ai_hud_data);
criterion_main!(benches);
