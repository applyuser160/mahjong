use crate::hand::Hand;
use crate::shanten::calculate_shanten_from_counts;
use crate::tile::TileName;
use arrayvec::ArrayVec;

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
    pub waits: ArrayVec<WaitTile, 34>,
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

    let mut waits = ArrayVec::<WaitTile, 34>::new();
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

    // 候補評価では、打牌前の14枚の手牌（および外部指定の可視牌）をすべて可視牌として含める
    let mut base_visible = [0u8; 35];
    if let Some(v) = visible_counts {
        for i in 1..=34 {
            base_visible[i] = v[i].max(hand.counts[i]);
        }
    } else {
        base_visible = hand.counts;
    }

    let mut working = hand.counts;

    // 手牌に含まれるユニークな牌を走査
    for i in 1..=34 {
        if working[i] == 0 {
            continue;
        }

        let discard_tile = TileName::from_usize(i);
        working[i] -= 1;

        // 切った牌を含む base_visible を可視牌として渡すことで、切った牌の残り枚数も正しく減算される
        let acceptance = calculate_acceptance(&working, open_melds_count, Some(&base_visible));
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
            .then_with(|| {
                b.acceptance
                    .total_remaining
                    .cmp(&a.acceptance.total_remaining)
            })
            .then_with(|| {
                b.acceptance
                    .tile_types_count
                    .cmp(&a.acceptance.tile_types_count)
            })
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

    #[test]
    fn test_discarded_tile_included_in_visible_counts() {
        // 2m2m3m4m (4枚) + 4p5p6p + 7s8s9s + 東東東 (13枚) + 1s (ツモ)
        // ここで 2m を切ると、手牌は 2m 3m 4m で雀頭なし、または 3m4m 待ち (2m, 5m)
        // 手牌に元々2枚あった 2m のうち 1枚を切った場合、
        // 切った 2m (1枚) + 残った手牌の 2m (1枚) = 計2枚 が可視なので、残り 2m は 4 - 2 = 2枚になるべき！
        let mut hand = Hand::new();
        for &t in &[
            TileName::TwoM,
            TileName::TwoM,
            TileName::ThreeM,
            TileName::FourM,
            TileName::FourP,
            TileName::FiveP,
            TileName::SixP,
            TileName::SevenS,
            TileName::EightS,
            TileName::NineS,
            TileName::East,
            TileName::East,
            TileName::East,
            TileName::OneS,
        ] {
            hand.push(t);
        }

        let analyses = analyze_all_discards(&hand, None);
        // 2m を切った分析結果を探す
        let analysis_2m = analyses
            .iter()
            .find(|a| a.discard_tile == TileName::TwoM)
            .expect("Should contain 2m discard analysis");

        // もし 2m が受け入れ牌に含まれているなら、残り枚数は手牌(1枚)+切った牌(1枚)を除いた2枚であること
        if let Some(wait_2m) = analysis_2m
            .acceptance
            .waits
            .iter()
            .find(|w| w.tile == TileName::TwoM)
        {
            assert_eq!(
                wait_2m.remaining, 2,
                "Discarded 2m must be counted in visible tiles (remaining 4 - 2 = 2)"
            );
        }
    }

    #[test]
    fn test_hand_four_copies_excluded() {
        // 2m2m2m2m (4枚) + 4p5p6p (3枚) + 7s8s9s (3枚) + 2s3s4s (3枚) = 13枚
        // 4面子 (4p5p6p, 7s8s9s, 2s3s4s, 2m2m2m) 完成しているが雀頭がない手牌。
        // 残り1枚の 2m が雀頭候補の単騎待ちの形だが、2m は既に手牌に4枚あるためツモれない！
        // したがって 2m は有効牌リストから除外され、ツモれる有効牌は存在しない (0種0枚)
        let mut hand = Hand::new();
        for &t in &[
            TileName::TwoM,
            TileName::TwoM,
            TileName::TwoM,
            TileName::TwoM,
            TileName::FourP,
            TileName::FiveP,
            TileName::SixP,
            TileName::SevenS,
            TileName::EightS,
            TileName::NineS,
            TileName::TwoS,
            TileName::ThreeS,
            TileName::FourS,
        ] {
            hand.push(t);
        }

        let acceptance = calculate_acceptance(&hand.counts, 0, None);
        assert_eq!(acceptance.current_shanten, 0); // テンパイ形
        assert_eq!(acceptance.tile_types_count, 0);
        assert_eq!(acceptance.waits.len(), 0);
        assert_eq!(acceptance.total_remaining, 0);
    }

    #[test]
    fn test_zero_remaining_visible_tiles() {
        // 1m2m3m 4p5p6p 7s8s9s 東東 2s3s (テンパイ, 待ち: 1s, 4s)
        let mut counts = [0u8; 35];
        for &idx in &[1, 2, 3, 13, 14, 15, 25, 26, 27, 28, 28, 20, 21] {
            counts[idx] += 1;
        }

        // 外部可視牌として 1s が4枚すべて見えている状態を設定
        let mut visible = counts;
        visible[19] = 4; // 1s = 19

        let acceptance = calculate_acceptance(&counts, 0, Some(&visible));
        assert_eq!(acceptance.current_shanten, 0);
        assert_eq!(acceptance.tile_types_count, 2); // 1s (0枚) と 4s (4枚) の2種

        let wait_1s = acceptance
            .waits
            .iter()
            .find(|w| w.tile == TileName::OneS)
            .unwrap();
        assert_eq!(wait_1s.remaining, 0, "1s must have 0 remaining");

        let wait_4s = acceptance
            .waits
            .iter()
            .find(|w| w.tile == TileName::FourS)
            .unwrap();
        assert_eq!(wait_4s.remaining, 4, "4s must have 4 remaining");

        // 合計枚数は 0 + 4 = 4枚
        assert_eq!(acceptance.total_remaining, 4);
    }

    #[test]
    fn test_completely_empty_remaining_waits() {
        // 1m2m3m 4p5p6p 7s8s9s 東東 2s3s (テンパイ, 待ち: 1s, 4s)
        let mut counts = [0u8; 35];
        for &idx in &[1, 2, 3, 13, 14, 15, 25, 26, 27, 28, 28, 20, 21] {
            counts[idx] += 1;
        }

        // 1s も 4s も場に4枚すべて見えている（完全純カラ・ヤマゼロ）
        let mut visible = counts;
        visible[19] = 4; // 1s
        visible[22] = 4; // 4s

        let acceptance = calculate_acceptance(&counts, 0, Some(&visible));
        assert_eq!(acceptance.current_shanten, 0);
        assert_eq!(acceptance.tile_types_count, 2);
        assert_eq!(acceptance.total_remaining, 0);
        for w in &acceptance.waits {
            assert_eq!(w.remaining, 0);
        }
    }

    #[test]
    fn test_visible_counts_overflow_safe() {
        // visible_counts に誤って 5 以上の値が渡された場合の saturating_sub 安全性検証
        let mut counts = [0u8; 35];
        counts[20] = 1; // 2s
        counts[21] = 1; // 3s
        counts[28] = 2; // 東東
                        // 他面子
        counts[1] = 3;
        counts[4] = 3;
        counts[7] = 3;

        let mut visible = counts;
        visible[19] = 10; // 1s に異常値 10

        let acceptance = calculate_acceptance(&counts, 0, Some(&visible));
        let wait_1s = acceptance
            .waits
            .iter()
            .find(|w| w.tile == TileName::OneS)
            .unwrap();
        assert_eq!(wait_1s.remaining, 0);
    }

    #[test]
    fn test_acceptance_already_agari() {
        // 和了形（-1向聴）の手牌に対して即座に空結果が返ること
        let mut counts = [0u8; 35];
        counts[1] = 3; // 1m1m1m
        counts[4] = 3; // 4m4m4m
        counts[7] = 3; // 7m7m7m
        counts[10] = 3; // 1p1p1p
        counts[28] = 2; // 東東

        let acceptance = calculate_acceptance(&counts, 0, None);
        assert_eq!(acceptance.current_shanten, -1);
        assert_eq!(acceptance.tile_types_count, 0);
        assert_eq!(acceptance.total_remaining, 0);
        assert!(acceptance.waits.is_empty());
    }

    #[test]
    fn test_acceptance_open_melds_1_to_4() {
        // 1副露 (手牌10枚, チー: 7s8s9s): 1m2m3m 4p5p6p 東東 2s3s (テンパイ, 待ち 1s, 4s)
        let mut counts_1meld = [0u8; 35];
        counts_1meld[1] = 1;
        counts_1meld[2] = 1;
        counts_1meld[3] = 1;
        counts_1meld[13] = 1;
        counts_1meld[14] = 1;
        counts_1meld[15] = 1;
        counts_1meld[28] = 2; // 東東
        counts_1meld[20] = 1; // 2s
        counts_1meld[21] = 1; // 3s

        let acc_1 = calculate_acceptance(&counts_1meld, 1, None);
        assert_eq!(acc_1.current_shanten, 0);
        assert_eq!(acc_1.tile_types_count, 2);
        assert_eq!(acc_1.total_remaining, 8);

        // 2副露 (手牌7枚): 4p5p6p 東東 2s4s (嵌張待ち: 3s)
        let mut counts_2melds = [0u8; 35];
        counts_2melds[13] = 1;
        counts_2melds[14] = 1;
        counts_2melds[15] = 1;
        counts_2melds[28] = 2; // 東東
        counts_2melds[20] = 1; // 2s
        counts_2melds[22] = 1; // 4s

        let acc_2 = calculate_acceptance(&counts_2melds, 2, None);
        assert_eq!(acc_2.current_shanten, 0);
        assert_eq!(acc_2.tile_types_count, 1);
        assert_eq!(acc_2.waits[0].tile, TileName::ThreeS);
        assert_eq!(acc_2.total_remaining, 4);

        // 3副露 (手牌4枚): 東東 2s3s (両面待ち: 1s, 4s)
        let mut counts_3melds = [0u8; 35];
        counts_3melds[28] = 2; // 東東
        counts_3melds[20] = 1; // 2s
        counts_3melds[21] = 1; // 3s

        let acc_3 = calculate_acceptance(&counts_3melds, 3, None);
        assert_eq!(acc_3.current_shanten, 0);
        assert_eq!(acc_3.tile_types_count, 2);
        assert_eq!(acc_3.total_remaining, 8);

        // 4副露 (手牌1枚: 裸単騎): 東単騎
        let mut counts_4melds = [0u8; 35];
        counts_4melds[28] = 1; // 東1枚

        let acc_4 = calculate_acceptance(&counts_4melds, 4, None);
        assert_eq!(acc_4.current_shanten, 0); // 単騎テンパイ
        assert_eq!(acc_4.tile_types_count, 1);
        assert_eq!(acc_4.waits[0].tile, TileName::East);
        // 手牌に1枚あるので、残り枚数は 4 - 1 = 3枚
        assert_eq!(acc_4.waits[0].remaining, 3);
        assert_eq!(acc_4.total_remaining, 3);
    }

    #[test]
    fn test_acceptance_sanmenchan() {
        // 3面張: 2m3m4m5m6m + 4p5p6p + 7s8s9s + 東東 (13枚)
        // 待ち: 1m, 4m, 7m
        let mut counts = [0u8; 35];
        counts[2] = 1; // 2m
        counts[3] = 1; // 3m
        counts[4] = 1; // 4m
        counts[5] = 1; // 5m
        counts[6] = 1; // 6m
        counts[13] = 1; // 4p
        counts[14] = 1; // 5p
        counts[15] = 1; // 6p
        counts[25] = 1; // 7s
        counts[26] = 1; // 8s
        counts[27] = 1; // 9s
        counts[28] = 2; // 東東

        let acceptance = calculate_acceptance(&counts, 0, None);
        assert_eq!(acceptance.current_shanten, 0);
        assert_eq!(acceptance.tile_types_count, 3);

        let wait_tiles: Vec<TileName> = acceptance.waits.iter().map(|w| w.tile).collect();
        assert!(wait_tiles.contains(&TileName::OneM));
        assert!(wait_tiles.contains(&TileName::FourM));
        assert!(wait_tiles.contains(&TileName::SevenM));

        // 1m: 4枚, 4m: 手牌に1枚あるので3枚, 7m: 4枚 -> 計 4 + 3 + 4 = 11枚
        assert_eq!(acceptance.total_remaining, 11);
    }

    #[test]
    fn test_acceptance_chuuren_poutou_9_waits() {
        // 純正九蓮宝燈テンパイ: 1m1m1m 2m3m4m5m6m7m8m 9m9m9m (13枚)
        // 待ち牌: 1m 〜 9m の全9種！
        let mut counts = [0u8; 35];
        counts[1] = 3;
        counts[2] = 1;
        counts[3] = 1;
        counts[4] = 1;
        counts[5] = 1;
        counts[6] = 1;
        counts[7] = 1;
        counts[8] = 1;
        counts[9] = 3;

        let acceptance = calculate_acceptance(&counts, 0, None);
        assert_eq!(acceptance.current_shanten, 0);
        assert_eq!(acceptance.tile_types_count, 9);

        // 1m: 4 - 3 = 1枚
        // 2m..8m: 4 - 1 = 3枚 each (3 * 7 = 21枚)
        // 9m: 4 - 3 = 1枚
        // 合計: 1 + 21 + 1 = 23枚！
        assert_eq!(acceptance.total_remaining, 23);

        for (idx, &t) in [
            TileName::OneM,
            TileName::TwoM,
            TileName::ThreeM,
            TileName::FourM,
            TileName::FiveM,
            TileName::SixM,
            TileName::SevenM,
            TileName::EightM,
            TileName::NineM,
        ]
        .iter()
        .enumerate()
        {
            let wait = acceptance
                .waits
                .iter()
                .find(|w| w.tile == t)
                .expect("Must contain wait tile");
            let expected_rem = if idx == 0 || idx == 8 { 1 } else { 3 };
            assert_eq!(wait.remaining, expected_rem);
        }
    }

    #[test]
    fn test_acceptance_iishanten_ryamen_ryamen() {
        // 典型的な両面×両面の一向聴:
        // 1m2m3m (3枚) + 4p5p6p (3枚) + 東東 (2枚) + 2s3s (2枚) + 7s8s (2枚) + 西 (1枚) = 13枚
        // 2面子 (1m2m3m, 4p5p6p) + 1雀頭 (東東) + 2両面搭子 (2s3s, 7s8s) + 孤立牌 (西)
        // 有効牌: 1s, 4s, 6s, 9s (計4種16枚)
        let mut counts = [0u8; 35];
        counts[1] = 1; // 1m
        counts[2] = 1; // 2m
        counts[3] = 1; // 3m
        counts[13] = 1; // 4p
        counts[14] = 1; // 5p
        counts[15] = 1; // 6p
        counts[28] = 2; // 東東
        counts[20] = 1; // 2s
        counts[21] = 1; // 3s
        counts[25] = 1; // 7s
        counts[26] = 1; // 8s
        counts[30] = 1; // 西

        let acceptance = calculate_acceptance(&counts, 0, None);
        assert_eq!(acceptance.current_shanten, 1); // 一向聴

        let wait_tiles: Vec<TileName> = acceptance.waits.iter().map(|w| w.tile).collect();
        assert_eq!(acceptance.tile_types_count, 4);
        assert!(wait_tiles.contains(&TileName::OneS));
        assert!(wait_tiles.contains(&TileName::FourS));
        assert!(wait_tiles.contains(&TileName::SixS));
        assert!(wait_tiles.contains(&TileName::NineS));

        // 1s, 4s, 6s, 9s: 手牌に持っていないので各4枚 -> 計 16枚
        assert_eq!(acceptance.total_remaining, 16);
    }
}
