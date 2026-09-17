use mahjong::hand::Hand;
use mahjong::shanten::{calculate_chitoitsu_shanten, calculate_chitoitsu_shanten_scalar};
use mahjong::suit_table::{encode_suit_key, encode_suit_key_scalar};
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

#[cfg(target_arch = "x86_64")]
use mahjong::shanten::calculate_chitoitsu_shanten_avx2;
#[cfg(target_arch = "x86_64")]
use mahjong::suit_table::encode_suit_key_avx2;

#[test]
fn test_chitoitsu_scalar_vs_avx2_edge_cases() {
    #[cfg(target_arch = "x86_64")]
    {
        if !is_x86_feature_detected!("avx2") {
            println!("Skipping AVX2 test on non-AVX2 hardware");
            return;
        }

        let mut test_cases = Vec::new();

        // 1. 空手牌
        test_cases.push([0u8; 35]);

        // 2. 7対子 (和了形: shanten = -1)
        let mut c = [0u8; 35];
        for item in &mut c[1..=7] {
            *item = 2;
        }
        test_cases.push(c);

        // 3. 6対子 (テンパイ: shanten = 0)
        let mut c = [0u8; 35];
        for item in &mut c[1..=6] {
            *item = 2;
        }
        c[8] = 1;
        test_cases.push(c);

        // 4. 同一牌4枚を含む七対子形 (1枚のみ対子として有効)
        let mut c = [0u8; 35];
        c[1] = 4;
        for item in &mut c[2..=6] {
            *item = 2;
        }
        test_cases.push(c);

        // 5. 境界: 33(発), 34(中) に対子があるケース
        let mut c = [0u8; 35];
        c[33] = 2;
        c[34] = 2;
        for item in &mut c[1..=5] {
            *item = 2;
        }
        test_cases.push(c);

        // 6. 全牌1枚 (13種類: 6 - 0 + (7 - 7) = 6)
        let mut c = [0u8; 35];
        for item in &mut c[1..=13] {
            *item = 1;
        }
        test_cases.push(c);

        // 7. 4枚持ちが多数
        let mut c = [0u8; 35];
        for item in &mut c[1..=8] {
            *item = 4;
        }
        test_cases.push(c);

        for counts in test_cases {
            let scalar_res = calculate_chitoitsu_shanten_scalar(&counts);
            let avx2_res = unsafe { calculate_chitoitsu_shanten_avx2(&counts) };
            let dispatch_res = calculate_chitoitsu_shanten(&counts);
            assert_eq!(scalar_res, avx2_res, "Counts: {:?}", counts);
            assert_eq!(scalar_res, dispatch_res, "Counts: {:?}", counts);
        }
    }
}

#[test]
fn test_chitoitsu_scalar_vs_avx2_random_hands() {
    #[cfg(target_arch = "x86_64")]
    {
        if !is_x86_feature_detected!("avx2") {
            return;
        }

        let mut rng = SmallRng::seed_from_u64(12345);

        // 10,000 件の合法手牌および任意カウント分布で検証
        for _ in 0..10_000 {
            let mut hand = Hand::new();
            // 13〜14枚のランダム牌を配牌
            let num_tiles = rng.gen_range(13..=14);
            let mut available = [4u8; 35];
            let mut dealt = 0;
            while dealt < num_tiles {
                let tid = rng.gen_range(1..=34);
                if available[tid] > 0 {
                    available[tid] -= 1;
                    let t = mahjong::tile::TileName::from_usize(tid);
                    hand.push(t);
                    dealt += 1;
                }
            }

            let scalar = calculate_chitoitsu_shanten_scalar(&hand.counts);
            let avx2 = unsafe { calculate_chitoitsu_shanten_avx2(&hand.counts) };
            let dispatch = calculate_chitoitsu_shanten(&hand.counts);

            assert_eq!(scalar, avx2, "Mismatch on hand counts: {:?}", hand.counts);
            assert_eq!(scalar, dispatch);
        }
    }
}

#[test]
fn test_encode_suit_key_scalar_vs_avx2_exhaustive() {
    #[cfg(target_arch = "x86_64")]
    {
        if !is_x86_feature_detected!("avx2") {
            return;
        }

        // 1. 境界値ケース
        let zero = [0u8; 9];
        assert_eq!(encode_suit_key_scalar(&zero), unsafe {
            encode_suit_key_avx2(&zero)
        });

        let max_val = [4u8; 9];
        assert_eq!(encode_suit_key_scalar(&max_val), unsafe {
            encode_suit_key_avx2(&max_val)
        });

        // 5以上の異常値（min(4) 飽和の検証）
        let overflow = [5, 6, 7, 8, 9, 10, 11, 12, 13];
        assert_eq!(encode_suit_key_scalar(&overflow), unsafe {
            encode_suit_key_avx2(&overflow)
        });

        let mut rng = SmallRng::seed_from_u64(98765);
        // 100,000 パターンのランダムスーツキー検証（オーバーフローケース含む）
        for _ in 0..100_000 {
            let mut counts = [0u8; 9];
            for item in &mut counts {
                *item = rng.gen_range(0..=6);
            }

            let scalar = encode_suit_key_scalar(&counts);
            let avx2 = unsafe { encode_suit_key_avx2(&counts) };
            let dispatch = encode_suit_key(&counts);

            assert_eq!(scalar, avx2, "Mismatch on counts: {:?}", counts);
            assert_eq!(scalar, dispatch);
        }
    }
}
