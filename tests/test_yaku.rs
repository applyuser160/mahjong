#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use mahjong::tile::TileName::*;
    use mahjong::yaku::{judge_yaku, WinContext, YakuId};

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
        let result = judge_yaku(&tiles, &[], ctx);
        let expected: HashSet<YakuId> =
            HashSet::from([YakuId::Pinfu, YakuId::Tanyao, YakuId::MenzenTsumo]);
        assert!(expected.is_subset(&result));
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
        let result = judge_yaku(&tiles, &open_melds, ctx);
        assert!(result.contains(&YakuId::Suukantsu));
        assert!(!result.contains(&YakuId::Sankantsu)); // Normal yaku should be filtered out
    }

    #[test]
    fn detect_chitoitsu() {
        let tiles = vec![
            OneM, OneM, TwoM, TwoM, ThreeM, ThreeM, FourM, FourM, FiveP, FiveP, SixP, SixP, SevenS,
            SevenS,
        ];

        let result = judge_yaku(&tiles, &[], WinContext::default());
        assert!(result.contains(&YakuId::Chitoitsu));
    }

    #[test]
    fn detect_kokushi() {
        let tiles = vec![
            OneM, NineM, OneP, NineP, OneS, NineS, East, South, West, North, Red, Green, White,
            OneM,
        ];

        let result = judge_yaku(&tiles, &[], WinContext::default());
        assert!(result.contains(&YakuId::KokushiMusou));
    }

    #[test]
    fn detect_daisangen() {
        let tiles = vec![
            Red, Red, Red, Green, Green, Green, White, White, White, OneM, OneM, OneM, TwoM, TwoM,
        ];

        let result = judge_yaku(&tiles, &[], WinContext::default());
        assert!(result.contains(&YakuId::Daisangen));
        assert!(!result.contains(&YakuId::Toitoi));
    }

    #[test]
    fn detect_sanshoku_doujun() {
        let tiles = vec![
            FourM, FiveM, SixM, FourP, FiveP, SixP, FourS, FiveS, SixS, TwoM, TwoM, TwoM, NineP,
            NineP,
        ];

        let result = judge_yaku(&tiles, &[], WinContext::default());
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

        let result = judge_yaku(&tiles, &[], WinContext::default());
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

        let result = judge_yaku(&tiles, &[], WinContext::default());
        assert!(result.contains(&YakuId::Ipeiko));
    }

    #[test]
    fn detect_ryanpeiko() {
        let tiles = vec![
            OneM, OneM, TwoM, TwoM, ThreeM, ThreeM, // double 123m
            FourM, FourM, FiveM, FiveM, SixM, SixM, // double 456m
            SevenM, SevenM, // pair
        ];

        let result = judge_yaku(&tiles, &[], WinContext::default());
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
        let result = judge_yaku(&tiles, &[], ctx);
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
        let result = judge_yaku(&tiles, &[], ctx);
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

        let result = judge_yaku(&tiles, &[], WinContext::default());
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

        let result = judge_yaku(&tiles, &[], WinContext::default());
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

        let result = judge_yaku(&tiles, &[], WinContext::default());
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
        let result = judge_yaku(&tiles, &[], ctx);
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
        let result = judge_yaku(&tiles, &[], ctx);
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
        let result = judge_yaku(&tiles, &open_melds, ctx);
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
        let result = judge_yaku(&tiles, &[], ctx);
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
        let result = judge_yaku(&tiles, &[], ctx);
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
        let result = judge_yaku(&tiles, &[], ctx);
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
        let result = judge_yaku(&tiles, &[], ctx);
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
        let result = judge_yaku(&tiles, &[], ctx);
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
        let result = judge_yaku(&tiles, &[], ctx);
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
        let result = judge_yaku(&tiles, &[], ctx);
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
        let result = judge_yaku(&tiles, &[], ctx);
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
        let result = judge_yaku(&tiles, &[], ctx);
        assert!(!result.contains(&YakuId::Pinfu));
    }
}

#[cfg(test)]
mod tests_kan {
    use mahjong::hand::Meld;
    use mahjong::tile::TileName::*;
    use mahjong::yaku::{judge_yaku, WinContext, YakuId};

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
        let result = judge_yaku(&tiles, &open_melds, ctx);
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
        let result = judge_yaku(&tiles, &open_melds, ctx);
        assert!(result.contains(&YakuId::Sankantsu));
    }

    #[test]
    fn test_yakuman_filters_normal_yaku() {
        let tiles = vec![
            OneM, NineM, OneP, NineP, OneS, NineS, East, South, West, North, Red, Green, White,
            White,
        ];
        // Ensure that normal yaku are filtered out when Yakuman is achieved.
        let mut ctx = WinContext::default();
        ctx.riichi = true;
        ctx.is_closed = true;

        let result = judge_yaku(&tiles, &[], ctx);

        assert!(result.contains(&YakuId::KokushiMusou));
        assert!(!result.contains(&YakuId::Riichi));
    }
}
