use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rand::rngs::StdRng;
use rand::SeedableRng;

use mahjong::hand::{Hand, Meld};
use mahjong::round::Round;
use mahjong::tile::TileName::*;
use mahjong::wall::Wall;
use mahjong::yaku::{judge_yaku, WinContext};

fn bench_yaku(c: &mut Criterion) {
    let mut group = c.benchmark_group("Yaku Evaluation");

    // Case 1: Chinitsu (Complex)
    // 1112345678999m waiting for 123456789m (Chuuren Poutou like structure)
    // We construct a Hand with these tiles.
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
        ..Default::default()
    };

    group.bench_function("Chinitsu (Complex)", |b| {
        b.iter(|| judge_yaku(black_box(complex_hand.tiles()), black_box(&[]), black_box(complex_ctx.clone())))
    });

    // Case 2: Pinfu (Simple)
    // 123m 456p 789s 11z waiting for 23s, win tile 1s
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
        ..Default::default()
    };

    group.bench_function("Pinfu (Simple)", |b| {
        b.iter(|| judge_yaku(black_box(simple_hand.tiles()), black_box(&[]), black_box(simple_ctx.clone())))
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

    group.bench_function("Draw, Discard, and Meld", |b| {
        b.iter(|| {
            let mut wall = Wall::new();
            let mut rng = StdRng::seed_from_u64(black_box(42));
            wall.shuffle(&mut rng);
            let mut round = Round::new(wall);

            // Turn 0: Draw and discard
            let _drawn = round.draw_tile().unwrap();
            let discarded = round.discard_tile(0).unwrap();

            // Player 1 plays Pon on the discarded tile
            // For a successful Pon, the hand must have two matching tiles.
            // In a real scenario, this is verified by Meld::Pon logic if strictly checked,
            // but `play_meld` primarily checks turn flow and state updates.
            // Let's simulate the play_meld call.
            let meld = Meld::Pon(discarded);
            let _ = round.play_meld(1, meld);

            // Because play_meld for Pon changes turn to the meld caller (Player 1),
            // Player 1 now needs to discard.
            let _ = round.discard_tile(0);
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_yaku,
    bench_round_init,
    bench_game_simulation
);
criterion_main!(benches);
