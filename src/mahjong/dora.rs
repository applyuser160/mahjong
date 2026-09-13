use crate::tile::TileName;

/// ドラ表示牌に対応するドラ牌を返します。
pub fn indicator_to_dora(indicator: TileName) -> TileName {
    match indicator {
        // 萬子 (1m -> 2m -> ... -> 9m -> 1m)
        TileName::OneM => TileName::TwoM,
        TileName::TwoM => TileName::ThreeM,
        TileName::ThreeM => TileName::FourM,
        TileName::FourM => TileName::FiveM,
        TileName::FiveM => TileName::SixM,
        TileName::SixM => TileName::SevenM,
        TileName::SevenM => TileName::EightM,
        TileName::EightM => TileName::NineM,
        TileName::NineM => TileName::OneM,

        // 筒子 (1p -> 2p -> ... -> 9p -> 1p)
        TileName::OneP => TileName::TwoP,
        TileName::TwoP => TileName::ThreeP,
        TileName::ThreeP => TileName::FourP,
        TileName::FourP => TileName::FiveP,
        TileName::FiveP => TileName::SixP,
        TileName::SixP => TileName::SevenP,
        TileName::SevenP => TileName::EightP,
        TileName::EightP => TileName::NineP,
        TileName::NineP => TileName::OneP,

        // 索子 (1s -> 2s -> ... -> 9s -> 1s)
        TileName::OneS => TileName::TwoS,
        TileName::TwoS => TileName::ThreeS,
        TileName::ThreeS => TileName::FourS,
        TileName::FourS => TileName::FiveS,
        TileName::FiveS => TileName::SixS,
        TileName::SixS => TileName::SevenS,
        TileName::SevenS => TileName::EightS,
        TileName::EightS => TileName::NineS,
        TileName::NineS => TileName::OneS,

        // 風牌 (東 -> 南 -> 西 -> 北 -> 東)
        TileName::East => TileName::South,
        TileName::South => TileName::West,
        TileName::West => TileName::North,
        TileName::North => TileName::East,

        // 三元牌 (白 -> 發 -> 中 -> 白)
        TileName::White => TileName::Green,
        TileName::Green => TileName::Red,
        TileName::Red => TileName::White,

        TileName::None => TileName::None,
    }
}

/// 手牌カウント（1..=34）に含まれるドラの総枚数をカウントします。
/// `dora_indicators`: ドラ表示牌の一覧（表ドラ・裏ドラ・槓ドラ等）
pub fn count_dora(counts: &[u8; 35], dora_indicators: &[TileName]) -> usize {
    let mut total_dora = 0;
    for &ind in dora_indicators {
        let dora_tile = indicator_to_dora(ind);
        let idx = dora_tile as usize;
        if idx < counts.len() {
            total_dora += counts[idx] as usize;
        }
    }
    total_dora
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_indicator_to_dora() {
        assert_eq!(indicator_to_dora(TileName::OneM), TileName::TwoM);
        assert_eq!(indicator_to_dora(TileName::NineM), TileName::OneM);
        assert_eq!(indicator_to_dora(TileName::NineP), TileName::OneP);
        assert_eq!(indicator_to_dora(TileName::NineS), TileName::OneS);
        assert_eq!(indicator_to_dora(TileName::North), TileName::East);
        assert_eq!(indicator_to_dora(TileName::Red), TileName::White);
    }

    #[test]
    fn test_count_dora() {
        let mut counts = [0u8; 35];
        counts[TileName::TwoM as usize] = 3; // 2m が 3枚
        counts[TileName::White as usize] = 2; // 白 が 2枚

        let indicators = [TileName::OneM, TileName::Red]; // ドラは 2m と 白
        let total = count_dora(&counts, &indicators);
        assert_eq!(total, 5); // 3 + 2 = 5
    }
}
