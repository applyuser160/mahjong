use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion};
use rand::rngs::StdRng;
use rand::SeedableRng;

use mahjong::hand::{Hand, Meld};
use mahjong::round::Round;
use mahjong::tile::TileName::*;
use mahjong::wall::Wall;
use mahjong::yaku::{judge_yaku, WinContext};

fn bench_yaku(c: &mut Criterion) {
    let mut group = c.benchmark_group("Yaku Evaluation");

    let complex_tiles = vec![
        OneM, OneM, OneM, TwoM, ThreeM, FourM, FiveM, SixM, SevenM, EightM, NineM, NineM, NineM,
    ];
    let complex_win_tile = OneM;
    let mut complex_hand = Hand::new();
    for t in complex_tiles {
        complex_hand.push(t);
    }
    complex_hand.push(complex_win_tile);

    let complex_ctx = WinContext {
        is_closed: true,
        is_tsumo: true,
        win_tile: Some(complex_win_tile),
        ..WinContext::default()
    };

    group.bench_function("Chinitsu (Complex)", |b| {
        b.iter(|| {
            judge_yaku(
                black_box(complex_hand.tiles()),
                black_box(&[] as &[Meld]),
                black_box(complex_ctx.clone()),
            )
        })
    });

    let simple_tiles = vec![
        OneM, TwoM, ThreeM, FourP, FiveP, SixP, SevenS, EightS, NineS, East, East, TwoS, ThreeS,
    ];
    let simple_win_tile = OneS;
    let mut simple_hand = Hand::new();
    for t in simple_tiles {
        simple_hand.push(t);
    }
    simple_hand.push(simple_win_tile);

    let simple_ctx = WinContext {
        is_closed: true,
        is_tsumo: true,
        win_tile: Some(simple_win_tile),
        ..WinContext::default()
    };

    group.bench_function("Pinfu (Simple)", |b| {
        b.iter(|| {
            judge_yaku(
                black_box(simple_hand.tiles()),
                black_box(&[] as &[Meld]),
                black_box(simple_ctx.clone()),
            )
        })
    });

    group.finish();
}

fn bench_round_init(c: &mut Criterion) {
    let mut group = c.benchmark_group("Round Initialization");

    group.bench_function("Round::new", |b| {
        b.iter(|| {
            let mut wall = Wall::new();
            let mut rng = StdRng::seed_from_u64(black_box(42));
            wall.shuffle(&mut rng);
            Round::new(black_box(wall))
        })
    });

    group.finish();
}

fn bench_game_simulation(c: &mut Criterion) {
    let mut group = c.benchmark_group("Game Simulation");

    group.bench_function("Draw and Discard", |b| {
        b.iter_batched(
            || {
                let mut wall = Wall::new();
                let mut rng = StdRng::seed_from_u64(black_box(42));
                wall.shuffle(&mut rng);
                Round::new(wall)
            },
            |mut round| {
                let _drawn = round.draw_tile();
                let _discarded = round.discard_tile(0);
            },
            BatchSize::SmallInput,
        )
    });

    // To properly benchmark the happy path of `play_meld` logic without breaking
    // encapsulation of `Round`, we can measure the underlying `Hand::call_meld`
    // which handles the heavy lifting of state transitions for a successful meld.
    group.bench_function("Call Meld (Pon)", |b| {
        b.iter_batched(
            || {
                let mut hand = Hand::new();
                hand.push(East);
                hand.push(East);
                hand
            },
            |mut hand| {
                let meld = Meld::Pon(East);
                let _ = hand.call_meld(meld);
            },
            BatchSize::SmallInput,
        )
    });

    group.finish();
}

criterion_group!(benches, bench_yaku, bench_round_init, bench_game_simulation);
criterion_main!(benches);
