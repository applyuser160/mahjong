#[allow(unused_macros)]
macro_rules! to_counts {
    ($tiles:expr) => {{
        let mut counts = [0u8; 35];
        for &t in $tiles {
            counts[t as usize] += 1;
        }
        counts
    }};
}
#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use mahjong::tile::TileName::*;
    use mahjong::yaku::{judge_yaku, judge_yaku_set, WinContext, YakuId, YakuSet};

    #[test]
    fn detect_pinfu_and_tanyao() {
        let tiles = vec![
            TwoM, ThreeM, FourM, // 234m
            FourP, FiveP, SixP, // 456p
            ThreeS, FourS, FiveS, // 345s
            SixS, SevenS, EightS, // 678s
            TwoP, TwoP, // pair
        ];

        let ctx = WinContext {
            is_closed: true,
            is_tsumo: true,
            win_tile: Some(FourM),
            ..Default::default()
        };
        // 後方互換性テスト: judge_yaku は HashSet<YakuId> を返却
        let result: HashSet<YakuId> = judge_yaku(&to_counts!(&tiles), &[], ctx);
        let expected: HashSet<YakuId> =
            HashSet::from([YakuId::Pinfu, YakuId::Tanyao, YakuId::MenzenTsumo]);
        assert!(expected.is_subset(&result));

        // 新APIテスト: judge_yaku_set は YakuSet を返却
        let set_result: YakuSet = judge_yaku_set(&to_counts!(&tiles), &[], ctx);
        let set_expected = YakuSet::from([YakuId::Pinfu, YakuId::Tanyao, YakuId::MenzenTsumo]);
        assert!(set_expected.is_subset(&set_result));
    }

    #[test]
    fn detect_suukantsu() {
        let tiles = vec![
            White, White, // pair
        ];

        // 4 Kans -> Suukantsu
        let open_melds = vec![
            mahjong::hand::Meld::Ankan(ThreeP),
            mahjong::hand::Meld::Daiminkan(FourP),
            mahjong::hand::Meld::Kakan(FiveP),
            mahjong::hand::Meld::Ankan(SixP),
        ];

        let ctx = WinContext {
            is_tsumo: true,
            ..Default::default()
        };
        let result = judge_yaku(&to_counts!(&tiles), &open_melds, ctx);
        assert!(result.contains(&YakuId::Suukantsu));
        assert!(!result.contains(&YakuId::Sankantsu)); // Normal yaku should be filtered out
    }

    #[test]
    fn detect_chitoitsu() {
        let tiles = vec![
            OneM, OneM, TwoM, TwoM, ThreeM, ThreeM, FourM, FourM, FiveP, FiveP, SixP, SixP, SevenS,
            SevenS,
        ];

        let result = judge_yaku(&to_counts!(&tiles), &[], WinContext::default());
        assert!(result.contains(&YakuId::Chitoitsu));
    }

    #[test]
    fn detect_kokushi() {
        let tiles = vec![
            OneM, NineM, OneP, NineP, OneS, NineS, East, South, West, North, Red, Green, White,
            OneM,
        ];

        let result = judge_yaku(&to_counts!(&tiles), &[], WinContext::default());
        assert!(result.contains(&YakuId::KokushiMusou));
    }

    #[test]
    fn detect_daisangen() {
        let tiles = vec![
            Red, Red, Red, Green, Green, Green, White, White, White, OneM, OneM, OneM, TwoM, TwoM,
        ];

        let result = judge_yaku(&to_counts!(&tiles), &[], WinContext::default());
        assert!(result.contains(&YakuId::Daisangen));
        assert!(!result.contains(&YakuId::Toitoi));
    }

    #[test]
    fn detect_sanshoku_doujun() {
        let tiles = vec![
            FourM, FiveM, SixM, FourP, FiveP, SixP, FourS, FiveS, SixS, TwoM, TwoM, TwoM, NineP,
            NineP,
        ];

        let result = judge_yaku(&to_counts!(&tiles), &[], WinContext::default());
        assert!(result.contains(&YakuId::SanshokuDoujun));
    }

    #[test]
    fn detect_ipeiko() {
        let tiles = vec![
            OneM, OneM, TwoM, TwoM, ThreeM, ThreeM, // double 123m
            FourP, FiveP, SixP, // 456p
            SevenS, EightS, NineS, // 789s
            FiveS, FiveS, // pair
        ];

        let result = judge_yaku(&to_counts!(&tiles), &[], WinContext::default());
        assert!(result.contains(&YakuId::Ipeiko));
    }

    #[test]
    fn detect_ipeiko_with_triplet() {
        let tiles = vec![
            OneM, OneM, TwoM, TwoM, ThreeM, ThreeM, // double 123m
            FiveP, FiveP, FiveP, // 555p
            SevenS, EightS, NineS, // 789s
            OneS, OneS, // pair
        ];

        let result = judge_yaku(&to_counts!(&tiles), &[], WinContext::default());
        assert!(result.contains(&YakuId::Ipeiko));
    }

    #[test]
    fn detect_ryanpeiko() {
        let tiles = vec![
            OneM, OneM, TwoM, TwoM, ThreeM, ThreeM, // double 123m
            FourM, FourM, FiveM, FiveM, SixM, SixM, // double 456m
            SevenM, SevenM, // pair
        ];

        let result = judge_yaku(&to_counts!(&tiles), &[], WinContext::default());
        assert!(result.contains(&YakuId::Ryanpeiko));
    }

    #[test]
    fn detect_yakuhai_with_seat_and_round_wind() {
        let tiles = vec![
            East, East, East, // seat/round wind triplet
            South, South, South, // additional triplet
            OneM, TwoM, ThreeM, // 123m
            FourP, FiveP, SixP, // 456p
            NineS, NineS, // pair
        ];

        let ctx = WinContext {
            seat_wind: Some(East),
            round_wind: Some(East),
            ..Default::default()
        };
        let result = judge_yaku(&to_counts!(&tiles), &[], ctx);
        assert!(result.contains(&YakuId::YakuhaiJikaze));
        assert!(result.contains(&YakuId::YakuhaiBakaze));
    }

    #[test]
    fn detect_yakuhai_with_round_wind_only() {
        let tiles = vec![
            East, East, East, // round wind triplet
            OneM, TwoM, ThreeM, // 123m
            FourM, FiveM, SixM, // 456m
            FourP, FiveP, SixP, // 456p
            NineS, NineS, // pair
        ];

        let ctx = WinContext {
            seat_wind: Some(South), // seat wind is South
            round_wind: Some(East), // round wind is East
            ..Default::default()
        };
        let result = judge_yaku(&to_counts!(&tiles), &[], ctx);
        // It should contain Bakaze (round wind), but not Jikaze (seat wind)
        assert!(result.contains(&YakuId::YakuhaiBakaze));
        assert!(!result.contains(&YakuId::YakuhaiJikaze));
    }

    #[test]
    fn detect_honitsu() {
        let tiles = vec![
            OneM, TwoM, ThreeM, // 123m
            FourM, FiveM, SixM, // 456m
            SevenM, EightM, NineM, // 789m
            OneM, OneM, OneM, // 111m
            White, White, // pair
        ];

        let result = judge_yaku(&to_counts!(&tiles), &[], WinContext::default());
        assert!(result.contains(&YakuId::Honitsu));
    }

    #[test]
    fn detect_sanshoku_doukou() {
        let tiles = vec![
            FiveM, FiveM, FiveM, // 555m
            FiveP, FiveP, FiveP, // 555p
            FiveS, FiveS, FiveS, // 555s
            OneM, TwoM, ThreeM, // 123m
            East, East, // pair
        ];

        let result = judge_yaku(&to_counts!(&tiles), &[], WinContext::default());
        assert!(result.contains(&YakuId::SanshokuDoukou));
    }

    #[test]
    fn detect_chinroutou() {
        let tiles = vec![
            OneM, OneM, OneM, // 111m
            NineM, NineM, NineM, // 999m
            OneP, OneP, OneP, // 111p
            NineS, NineS, NineS, // 999s
            OneS, OneS, // pair
        ];

        let result = judge_yaku(&to_counts!(&tiles), &[], WinContext::default());
        assert!(result.contains(&YakuId::Chinroutou));
        assert!(!result.contains(&YakuId::Honroutou));
    }

    #[test]
    fn detect_sanankou_with_undeclared_quad() {
        let tiles = vec![
            TwoM, TwoM, TwoM, TwoM, // 2222m (used as 222m triplet + 2m for sequence)
            ThreeM, FourM, // 34m (completed as 234m sequence)
            SixP, SixP, SixP, // 666p
            NineS, NineS, NineS, // 999s
            East, East, // pair
        ];

        let ctx = WinContext {
            is_tsumo: true,
            is_closed: true,
            ..Default::default()
        };
        let result = judge_yaku(&to_counts!(&tiles), &[], ctx);
        // This hand is Sanankou (222m, 666p, 999s are closed triplets).
        assert!(result.contains(&YakuId::Sanankou));
    }

    #[test]
    fn detect_sanankou_tsumo() {
        let tiles = vec![
            OneM, OneM, OneM, // 111m
            TwoM, TwoM, TwoM, // 222m
            ThreeM, ThreeM, ThreeM, // 333m
            FourM, FiveM, SixM, // 456m
            White, White, // pair
        ];

        let ctx = WinContext {
            is_tsumo: true,
            ..Default::default()
        };
        let result = judge_yaku(&to_counts!(&tiles), &[], ctx);
        assert!(result.contains(&YakuId::Sanankou));
    }

    #[test]
    fn detect_sanankou_fails_with_open_meld() {
        let tiles = vec![
            OneM, OneM, OneM, // 111m
            TwoM, TwoM, TwoM, // 222m
            FourM, FiveM, SixM, // 456m
            White, White, // pair
        ];

        let open_melds = vec![mahjong::hand::Meld::Pon(ThreeM)];

        let ctx = WinContext {
            is_tsumo: true, // Tsumo shouldn't matter if we rely on closed melds
            ..Default::default()
        };
        let result = judge_yaku(&to_counts!(&tiles), &open_melds, ctx);
        // We only have 2 closed triplets (111m, 222m) and 1 open triplet (333m).
        assert!(!result.contains(&YakuId::Sanankou));
    }

    #[test]
    fn detect_sanankou_fails_on_ron() {
        let tiles = vec![
            OneM, OneM, OneM, // 111m
            TwoM, TwoM, TwoM, // 222m
            ThreeM, ThreeM, ThreeM, // 333m
            FourM, FiveM, SixM, // 456m
            White, White, // pair
        ];

        let ctx = WinContext {
            is_tsumo: false,
            win_tile: Some(ThreeM), // ron on 3m, making 333m open
            ..Default::default()
        };
        let result = judge_yaku(&to_counts!(&tiles), &[], ctx);
        assert!(!result.contains(&YakuId::Sanankou));
    }

    #[test]
    fn detect_sanankou_passes_on_ron_sequence() {
        let tiles = vec![
            OneM, OneM, OneM, // 111m
            TwoM, TwoM, TwoM, // 222m
            ThreeM, ThreeM, ThreeM, // 333m
            FourM, FiveM, SixM, // 456m
            White, White, // pair
        ];

        let ctx = WinContext {
            is_tsumo: false,
            win_tile: Some(FourM), // ron on 4m, making 456m open, but triplets remain closed
            ..Default::default()
        };
        let result = judge_yaku(&to_counts!(&tiles), &[], ctx);
        assert!(result.contains(&YakuId::Sanankou));
    }

    #[test]
    fn detect_suuankou_ron_tanki() {
        let tiles = vec![
            OneM, OneM, OneM, // 111m
            TwoM, TwoM, TwoM, // 222m
            ThreeM, ThreeM, ThreeM, // 333m
            FourP, FourP, FourP, // 444p
            White, White, // pair
        ];

        let ctx = WinContext {
            is_tsumo: false,
            win_tile: Some(White), // ron on pair (Tanki), triplets remain closed
            ..Default::default()
        };
        let result = judge_yaku(&to_counts!(&tiles), &[], ctx);
        assert!(result.contains(&YakuId::Suuankou));
    }

    #[test]
    fn detect_suuankou_fails_on_ron_shanpon() {
        let tiles = vec![
            OneM, OneM, OneM, // 111m
            TwoM, TwoM, TwoM, // 222m
            ThreeM, ThreeM, ThreeM, // 333m
            FourP, FourP, FourP, // 444p
            White, White, // pair
        ];

        let ctx = WinContext {
            is_tsumo: false,
            win_tile: Some(FourP), // ron on a triplet (Shanpon), downgrades to Sanankou + Toitoi
            ..Default::default()
        };
        let result = judge_yaku(&to_counts!(&tiles), &[], ctx);
        assert!(!result.contains(&YakuId::Suuankou));
        assert!(result.contains(&YakuId::Sanankou));
        assert!(result.contains(&YakuId::Toitoi));
    }

    #[test]
    fn detect_pinfu_ryamen() {
        let tiles = vec![
            TwoM, ThreeM, FourM, // 234m
            FourP, FiveP, SixP, // 456p
            ThreeS, FourS, FiveS, // 345s
            SixS, SevenS, EightS, // 678s
            TwoP, TwoP, // pair
        ];

        let ctx = WinContext {
            is_closed: true,
            is_tsumo: false,
            win_tile: Some(FourM), // ryamen wait on 1m or 4m
            ..Default::default()
        };
        let result = judge_yaku(&to_counts!(&tiles), &[], ctx);
        assert!(result.contains(&YakuId::Pinfu));
    }

    #[test]
    fn detect_pinfu_fails_kanchan() {
        let tiles = vec![
            TwoM, ThreeM, FourM, // 234m
            FourP, FiveP, SixP, // 456p
            ThreeS, FourS, FiveS, // 345s
            SixS, SevenS, EightS, // 678s
            TwoP, TwoP, // pair
        ];

        let ctx = WinContext {
            is_closed: true,
            is_tsumo: false,
            win_tile: Some(ThreeM), // kanchan wait on 3m (2m 4m wait for 3m)
            ..Default::default()
        };
        let result = judge_yaku(&to_counts!(&tiles), &[], ctx);
        assert!(!result.contains(&YakuId::Pinfu));
    }

    #[test]
    fn detect_pinfu_fails_penchan() {
        let tiles = vec![
            OneM, TwoM, ThreeM, // 123m
            FourP, FiveP, SixP, // 456p
            ThreeS, FourS, FiveS, // 345s
            SixS, SevenS, EightS, // 678s
            TwoP, TwoP, // pair
        ];

        let ctx = WinContext {
            is_closed: true,
            is_tsumo: false,
            win_tile: Some(ThreeM), // penchan wait on 3m (1m 2m wait for 3m)
            ..Default::default()
        };
        let result = judge_yaku(&to_counts!(&tiles), &[], ctx);
        assert!(!result.contains(&YakuId::Pinfu));
    }

    #[test]
    fn detect_pinfu_fails_nobetan() {
        let tiles = vec![
            TwoM, ThreeM, FourM, FiveM, // 2345m (nobetan)
            FourP, FiveP, SixP, // 456p
            ThreeS, FourS, FiveS, // 345s
            SixS, SevenS, EightS, // 678s
            TwoM,   // The drawn tile
        ];

        let ctx = WinContext {
            is_closed: true,
            is_tsumo: false,
            win_tile: Some(TwoM), // nobetan wait on 2m or 5m (acts as tanki pair)
            ..Default::default()
        };
        let result = judge_yaku(&to_counts!(&tiles), &[], ctx);
        assert!(!result.contains(&YakuId::Pinfu));
    }

    #[test]
    fn detect_pinfu_fails_tanki_with_complete_sequence() {
        let tiles = vec![
            TwoM, ThreeM, FourM, // complete sequence 234m
            FourP, FiveP, SixP, // 456p
            ThreeS, FourS, FiveS, // 345s
            SixS, SevenS, EightS, // 678s
            FourM, FourM, // pair 4m (win_tile is 4m, so it completed the pair)
        ];

        let ctx = WinContext {
            is_closed: true,
            is_tsumo: false,
            win_tile: Some(FourM), // tanki wait on 4m
            ..Default::default()
        };
        let result = judge_yaku(&to_counts!(&tiles), &[], ctx);
        assert!(!result.contains(&YakuId::Pinfu));
    }
}

#[cfg(test)]
mod tests_kan {
    use mahjong::hand::Meld;
    use mahjong::tile::TileName::*;
    use mahjong::yaku::{judge_yaku, WinContext, YakuId, YakuSet};

    #[test]
    fn detect_sanankou_passes_with_ankan() {
        let tiles = vec![
            OneM, OneM, OneM, // 111m
            TwoM, TwoM, TwoM, // 222m
            FourM, FiveM, SixM, // 456m
            White, White, // pair
        ];

        let open_melds = vec![Meld::Ankan(ThreeM)];

        let ctx = WinContext {
            is_tsumo: true,
            ..Default::default()
        };
        let result = judge_yaku(&to_counts!(&tiles), &open_melds, ctx);
        // Ankan is a closed meld, so we have 3 closed triplets/quads (111m, 222m, 3333m)
        assert!(result.contains(&YakuId::Sanankou));
    }

    #[test]
    fn detect_sankantsu() {
        let tiles = vec![
            OneM, TwoM, ThreeM, // 123m
            White, White, // pair
        ];

        // 3 Kans -> Sankantsu
        let open_melds = vec![
            mahjong::hand::Meld::Ankan(ThreeP),
            mahjong::hand::Meld::Daiminkan(FourP),
            mahjong::hand::Meld::Kakan(FiveP),
        ];

        let ctx = WinContext {
            is_tsumo: true,
            ..Default::default()
        };
        let result = judge_yaku(&to_counts!(&tiles), &open_melds, ctx);
        assert!(result.contains(&YakuId::Sankantsu));
    }

    #[test]
    fn test_yakuman_filters_normal_yaku() {
        let tiles = vec![
            OneM, NineM, OneP, NineP, OneS, NineS, East, South, West, North, Red, Green, White,
            White,
        ];
        // Ensure that normal yaku are filtered out when Yakuman is achieved.
        let ctx = WinContext {
            riichi: true,
            is_closed: true,
            ..WinContext::default()
        };

        let result = judge_yaku(&to_counts!(&tiles), &[], ctx);

        assert!(result.contains(&YakuId::KokushiMusou));
        assert!(!result.contains(&YakuId::Riichi));
    }

    #[test]
    fn test_yakuset_operations() {
        let mut set = YakuSet::empty();
        assert!(set.is_empty());
        assert_eq!(set.len(), 0);

        set.insert(YakuId::Riichi);
        set.insert(YakuId::Tanyao);
        assert!(!set.is_empty());
        assert_eq!(set.len(), 2);
        assert!(set.contains(&YakuId::Riichi));
        assert!(set.contains(&YakuId::Tanyao));
        assert!(!set.contains(&YakuId::Pinfu));

        set.remove(YakuId::Riichi);
        assert_eq!(set.len(), 1);
        assert!(!set.contains(&YakuId::Riichi));
        assert!(set.contains(&YakuId::Tanyao));

        // Union & Intersection
        let set_a = YakuSet::from([YakuId::Riichi, YakuId::Ippatsu]);
        let set_b = YakuSet::from([YakuId::Ippatsu, YakuId::Tanyao]);
        let union_set = set_a | set_b;
        assert_eq!(union_set.len(), 3);
        assert!(union_set.contains(&YakuId::Riichi));
        assert!(union_set.contains(&YakuId::Ippatsu));
        assert!(union_set.contains(&YakuId::Tanyao));

        let inter_set = set_a & set_b;
        assert_eq!(inter_set.len(), 1);
        assert!(inter_set.contains(&YakuId::Ippatsu));

        // Subset
        assert!(inter_set.is_subset(&set_a));
        assert!(inter_set.is_subset(&set_b));
        assert!(!set_a.is_subset(&inter_set));

        // Iteration
        let collected: Vec<YakuId> = union_set.into_iter().collect();
        assert_eq!(collected.len(), 3);

        // All 41 Yaku IDs roundtrip
        for (i, &yaku_id) in mahjong::yaku::ALL_YAKU_IDS.iter().enumerate() {
            assert_eq!(yaku_id as u8, i as u8);
            assert_eq!(YakuId::from_u8(i as u8), Some(yaku_id));
        }

        // Yakuman retain
        let mut mixed = YakuSet::from([YakuId::Riichi, YakuId::Daisangen, YakuId::Tanyao]);
        mixed.retain_yakuman_only();
        assert_eq!(mixed.len(), 1);
        assert!(mixed.contains(&YakuId::Daisangen));
        assert!(!mixed.contains(&YakuId::Riichi));

        // Out-of-bounds bitmask handling (PR #101 review [P2])
        let out_of_bounds = YakuSet::from_raw(1u64 << 63);
        assert!(out_of_bounds.is_empty());
        assert_eq!(out_of_bounds.len(), 0);
        assert_eq!(out_of_bounds.as_raw(), 0);
        assert_eq!(out_of_bounds.iter().count(), 0);
        assert_eq!(out_of_bounds.iter().len(), 0);

        assert_eq!(YakuSet::from_raw_checked(1u64 << 63), Option::None);
        assert_eq!(YakuSet::from_raw_checked(1u64 << 41), Option::None);
        let max_valid = (1u64 << 41) - 1;
        assert!(YakuSet::from_raw_checked(max_valid).is_some());
        let full_set = YakuSet::from_raw_checked(max_valid).unwrap();
        assert_eq!(full_set.len(), 41);
        assert_eq!(full_set.iter().len(), 41);
        assert_eq!(full_set.iter().count(), 41);
    }

    #[test]
    fn test_is_number_tile_exhaustive() {
        use mahjong::tile::TileName;
        use mahjong::yaku::is_number_tile;

        assert_eq!(is_number_tile(TileName::None), Option::None);

        // 萬子 (1m..=9m)
        for rank in 1..=9 {
            let tile = TileName::from_usize(rank);
            assert_eq!(is_number_tile(tile), Some((0, rank)));
        }

        // 筒子 (1p..=9p)
        for rank in 1..=9 {
            let tile = TileName::from_usize(9 + rank);
            assert_eq!(is_number_tile(tile), Some((1, rank)));
        }

        // 索子 (1s..=9s)
        for rank in 1..=9 {
            let tile = TileName::from_usize(18 + rank);
            assert_eq!(is_number_tile(tile), Some((2, rank)));
        }

        // 字牌 (東..=白)
        for idx in 28..=34 {
            let tile = TileName::from_usize(idx);
            assert_eq!(is_number_tile(tile), Option::None);
        }
    }
}
