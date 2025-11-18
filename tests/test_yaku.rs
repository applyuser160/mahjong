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
}
