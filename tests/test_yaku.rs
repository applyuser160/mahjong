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
            ..Default::default()
        };
        let result = judge_yaku(&tiles, &[], ctx);
        let expected: HashSet<YakuId> =
            HashSet::from([YakuId::Pinfu, YakuId::Tanyao, YakuId::MenzenTsumo]);
        assert!(expected.is_subset(&result));
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
        assert!(result.contains(&YakuId::Toitoi));
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
        assert!(result.contains(&YakuId::Honroutou));
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
            ron_tile: Some(ThreeM), // ron on 3m, making 333m open
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
            ron_tile: Some(FourM), // ron on 4m, making 456m open, but triplets remain closed
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
            ron_tile: Some(White), // ron on pair (Tanki), triplets remain closed
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
            ron_tile: Some(FourP), // ron on a triplet (Shanpon), downgrades to Sanankou + Toitoi
            ..Default::default()
        };
        let result = judge_yaku(&tiles, &[], ctx);
        assert!(!result.contains(&YakuId::Suuankou));
        assert!(result.contains(&YakuId::Sanankou));
        assert!(result.contains(&YakuId::Toitoi));
    }
}
