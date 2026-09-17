use criterion::{black_box, criterion_group, criterion_main, Criterion};
use mahjong::hand::Hand;
use mahjong::shanten::{
    calculate_chitoitsu_shanten, calculate_chitoitsu_shanten_scalar,
};
use mahjong::suit_table::{
    encode_suit_key, encode_suit_key_scalar,
};
use mahjong::tile::TileName::*;

#[cfg(target_arch = "x86_64")]
use mahjong::shanten::calculate_chitoitsu_shanten_avx2;
#[cfg(target_arch = "x86_64")]
use mahjong::suit_table::encode_suit_key_avx2;

fn bench_chitoitsu(c: &mut Criterion) {
    let mut group = c.benchmark_group("Chitoitsu Shanten");

    // テンパイ手牌
    let mut chitoi_hand = Hand::new();
    for &t in &[
        OneM, OneM, ThreeM, ThreeM, FiveP, FiveP, SevenP, SevenP, NineS, NineS, East, East, White,
    ] {
        chitoi_hand.push(t);
    }
    let counts = chitoi_hand.counts;

    group.bench_function("Scalar", |b| {
        b.iter(|| calculate_chitoitsu_shanten_scalar(black_box(&counts)))
    });

    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx2") {
            group.bench_function("AVX2 (Direct)", |b| {
                b.iter(|| unsafe { calculate_chitoitsu_shanten_avx2(black_box(&counts)) })
            });
        }
    }

    group.bench_function("Dispatch (calculate_chitoitsu_shanten)", |b| {
        b.iter(|| calculate_chitoitsu_shanten(black_box(&counts)))
    });

    group.finish();
}

fn bench_suit_key(c: &mut Criterion) {
    let mut group = c.benchmark_group("Suit Key Encoding");

    let counts = [1u8, 2, 0, 3, 1, 4, 0, 1, 2];

    group.bench_function("Scalar", |b| {
        b.iter(|| encode_suit_key_scalar(black_box(&counts)))
    });

    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx2") {
            group.bench_function("AVX2 (Direct)", |b| {
                b.iter(|| unsafe { encode_suit_key_avx2(black_box(&counts)) })
            });
        }
    }

    group.bench_function("Dispatch (encode_suit_key)", |b| {
        b.iter(|| encode_suit_key(black_box(&counts)))
    });

    group.finish();
}

criterion_group!(benches, bench_chitoitsu, bench_suit_key);
criterion_main!(benches);