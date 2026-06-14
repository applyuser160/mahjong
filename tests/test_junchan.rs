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
    use mahjong::tile::TileName::*;
    use mahjong::yaku::{judge_yaku, WinContext, YakuId};

    #[test]
    fn test_junchan_invalid_pair() {
        // Test case where pair is non-terminal (e.g. 5m) but melds all have terminals
        // This should NOT be junchan.
        let tiles = vec![
            OneM, TwoM, ThreeM, // 123m
            SevenM, EightM, NineM, // 789m
            OneP, TwoP, ThreeP, // 123p
            SevenS, EightS, NineS, // 789s
            FiveM, FiveM, // 5m pair - Invalid for junchan
        ];

        let ctx = WinContext {
            is_closed: true,
            is_tsumo: true,
            win_tile: Some(OneM),
            ..Default::default()
        };
        let result = judge_yaku(&to_counts!(&tiles), &[], ctx);
        assert!(
            !result.contains(&YakuId::Junchan),
            "Junchan should not be valid with a non-terminal pair"
        );
    }

    #[test]
    fn test_junchan_valid_pair() {
        // Test case where pair is terminal (e.g. 1m) and melds all have terminals
        // This SHOULD be junchan.
        let tiles = vec![
            OneM, TwoM, ThreeM, // 123m
            SevenM, EightM, NineM, // 789m
            OneP, TwoP, ThreeP, // 123p
            SevenS, EightS, NineS, // 789s
            NineP, NineP, // 9p pair - Valid for junchan
        ];

        let ctx = WinContext {
            is_closed: true,
            is_tsumo: true,
            win_tile: Some(OneM),
            ..Default::default()
        };
        let result = judge_yaku(&to_counts!(&tiles), &[], ctx);
        assert!(
            result.contains(&YakuId::Junchan),
            "Junchan should be valid with a terminal pair"
        );
    }
}
