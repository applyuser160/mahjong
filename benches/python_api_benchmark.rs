use criterion::{black_box, criterion_group, criterion_main, Criterion};

use mahjong::python_api::{py_judge_yaku, PyMeld, PyTileName, PyWinContext};

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

criterion_group!(benches, bench_py_judge_yaku);
criterion_main!(benches);
