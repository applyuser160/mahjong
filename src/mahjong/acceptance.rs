use crate::hand::Hand;
use crate::shanten::calculate_shanten_from_counts;
use crate::tile::TileName;

/// 1つの有効牌の情報
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WaitTile {
    pub tile: TileName,
    pub remaining: u8,
}

/// 有効牌（受け入れ）の集計結果
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcceptanceResult {
    /// 現在のシャンテン数
    pub current_shanten: i8,
    /// 有効牌のリスト
    pub waits: Vec<WaitTile>,
    /// 有効牌の総残り枚数
    pub total_remaining: usize,
    /// 有効牌の種類数
    pub tile_types_count: usize,
}

/// 打牌候補ごとの評価結果
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscardAnalysis {
    /// 切る牌
    pub discard_tile: TileName,
    /// 打牌後のシャンテン数
    pub shanten_after: i8,
    /// 打牌後の有効牌一覧
    pub acceptance: AcceptanceResult,
}

/// 13枚の手牌（または手牌カウント）に対して、向聴数を進める有効牌を算出します。
/// `visible_counts`: 手牌・河・副露・ドラ表示牌などですでに見えている牌の枚数（1..=34）。Noneの場合は手牌自身の枚数のみを見えているものとします。
pub fn calculate_acceptance(
    counts: &[u8; 35],
    open_melds_count: usize,
    visible_counts: Option<&[u8; 35]>,
) -> AcceptanceResult {
    let current_res = calculate_shanten_from_counts(counts, open_melds_count);
    let current_shanten = current_res.min_shanten;

    let mut waits = Vec::new();
    let mut total_remaining = 0;

    // 和了形（-1）の場合はこれ以上進まない
    if current_shanten < 0 {
        return AcceptanceResult {
            current_shanten,
            waits,
            total_remaining: 0,
            tile_types_count: 0,
        };
    }

    let default_visible = *counts;
    let visible = visible_counts.unwrap_or(&default_visible);

    let mut working = *counts;

    // 全34種の牌を1枚ずつ仮ツモしてシャンテン数を判定
    for i in 1..=34 {
        if working[i] >= 4 {
            continue; // 手牌に既に4枚ある場合はツモれない
        }

        working[i] += 1;
        let next_res = calculate_shanten_from_counts(&working, open_melds_count);
        working[i] -= 1;

        if next_res.min_shanten < current_shanten {
            let seen = visible[i];
            let remaining = 4u8.saturating_sub(seen);
            let tile = TileName::from_usize(i);

            waits.push(WaitTile { tile, remaining });
            total_remaining += remaining as usize;
        }
    }

    let tile_types_count = waits.len();

    AcceptanceResult {
        current_shanten,
        waits,
        total_remaining,
        tile_types_count,
    }
}

/// 14枚の手牌（ツモ後）に対して、各打牌候補を選んだ場合のシャンテン数および受け入れ枚数を網羅計算します。
/// 結果は受け入れ枚数が多い順（降順）にソートされます。
pub fn analyze_all_discards(
    hand: &Hand,
    visible_counts: Option<&[u8; 35]>,
) -> Vec<DiscardAnalysis> {
    let open_melds_count = hand.open_melds.len();
    let mut results = Vec::new();

    let mut working = hand.counts;

    // 手牌に含まれるユニークな牌を走査
    for i in 1..=34 {
        if working[i] == 0 {
            continue;
        }

        let discard_tile = TileName::from_usize(i);
        working[i] -= 1;

        let acceptance = calculate_acceptance(&working, open_melds_count, visible_counts);
        let shanten_after = acceptance.current_shanten;

        results.push(DiscardAnalysis {
            discard_tile,
            shanten_after,
            acceptance,
        });

        working[i] += 1;
    }

    // ソート順:
    // 1. 打牌後シャンテン数が小さい順（早い順）
    // 2. 受け入れ合計枚数が多い順
    // 3. 有効牌の種類数が多い順
    results.sort_by(|a, b| {
        a.shanten_after
            .cmp(&b.shanten_after)
            .then_with(|| b.acceptance.total_remaining.cmp(&a.acceptance.total_remaining))
            .then_with(|| b.acceptance.tile_types_count.cmp(&a.acceptance.tile_types_count))
    });

    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tile::TileName;

    #[test]
    fn test_ryamen_tenpai_acceptance() {
        // 1m2m3m 4p5p6p 7s8s9s 東東 2s3s (テンパイ, 待ち: 1s, 4s の両面)
        let mut hand = Hand::new();
        for &t in &[
            TileName::OneM,
            TileName::TwoM,
            TileName::ThreeM,
            TileName::FourP,
            TileName::FiveP,
            TileName::SixP,
            TileName::SevenS,
            TileName::EightS,
            TileName::NineS,
            TileName::East,
            TileName::East,
            TileName::TwoS,
            TileName::ThreeS,
        ] {
            hand.push(t);
        }

        let acceptance = calculate_acceptance(&hand.counts, 0, None);
        assert_eq!(acceptance.current_shanten, 0);
        assert_eq!(acceptance.tile_types_count, 2);

        let wait_names: Vec<TileName> = acceptance.waits.iter().map(|w| w.tile).collect();
        assert!(wait_names.contains(&TileName::OneS));
        assert!(wait_names.contains(&TileName::FourS));

        // 手牌に 1s, 4s はないので残り枚数はそれぞれ4枚、計8枚
        assert_eq!(acceptance.total_remaining, 8);
    }

    #[test]
    fn test_kanchan_tenpai_acceptance() {
        // 1m2m3m 4p5p6p 7s8s9s 東東 2s4s (嵌張待ち: 3s)
        let mut hand = Hand::new();
        for &t in &[
            TileName::OneM,
            TileName::TwoM,
            TileName::ThreeM,
            TileName::FourP,
            TileName::FiveP,
            TileName::SixP,
            TileName::SevenS,
            TileName::EightS,
            TileName::NineS,
            TileName::East,
            TileName::East,
            TileName::TwoS,
            TileName::FourS,
        ] {
            hand.push(t);
        }

        let acceptance = calculate_acceptance(&hand.counts, 0, None);
        assert_eq!(acceptance.current_shanten, 0);
        assert_eq!(acceptance.tile_types_count, 1);
        assert_eq!(acceptance.waits[0].tile, TileName::ThreeS);
        assert_eq!(acceptance.total_remaining, 4);
    }

    #[test]
    fn test_analyze_all_discards_14_tiles() {
        // 14枚手牌: 1m2m3m 4p5p6p 7s8s9s 東東 2s3s + 9p (余剰牌 9p を切れば両面テンパイ)
        let mut hand = Hand::new();
        for &t in &[
            TileName::OneM,
            TileName::TwoM,
            TileName::ThreeM,
            TileName::FourP,
            TileName::FiveP,
            TileName::SixP,
            TileName::SevenS,
            TileName::EightS,
            TileName::NineS,
            TileName::East,
            TileName::East,
            TileName::TwoS,
            TileName::ThreeS,
            TileName::NineP, // 余剰牌
        ] {
            hand.push(t);
        }

        let analyses = analyze_all_discards(&hand, None);
        assert!(!analyses.is_empty());

        // 最善打牌は 9p 切り (両面テンパイ: 1s, 4s の計8枚)
        let best = &analyses[0];
        assert_eq!(best.discard_tile, TileName::NineP);
        assert_eq!(best.shanten_after, 0);
        assert_eq!(best.acceptance.total_remaining, 8);
    }
}
