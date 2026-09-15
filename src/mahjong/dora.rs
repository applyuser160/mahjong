use crate::tile::TileName;

const DORA_INDICATOR_TABLE: [TileName; 35] = [
    TileName::None,
    // 萬子 (1m..=9m) -> 2m..=9m, 1m
    TileName::TwoM,
    TileName::ThreeM,
    TileName::FourM,
    TileName::FiveM,
    TileName::SixM,
    TileName::SevenM,
    TileName::EightM,
    TileName::NineM,
    TileName::OneM,
    // 筒子 (1p..=9p) -> 2p..=9p, 1p
    TileName::TwoP,
    TileName::ThreeP,
    TileName::FourP,
    TileName::FiveP,
    TileName::SixP,
    TileName::SevenP,
    TileName::EightP,
    TileName::NineP,
    TileName::OneP,
    // 索子 (1s..=9s) -> 2s..=9s, 1s
    TileName::TwoS,
    TileName::ThreeS,
    TileName::FourS,
    TileName::FiveS,
    TileName::SixS,
    TileName::SevenS,
    TileName::EightS,
    TileName::NineS,
    TileName::OneS,
    // 風牌 (東 -> 南 -> 西 -> 北 -> 東)
    TileName::South,
    TileName::West,
    TileName::North,
    TileName::East,
    // 三元牌 (白 -> 發 -> 中 -> 白)
    TileName::White, // Red(32: 中) -> White(34: 白)
    TileName::Red,   // Green(33: 發) -> Red(32: 中)
    TileName::Green, // White(34: 白) -> Green(33: 發)
];

/// ドラ表示牌に対応するドラ牌を返します。
#[inline]
pub fn indicator_to_dora(indicator: TileName) -> TileName {
    let idx = indicator as usize;
    if idx < DORA_INDICATOR_TABLE.len() {
        DORA_INDICATOR_TABLE[idx]
    } else {
        TileName::None
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
