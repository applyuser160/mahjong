#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use mahjong::tile::TileName;
    use mahjong::tile::TileName::*;
    use mahjong::yaku::{judge_yaku, WinContext, YakuId};

    fn assert_detects_all(id: YakuId, hands: &[(Vec<TileName>, WinContext)]) {
        for (tiles, ctx) in hands {
            let result = judge_yaku(tiles, *ctx);
            assert!(
                result.contains(&id),
                "expected {:?} for hand {:?} with ctx {:?}",
                id,
                tiles,
                ctx
            );
        }
    }

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
        let result = judge_yaku(&tiles, ctx);
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

        let result = judge_yaku(&tiles, WinContext::default());
        assert!(result.contains(&YakuId::Chitoitsu));
    }

    #[test]
    fn detect_kokushi() {
        let tiles = vec![
            OneM, NineM, OneP, NineP, OneS, NineS, East, South, West, North, Red, Green, White,
            OneM,
        ];

        let result = judge_yaku(&tiles, WinContext::default());
        assert!(result.contains(&YakuId::KokushiMusou));
    }

    #[test]
    fn detect_daisangen() {
        let tiles = vec![
            Red, Red, Red, Green, Green, Green, White, White, White, OneM, OneM, OneM, TwoM, TwoM,
        ];

        let result = judge_yaku(&tiles, WinContext::default());
        assert!(result.contains(&YakuId::Daisangen));
        assert!(result.contains(&YakuId::Toitoi));
    }

    #[test]
    fn detect_sanshoku_doujun() {
        let tiles = vec![
            FourM, FiveM, SixM, FourP, FiveP, SixP, FourS, FiveS, SixS, TwoM, TwoM, TwoM, NineP,
            NineP,
        ];

        let result = judge_yaku(&tiles, WinContext::default());
        assert!(result.contains(&YakuId::SanshokuDoujun));
    }

    #[test]
    fn detect_every_yaku_with_multiple_examples() {
        let cases: Vec<(YakuId, Vec<(Vec<TileName>, WinContext)>)> = vec![
            (
                YakuId::Riichi,
                vec![
                    (
                        vec![
                            OneM, TwoM, ThreeM, FourM, FiveM, SixM, SevenP, EightP, NineP, SevenS, EightS,
                            NineS, TwoP, TwoP,
                        ],
                        WinContext {
                            riichi: true,
                            ..Default::default()
                        },
                    ),
                    (
                        vec![
                            TwoM, ThreeM, FourM, ThreeP, FourP, FiveP, SixS, SevenS, EightS, SixP, SixP,
                            NineM, NineM, NineM,
                        ],
                        WinContext {
                            riichi: true,
                            ..Default::default()
                        },
                    ),
                ],
            ),
            (
                YakuId::MenzenTsumo,
                vec![
                    (
                        vec![
                            TwoM, ThreeM, FourM, FiveM, SixM, SevenM, SevenP, EightP, NineP, TwoS, ThreeS,
                            FourS, FiveP, FiveP,
                        ],
                        WinContext {
                            is_tsumo: true,
                            ..Default::default()
                        },
                    ),
                    (
                        vec![
                            ThreeM, FourM, FiveM, TwoP, ThreeP, FourP, TwoS, ThreeS, FourS, EightM, EightM,
                            EightM, FiveS, FiveS,
                        ],
                        WinContext {
                            is_tsumo: true,
                            ..Default::default()
                        },
                    ),
                ],
            ),
            (
                YakuId::Tanyao,
                vec![
                    (
                        vec![
                            TwoM, ThreeM, FourM, ThreeP, FourP, FiveP, SixP, SixP, SixP, FourS, FiveS, SixS,
                            SevenS, SevenS,
                        ],
                        WinContext::default(),
                    ),
                    (
                        vec![
                            TwoP, ThreeP, FourP, ThreeS, FourS, FiveS, SixM, SevenM, EightM, SixS, SixS,
                            SevenP, SevenP, SevenP,
                        ],
                        WinContext::default(),
                    ),
                ],
            ),
            (
                YakuId::Pinfu,
                vec![
                    (
                        vec![
                            TwoM, ThreeM, FourM, FiveM, SixM, SevenM, ThreeP, FourP, FiveP, FourS, FiveS,
                            SixS, SevenS, SevenS,
                        ],
                        WinContext::default(),
                    ),
                    (
                        vec![
                            ThreeM, FourM, FiveM, SixM, SevenM, EightM, TwoP, ThreeP, FourP, FiveS, SixS,
                            SevenS, FourS, FourS,
                        ],
                        WinContext::default(),
                    ),
                ],
            ),
            (
                YakuId::Ipeiko,
                vec![
                    (
                        vec![
                            TwoM, ThreeM, FourM, TwoM, ThreeM, FourM, FiveP, SixP, SevenP, ThreeS, FourS,
                            FiveS, SixS, SixS,
                        ],
                        WinContext::default(),
                    ),
                    (
                        vec![
                            ThreeP, FourP, FiveP, ThreeP, FourP, FiveP, SixM, SevenM, EightM, SixS, SixS,
                            SevenS, EightS, EightS,
                        ],
                        WinContext::default(),
                    ),
                ],
            ),
            (
                YakuId::YakuhaiHaku,
                vec![
                    (
                        vec![
                            White, White, White, TwoM, TwoM, TwoM, ThreeP, FourP, FiveP, SixS, SevenS,
                            EightS, NineM, NineM,
                        ],
                        WinContext::default(),
                    ),
                    (
                        vec![
                            White, White, White, OneP, TwoP, ThreeP, FourP, FiveP, SixP, SevenS, EightS,
                            NineS, ThreeM, ThreeM,
                        ],
                        WinContext::default(),
                    ),
                ],
            ),
            (
                YakuId::YakuhaiHatsu,
                vec![
                    (
                        vec![
                            Green, Green, Green, ThreeM, FourM, FiveM, SixM, SevenM, EightM, OneS, TwoS,
                            ThreeS, FourP, FourP,
                        ],
                        WinContext::default(),
                    ),
                    (
                        vec![
                            Green, Green, Green, OneM, OneM, OneM, SevenP, EightP, NineP, TwoS, ThreeS,
                            FourS, SixP, SixP,
                        ],
                        WinContext::default(),
                    ),
                ],
            ),
            (
                YakuId::YakuhaiChun,
                vec![
                    (
                        vec![
                            Red, Red, Red, TwoM, ThreeM, FourM, ThreeP, ThreeP, ThreeP, SixS, SevenS,
                            EightS, NineM, NineM,
                        ],
                        WinContext::default(),
                    ),
                    (
                        vec![
                            Red, Red, Red, SevenM, EightM, NineM, FourP, FiveP, SixP, SevenS, EightS,
                            NineS, SixM, SixM,
                        ],
                        WinContext::default(),
                    ),
                ],
            ),
            (
                YakuId::YakuhaiJikaze,
                vec![
                    (
                        vec![
                            East, East, East, TwoM, ThreeM, FourM, ThreeP, FourP, FiveP, SevenS, EightS,
                            NineS, FiveM, FiveM,
                        ],
                        WinContext {
                            seat_wind: Some(East),
                            ..Default::default()
                        },
                    ),
                    (
                        vec![
                            South, South, South, TwoP, ThreeP, FourP, FourS, FiveS, SixS, OneM, TwoM, ThreeM,
                            EightP, EightP,
                        ],
                        WinContext {
                            seat_wind: Some(South),
                            ..Default::default()
                        },
                    ),
                ],
            ),
            (
                YakuId::YakuhaiBakaze,
                vec![
                    (
                        vec![
                            West, West, West, ThreeM, FourM, FiveM, SixM, SixM, SixM, ThreeP, FourP, FiveP,
                            TwoS, TwoS,
                        ],
                        WinContext {
                            round_wind: Some(West),
                            ..Default::default()
                        },
                    ),
                    (
                        vec![
                            North, North, North, TwoM, ThreeM, FourM, FiveP, SixP, SevenP, TwoS, ThreeS,
                            FourS, EightM, EightM,
                        ],
                        WinContext {
                            round_wind: Some(North),
                            ..Default::default()
                        },
                    ),
                ],
            ),
            (
                YakuId::Chitoitsu,
                vec![
                    (
                        vec![
                            OneM, OneM, TwoM, TwoM, ThreeM, ThreeM, FourM, FourM, FiveP, FiveP, SixP, SixP,
                            SevenS, SevenS,
                        ],
                        WinContext::default(),
                    ),
                    (
                        vec![
                            ThreeP, ThreeP, FourP, FourP, FiveP, FiveP, SixS, SixS, SevenS, SevenS, EightM,
                            EightM, NineM, NineM,
                        ],
                        WinContext::default(),
                    ),
                ],
            ),
            (
                YakuId::Toitoi,
                vec![
                    (
                        vec![
                            TwoM, TwoM, TwoM, ThreeP, ThreeP, ThreeP, FourS, FourS, FourS, SixM, SixM,
                            SixM, EightP, EightP,
                        ],
                        WinContext {
                            is_closed: false,
                            ..Default::default()
                        },
                    ),
                    (
                        vec![
                            OneP, OneP, OneP, FiveM, FiveM, FiveM, SevenS, SevenS, SevenS, NineP, NineP,
                            NineP, East, East,
                        ],
                        WinContext {
                            is_closed: false,
                            ..Default::default()
                        },
                    ),
                ],
            ),
            (
                YakuId::Sanankou,
                vec![
                    (
                        vec![
                            TwoM, TwoM, TwoM, ThreeP, ThreeP, ThreeP, FourS, FourS, FourS, SixM, SevenM,
                            EightM, NineP, NineP,
                        ],
                        WinContext::default(),
                    ),
                    (
                        vec![
                            SevenM, SevenM, SevenM, EightP, EightP, EightP, NineS, NineS, NineS, OneM, TwoM,
                            ThreeM, FourP, FourP,
                        ],
                        WinContext::default(),
                    ),
                ],
            ),
            (
                YakuId::Shousangen,
                vec![
                    (
                        vec![
                            Red, Red, Red, Green, Green, Green, White, White, TwoM, ThreeM, FourM, SevenP,
                            EightP, NineP,
                        ],
                        WinContext::default(),
                    ),
                    (
                        vec![
                            Red, Red, Red, Green, Green, White, White, White, OneS, TwoS, ThreeS, SixP,
                            SixP, SixP,
                        ],
                        WinContext::default(),
                    ),
                ],
            ),
            (
                YakuId::Chantaiyao,
                vec![
                    (
                        vec![
                            OneM, TwoM, ThreeM, SevenP, EightP, NineP, OneS, TwoS, ThreeS, East, East, East,
                            NineM, NineM,
                        ],
                        WinContext {
                            is_closed: false,
                            ..Default::default()
                        },
                    ),
                    (
                        vec![
                            OneP, TwoP, ThreeP, SevenS, EightS, NineS, OneM, OneM, OneM, NineM, NineM,
                            NineM, White, White,
                        ],
                        WinContext {
                            is_closed: false,
                            ..Default::default()
                        },
                    ),
                ],
            ),
            (
                YakuId::Ryanpeiko,
                vec![
                    (
                        vec![
                            OneM, TwoM, ThreeM, OneM, TwoM, ThreeM, FourM, FiveM, SixM, FourM, FiveM, SixM,
                            SevenM, SevenM,
                        ],
                        WinContext::default(),
                    ),
                    (
                        vec![
                            TwoP, ThreeP, FourP, TwoP, ThreeP, FourP, SixP, SevenP, EightP, SixP, SevenP,
                            EightP, FiveS, FiveS,
                        ],
                        WinContext::default(),
                    ),
                ],
            ),
            (
                YakuId::SanshokuDoujun,
                vec![
                    (
                        vec![
                            OneM, TwoM, ThreeM, OneP, TwoP, ThreeP, OneS, TwoS, ThreeS, FourM, FiveM, SixM,
                            SevenP, SevenP,
                        ],
                        WinContext::default(),
                    ),
                    (
                        vec![
                            SixM, SevenM, EightM, SixP, SevenP, EightP, SixS, SevenS, EightS, TwoM, TwoM,
                            TwoM, NineS, NineS,
                        ],
                        WinContext::default(),
                    ),
                ],
            ),
            (
                YakuId::SanshokuDoukou,
                vec![
                    (
                        vec![
                            ThreeM, ThreeM, ThreeM, ThreeP, ThreeP, ThreeP, ThreeS, ThreeS, ThreeS, FiveM,
                            SixM, SevenM, EightP, EightP,
                        ],
                        WinContext::default(),
                    ),
                    (
                        vec![
                            SevenM, SevenM, SevenM, SevenP, SevenP, SevenP, SevenS, SevenS, SevenS, TwoM,
                            ThreeM, FourM, NineS, NineS,
                        ],
                        WinContext::default(),
                    ),
                ],
            ),
            (
                YakuId::Honitsu,
                vec![
                    (
                        vec![
                            OneM, TwoM, ThreeM, FourM, FiveM, SixM, SevenM, EightM, NineM, East, East, East,
                            White, White,
                        ],
                        WinContext {
                            is_closed: false,
                            ..Default::default()
                        },
                    ),
                    (
                        vec![
                            TwoP, ThreeP, FourP, FiveP, SixP, SevenP, EightP, EightP, EightP, North, North,
                            North, Red, Red,
                        ],
                        WinContext {
                            is_closed: false,
                            ..Default::default()
                        },
                    ),
                ],
            ),
            (
                YakuId::Junchan,
                vec![
                    (
                        vec![
                            OneM, TwoM, ThreeM, SevenM, EightM, NineM, OneP, TwoP, ThreeP, NineP, NineP,
                            NineP, OneS, OneS,
                        ],
                        WinContext::default(),
                    ),
                    (
                        vec![
                            OneS, TwoS, ThreeS, SevenS, EightS, NineS, OneM, OneM, OneM, NineM, NineM,
                            NineM, OneP, OneP,
                        ],
                        WinContext::default(),
                    ),
                ],
            ),
            (
                YakuId::Chinitsu,
                vec![
                    (
                        vec![
                            TwoM, ThreeM, FourM, FourM, FourM, FiveM, SixM, SevenM, EightM, EightM, EightM,
                            NineM, NineM, NineM,
                        ],
                        WinContext::default(),
                    ),
                    (
                        vec![
                            OneS, TwoS, ThreeS, FourS, FiveS, SixS, SevenS, SevenS, SevenS, EightS, EightS,
                            EightS, NineS, NineS,
                        ],
                        WinContext {
                            is_closed: false,
                            ..Default::default()
                        },
                    ),
                ],
            ),
            (
                YakuId::Chinroutou,
                vec![
                    (
                        vec![
                            OneM, OneM, OneM, NineM, NineM, NineM, OneP, OneP, OneP, NineP, NineP, NineP,
                            OneS, OneS,
                        ],
                        WinContext::default(),
                    ),
                    (
                        vec![
                            OneS, OneS, OneS, NineS, NineS, NineS, OneM, OneM, OneM, NineM, NineM, NineM,
                            NineP, NineP,
                        ],
                        WinContext::default(),
                    ),
                ],
            ),
            (
                YakuId::Honroutou,
                vec![
                    (
                        vec![
                            OneM, OneM, OneM, NineM, NineM, NineM, East, East, East, White, White, White,
                            OneP, OneP,
                        ],
                        WinContext::default(),
                    ),
                    (
                        vec![
                            OneS, OneS, OneS, NineS, NineS, NineS, North, North, North, Red, Red, Red,
                            NineP, NineP,
                        ],
                        WinContext::default(),
                    ),
                ],
            ),
            (
                YakuId::Sankantsu,
                vec![
                    (
                        vec![
                            TwoM, TwoM, TwoM, ThreeM, ThreeM, ThreeM, FourM, FourM, FourM, FiveP, FiveP,
                            FiveP, SixS, SixS,
                        ],
                        WinContext {
                            kan_count: 3,
                            ..Default::default()
                        },
                    ),
                    (
                        vec![
                            SevenM, SevenM, SevenM, EightM, EightM, EightM, NineM, NineM, NineM, TwoP, ThreeP,
                            FourP, OneS, OneS,
                        ],
                        WinContext {
                            kan_count: 4,
                            ..Default::default()
                        },
                    ),
                ],
            ),
            (
                YakuId::KokushiMusou,
                vec![
                    (
                        vec![
                            OneM, NineM, OneP, NineP, OneS, NineS, East, South, West, North, Red, Green,
                            White, OneM,
                        ],
                        WinContext::default(),
                    ),
                    (
                        vec![
                            OneM, NineM, OneP, NineP, OneS, NineS, East, South, West, North, Red, Green,
                            White, NineS,
                        ],
                        WinContext::default(),
                    ),
                ],
            ),
            (
                YakuId::Suuankou,
                vec![
                    (
                        vec![
                            TwoM, TwoM, TwoM, ThreeP, ThreeP, ThreeP, FourS, FourS, FourS, SixM, SixM,
                            SixM, FiveP, FiveP,
                        ],
                        WinContext::default(),
                    ),
                    (
                        vec![
                            SevenM, SevenM, SevenM, EightP, EightP, EightP, NineS, NineS, NineS, OneM, OneM,
                            OneM, TwoS, TwoS,
                        ],
                        WinContext::default(),
                    ),
                ],
            ),
            (
                YakuId::Daisangen,
                vec![
                    (
                        vec![
                            Red, Red, Red, Green, Green, Green, White, White, White, OneM, TwoM, ThreeM,
                            FourP, FourP,
                        ],
                        WinContext::default(),
                    ),
                    (
                        vec![
                            Red, Red, Red, Green, Green, Green, White, White, White, SixM, SevenM, EightM,
                            NineS, NineS,
                        ],
                        WinContext::default(),
                    ),
                ],
            ),
            (
                YakuId::Shousuushi,
                vec![
                    (
                        vec![
                            East, East, East, South, South, South, West, West, West, North, North, ThreeM,
                            FourM, FiveM,
                        ],
                        WinContext::default(),
                    ),
                    (
                        vec![
                            East, East, East, South, South, South, West, West, West, North, North, TwoP,
                            ThreeP, FourP,
                        ],
                        WinContext::default(),
                    ),
                ],
            ),
            (
                YakuId::Daisuushi,
                vec![
                    (
                        vec![
                            East, East, East, South, South, South, West, West, West, North, North, North,
                            FiveM, FiveM,
                        ],
                        WinContext::default(),
                    ),
                    (
                        vec![
                            East, East, East, South, South, South, West, West, West, North, North, North,
                            SevenP, SevenP,
                        ],
                        WinContext::default(),
                    ),
                ],
            ),
            (
                YakuId::Tsuuiisou,
                vec![
                    (
                        vec![
                            East, East, East, South, South, South, West, West, West, North, North, North,
                            Red, Red,
                        ],
                        WinContext::default(),
                    ),
                    (
                        vec![
                            Red, Red, Red, Green, Green, Green, White, White, White, East, East, East, North,
                            North,
                        ],
                        WinContext::default(),
                    ),
                ],
            ),
            (
                YakuId::Ryuuiisou,
                vec![
                    (
                        vec![
                            TwoS, TwoS, TwoS, ThreeS, ThreeS, ThreeS, FourS, FourS, FourS, SixS, SixS,
                            SixS, Green, Green,
                        ],
                        WinContext::default(),
                    ),
                    (
                        vec![
                            TwoS, ThreeS, FourS, SixS, SixS, SixS, EightS, EightS, EightS, TwoS, ThreeS,
                            FourS, Green, Green,
                        ],
                        WinContext::default(),
                    ),
                ],
            ),
            (
                YakuId::ChuurenPoutou,
                vec![
                    (
                        vec![
                            OneM, OneM, OneM, TwoM, ThreeM, FourM, FiveM, SixM, SevenM, EightM, NineM, NineM,
                            NineM, NineM,
                        ],
                        WinContext::default(),
                    ),
                    (
                        vec![
                            OneP, OneP, OneP, TwoP, ThreeP, FourP, FiveP, SixP, SevenP, EightP, NineP, NineP,
                            NineP, NineP,
                        ],
                        WinContext::default(),
                    ),
                ],
            ),
            (
                YakuId::Tenhou,
                vec![
                    (
                        vec![
                            TwoM, ThreeM, FourM, FiveM, SixM, SevenM, ThreeP, ThreeP, ThreeP, FourS, FiveS,
                            SixS, SevenS, SevenS,
                        ],
                        WinContext {
                            tenhou: true,
                            is_tsumo: true,
                            ..Default::default()
                        },
                    ),
                    (
                        vec![
                            OneM, TwoM, ThreeM, TwoP, ThreeP, FourP, SevenP, SevenP, SevenP, SixS, SevenS,
                            EightS, SixM, SixM,
                        ],
                        WinContext {
                            tenhou: true,
                            is_tsumo: true,
                            ..Default::default()
                        },
                    ),
                ],
            ),
            (
                YakuId::Chiihou,
                vec![
                    (
                        vec![
                            ThreeM, FourM, FiveM, FourP, FiveP, SixP, TwoS, ThreeS, FourS, SevenM, SevenM,
                            SevenM, NineP, NineP,
                        ],
                        WinContext {
                            chiihou: true,
                            is_tsumo: true,
                            ..Default::default()
                        },
                    ),
                    (
                        vec![
                            OneP, TwoP, ThreeP, SevenP, EightP, NineP, ThreeS, ThreeS, ThreeS, TwoM, TwoM,
                            TwoM, FiveS, FiveS,
                        ],
                        WinContext {
                            chiihou: true,
                            is_tsumo: true,
                            ..Default::default()
                        },
                    ),
                ],
            ),
        ];

        for (id, hands) in cases {
            assert_eq!(hands.len(), 2);
            assert_detects_all(id, &hands);
        }
    }
}
