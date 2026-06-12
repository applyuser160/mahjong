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

    // 1. Open Hand
    let open_tiles = vec![White, White, East, East, East, OneP, TwoP];
    let open_win_tile = ThreeP;
    let mut open_hand = Hand::new();
    for t in open_tiles {
        open_hand.push(t);
    }
    open_hand.push(open_win_tile);

    let open_melds = vec![
        Meld::Pon(South),
        Meld::Chii {
            called: OneM,
            consumed: [TwoM, ThreeM],
        },
    ];

    let open_ctx = WinContext {
        is_closed: false,
        is_tsumo: true,
        win_tile: Some(open_win_tile),
        ..WinContext::default()
    };

    group.bench_function("Open Hand", |b| {
        b.iter(|| {
            judge_yaku(
                black_box(open_hand.tiles()),
                black_box(&open_melds),
                black_box(open_ctx.clone()),
            )
        })
    });

    // 2. Worst Case Branching
    let worst_case_tiles = vec![
        TwoP, TwoP, ThreeP, ThreeP, FourP, FourP, FiveP, FiveP, SixP, SixP, SevenP, SevenP, EightP,
    ];
    let worst_case_win_tile = EightP;
    let mut worst_case_hand = Hand::new();
    for t in worst_case_tiles {
        worst_case_hand.push(t);
    }
    worst_case_hand.push(worst_case_win_tile);

    let worst_case_ctx = WinContext {
        is_closed: true,
        is_tsumo: true,
        win_tile: Some(worst_case_win_tile),
        ..WinContext::default()
    };

    group.bench_function("Worst Case Branching", |b| {
        b.iter(|| {
            judge_yaku(
                black_box(worst_case_hand.tiles()),
                black_box(&[] as &[Meld]),
                black_box(worst_case_ctx.clone()),
            )
        })
    });

    // 3. No Yaku / Fast Reject
    let no_yaku_tiles = vec![
        OneM, FourM, SevenM, OneP, FourP, SevenP, OneS, FourS, SevenS, East, South, West, North,
    ];
    let no_yaku_win_tile = White;
    let mut no_yaku_hand = Hand::new();
    for t in no_yaku_tiles {
        no_yaku_hand.push(t);
    }
    no_yaku_hand.push(no_yaku_win_tile);

    let no_yaku_ctx = WinContext {
        is_closed: true,
        is_tsumo: true,
        win_tile: Some(no_yaku_win_tile),
        ..WinContext::default()
    };

    group.bench_function("No Yaku / Fast Reject", |b| {
        b.iter(|| {
            judge_yaku(
                black_box(no_yaku_hand.tiles()),
                black_box(&[] as &[Meld]),
                black_box(no_yaku_ctx.clone()),
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

    // 4. Kan Simulation
    group.bench_function("Call Meld (Ankan)", |b| {
        b.iter_batched(
            || {
                let mut hand = Hand::new();
                hand.push(East);
                hand.push(East);
                hand.push(East);
                hand.push(East);
                hand
            },
            |mut hand| {
                let meld = Meld::Ankan(East);
                black_box(hand.call_meld(meld))
            },
            BatchSize::SmallInput,
        )
    });

    group.finish();
}

criterion_group!(benches, bench_yaku, bench_round_init, bench_game_simulation);
criterion_main!(benches);
