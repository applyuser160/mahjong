use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion};
use rand::rngs::SmallRng;
use rand::SeedableRng;

use mahjong::acceptance::{analyze_all_discards, calculate_acceptance};
use mahjong::hand::{Hand, Meld};
use mahjong::round::Round;
use mahjong::shanten::calculate_shanten;
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
                black_box(&complex_hand.counts),
                black_box(&[] as &[Meld]),
                black_box(complex_ctx),
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
                black_box(&simple_hand.counts),
                black_box(&[] as &[Meld]),
                black_box(simple_ctx),
            )
        })
    });

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
                black_box(&open_hand.counts),
                black_box(&open_melds),
                black_box(open_ctx),
            )
        })
    });

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
                black_box(&worst_case_hand.counts),
                black_box(&[] as &[Meld]),
                black_box(worst_case_ctx),
            )
        })
    });

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
                black_box(&no_yaku_hand.counts),
                black_box(&[] as &[Meld]),
                black_box(no_yaku_ctx),
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
            let mut rng = SmallRng::seed_from_u64(black_box(42));
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
                let mut rng = SmallRng::seed_from_u64(black_box(42));
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

    group.bench_function("Call Meld (Chii)", |b| {
        b.iter_batched(
            || {
                let mut hand = Hand::new();
                hand.push(OneM);
                hand.push(TwoM);
                hand
            },
            |mut hand| {
                let meld = Meld::Chii {
                    called: ThreeM,
                    consumed: [OneM, TwoM],
                };
                let _ = hand.call_meld(meld);
            },
            BatchSize::SmallInput,
        )
    });

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

    group.bench_function("Round Play Meld (Fail Setup)", |b| {
        b.iter_batched(
            || {
                let mut wall = Wall::new();
                let mut rng = SmallRng::seed_from_u64(42);
                wall.shuffle(&mut rng);
                let mut round = Round::new(wall);

                // Let's just deal some cards, doesn't matter what, we just want to benchmark the failure case or basic case
                // Or we can draw a tile to make it our turn
                round.draw_tile();
                round
            },
            |mut round| {
                let meld = Meld::Ankan(East);
                // It might fail if we don't have the tiles, but we're mostly testing the setup/clone cost which happens before tile checks
                let _ = round.play_meld(0, meld);
            },
            BatchSize::SmallInput,
        )
    });

    group.finish();
}

fn bench_shanten_and_acceptance(c: &mut Criterion) {
    let mut group = c.benchmark_group("Shanten and Acceptance");

    // 1. Shanten Calculation
    let mut iishanten_hand = Hand::new();
    for &t in &[
        OneM, TwoM, ThreeM, FourP, FiveP, SixP, TwoS, ThreeS, SevenS, EightS, East, East, West,
    ] {
        iishanten_hand.push(t);
    }
    group.bench_function("calculate_shanten (Iishanten)", |b| {
        b.iter(|| calculate_shanten(black_box(&iishanten_hand)))
    });

    let mut chinitsu_hand = Hand::new();
    for &t in &[
        OneM, OneM, OneM, TwoM, ThreeM, FourM, FiveM, SixM, SevenM, EightM, NineM, NineM, NineM,
    ] {
        chinitsu_hand.push(t);
    }
    group.bench_function("calculate_shanten (Complex Chinitsu)", |b| {
        b.iter(|| calculate_shanten(black_box(&chinitsu_hand)))
    });

    let mut chitoi_hand = Hand::new();
    for &t in &[
        OneM, OneM, ThreeM, ThreeM, FiveP, FiveP, SevenP, SevenP, NineS, NineS, East, East, White,
    ] {
        chitoi_hand.push(t);
    }
    group.bench_function("calculate_shanten (Chitoitsu Tenpai)", |b| {
        b.iter(|| calculate_shanten(black_box(&chitoi_hand)))
    });

    // 2. Acceptance Calculation
    let mut tenpai_hand = Hand::new();
    for &t in &[
        OneM, TwoM, ThreeM, FourP, FiveP, SixP, SevenS, EightS, NineS, East, East, TwoS, ThreeS,
    ] {
        tenpai_hand.push(t);
    }
    group.bench_function("calculate_acceptance (Ryamen Tenpai)", |b| {
        b.iter(|| {
            calculate_acceptance(
                black_box(&tenpai_hand.counts),
                black_box(0),
                black_box(std::option::Option::None),
            )
        })
    });

    group.bench_function("calculate_acceptance (Iishanten 4-waits)", |b| {
        b.iter(|| {
            calculate_acceptance(
                black_box(&iishanten_hand.counts),
                black_box(0),
                black_box(std::option::Option::None),
            )
        })
    });

    let mut chuuren_hand = Hand::new();
    for &t in &[
        OneM, OneM, OneM, TwoM, ThreeM, FourM, FiveM, SixM, SevenM, EightM, NineM, NineM, NineM,
    ] {
        chuuren_hand.push(t);
    }
    group.bench_function("calculate_acceptance (Chuuren 9-waits)", |b| {
        b.iter(|| {
            calculate_acceptance(
                black_box(&chuuren_hand.counts),
                black_box(0),
                black_box(std::option::Option::None),
            )
        })
    });

    let mut naked_hand = Hand::new();
    naked_hand.push(East);
    group.bench_function("calculate_acceptance (Naked Tanki 4-melds)", |b| {
        b.iter(|| {
            calculate_acceptance(
                black_box(&naked_hand.counts),
                black_box(4),
                black_box(std::option::Option::None),
            )
        })
    });

    // 3. Analyze All Discards
    let mut discards_hand = Hand::new();
    for &t in &[
        OneM, TwoM, ThreeM, FourP, FiveP, SixP, SevenS, EightS, NineS, East, East, TwoS, ThreeS,
        NineP,
    ] {
        discards_hand.push(t);
    }
    group.bench_function("analyze_all_discards (14-tiles Hand)", |b| {
        b.iter(|| {
            analyze_all_discards(
                black_box(&discards_hand),
                black_box(std::option::Option::None),
            )
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_yaku,
    bench_round_init,
    bench_game_simulation,
    bench_shanten_and_acceptance
);
criterion_main!(benches);
