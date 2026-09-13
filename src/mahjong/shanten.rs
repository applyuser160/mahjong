use crate::hand::Hand;

/// 13種の公九牌のインデックス一覧（1m, 9m, 1p, 9p, 1s, 9s, 東, 南, 西, 北, 白, 発, 中）
pub const TERMINAL_AND_HONOR_INDICES: [usize; 13] = [
    1, 9,   // 1m, 9m
    10, 18, // 1p, 9p
    19, 27, // 1s, 9s
    28, 29, 30, 31, // 東, 南, 西, 北
    32, 33, 34,     // 白, 発, 中
];

/// 手牌の向聴数（シャンテン数）の内訳
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShantenResult {
    /// 最小向聴数（和了形は -1、テンパイは 0、一向聴は 1...）
    pub min_shanten: i8,
    /// 一般手（面子手）の向聴数
    pub normal: i8,
    /// 七対子の向聴数（副露がある場合は 99）
    pub chitoitsu: i8,
    /// 国士無双の向聴数（副露がある場合は 99）
    pub kokushi: i8,
}

/// Hand 構造体から最小向聴数を計算します。
pub fn calculate_shanten(hand: &Hand) -> ShantenResult {
    let open_melds_count = hand.open_melds.len();
    calculate_shanten_from_counts(&hand.counts, open_melds_count)
}

/// 牌カウント配列（1..=34）と副露面子数から向聴数を計算します。
pub fn calculate_shanten_from_counts(counts: &[u8; 35], open_melds_count: usize) -> ShantenResult {
    let is_closed = open_melds_count == 0;

    let normal = calculate_normal_shanten(counts, open_melds_count);
    let chitoitsu = if is_closed {
        calculate_chitoitsu_shanten(counts)
    } else {
        99
    };
    let kokushi = if is_closed {
        calculate_kokushi_shanten(counts)
    } else {
        99
    };

    let min_shanten = normal.min(chitoitsu).min(kokushi);

    ShantenResult {
        min_shanten,
        normal,
        chitoitsu,
        kokushi,
    }
}

/// 七対子の向聴数を計算します。
/// 7種類の対子が必要。同一牌が4枚ある場合は1組のみカウント可能。
pub fn calculate_chitoitsu_shanten(counts: &[u8; 35]) -> i8 {
    let mut pairs = 0;
    let mut kinds = 0;

    for i in 1..=34 {
        let c = counts[i];
        if c >= 2 {
            pairs += 1;
            kinds += 1;
        } else if c == 1 {
            kinds += 1;
        }
    }

    // 基本式: 6 - (対子数) + 種類数が7未満の場合のペナルティ
    let mut shanten = 6 - pairs;
    if kinds < 7 {
        shanten += 7 - kinds;
    }
    shanten
}

/// 国士無双の向聴数を計算します。
/// 13種の公九牌が必要。1種以上が2枚（雀頭）あればさらに1歩前進。
pub fn calculate_kokushi_shanten(counts: &[u8; 35]) -> i8 {
    let mut kinds = 0;
    let mut has_pair = false;

    for &idx in &TERMINAL_AND_HONOR_INDICES {
        let c = counts[idx];
        if c >= 1 {
            kinds += 1;
        }
        if c >= 2 {
            has_pair = true;
        }
    }

    // 基本式: 13 - (公九牌の種類数) - (雀頭があれば1)
    13 - kinds - if has_pair { 1 } else { 0 }
}

/// 一般手（4面子1雀頭）の向聴数を計算します。
/// 和了形は -1、テンパイは 0。
pub fn calculate_normal_shanten(counts: &[u8; 35], open_melds_count: usize) -> i8 {
    let target_melds = 4 - open_melds_count;
    let mut best_shanten = 8 - 2 * open_melds_count as i8;

    let mut working = *counts;

    // 1. 雀頭ありのケースを探索（対子を1つ雀頭として固定する）
    for i in 1..=34 {
        if working[i] >= 2 {
            working[i] -= 2;
            let shanten = search_normal(&mut working, 1, true, 0, 0, target_melds);
            best_shanten = best_shanten.min(shanten);
            working[i] += 2;
        }
    }

    // 2. 雀頭なしのケースを探索（搭子・面子のみで構成）
    let shanten = search_normal(&mut working, 1, false, 0, 0, target_melds);
    best_shanten = best_shanten.min(shanten);

    best_shanten
}

/// 一般手の再帰的探索（深さ優先探索）
fn search_normal(
    counts: &mut [u8; 35],
    index: usize,
    has_head: bool,
    melds: usize,
    taatsu: usize,
    target_melds: usize,
) -> i8 {
    // 次の牌があるインデックスを探す
    let mut next_idx = index;
    while next_idx <= 34 && counts[next_idx] == 0 {
        next_idx += 1;
    }

    // すべての牌を走査し終えた場合、シャンテン数を算出
    if next_idx > 34 {
        return evaluate_normal_shanten(has_head, melds, taatsu, target_melds);
    }

    let mut min_shanten = 8;
    let i = next_idx;

    // --- A. 字牌 (28..=34): 順子・搭子を作れないため、刻子のみ ---
    if i >= 28 {
        if counts[i] >= 3 {
            counts[i] -= 3;
            let s = search_normal(counts, i, has_head, melds + 1, taatsu, target_melds);
            min_shanten = min_shanten.min(s);
            counts[i] += 3;
        }
        // 刻子にしない場合はスキップして次へ
        let s = search_normal(counts, i + 1, has_head, melds, taatsu, target_melds);
        return min_shanten.min(s);
    }

    // --- B. 数牌 (萬子 1..=9, 筒子 10..=18, 索子 19..=27) ---
    let rank = (i - 1) % 9 + 1; // 1..=9

    // 1. 刻子 (AAA)
    if counts[i] >= 3 {
        counts[i] -= 3;
        let s = search_normal(counts, i, has_head, melds + 1, taatsu, target_melds);
        min_shanten = min_shanten.min(s);
        counts[i] += 3;
    }

    // 2. 順子 (ABC)
    if rank <= 7 && counts[i + 1] > 0 && counts[i + 2] > 0 {
        counts[i] -= 1;
        counts[i + 1] -= 1;
        counts[i + 2] -= 1;
        let s = search_normal(counts, i, has_head, melds + 1, taatsu, target_melds);
        min_shanten = min_shanten.min(s);
        counts[i] += 1;
        counts[i + 1] += 1;
        counts[i + 2] += 1;
    }

    // 3. 対子 (AA) - 雀頭が既に決まっている場合の搭子候補
    if counts[i] >= 2 {
        counts[i] -= 2;
        let s = search_normal(counts, i, has_head, melds, taatsu + 1, target_melds);
        min_shanten = min_shanten.min(s);
        counts[i] += 2;
    }

    // 4. 両面・辺張搭子 (AB)
    if rank <= 8 && counts[i + 1] > 0 {
        counts[i] -= 1;
        counts[i + 1] -= 1;
        let s = search_normal(counts, i, has_head, melds, taatsu + 1, target_melds);
        min_shanten = min_shanten.min(s);
        counts[i] += 1;
        counts[i + 1] += 1;
    }

    // 5. 嵌張搭子 (AC)
    if rank <= 7 && counts[i + 2] > 0 {
        counts[i] -= 1;
        counts[i + 2] -= 1;
        let s = search_normal(counts, i, has_head, melds, taatsu + 1, target_melds);
        min_shanten = min_shanten.min(s);
        counts[i] += 1;
        counts[i + 2] += 1;
    }

    // 6. この牌を孤立牌として残し、次の牌の走査へ進む
    let s = search_normal(counts, i + 1, has_head, melds, taatsu, target_melds);
    min_shanten.min(s)
}

/// 面子数・搭子数・雀頭の有無からシャンテン数を評価
fn evaluate_normal_shanten(
    has_head: bool,
    melds: usize,
    taatsu: usize,
    target_melds: usize,
) -> i8 {
    // 面子と搭子の合計が目標面子数を超えないよう搭子を制限
    let max_taatsu = target_melds.saturating_sub(melds);
    let valid_taatsu = taatsu.min(max_taatsu);

    // 基本向聴数: (目標面子数 * 2) - (面子数 * 2) - 有効搭子数 - (雀頭があれば1)
    let base = (target_melds as i8) * 2;
    let shanten = base - (melds as i8) * 2 - (valid_taatsu as i8) - if has_head { 1 } else { 0 };

    shanten
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tile::TileName;

    #[test]
    fn test_agari_normal() {
        // 1m2m3m 4p5p6p 7s8s9s 東東東 白白 (和了形: -1)
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
            TileName::East,
            TileName::White,
            TileName::White,
        ] {
            hand.push(t);
        }
        let res = calculate_shanten(&hand);
        assert_eq!(res.normal, -1);
        assert_eq!(res.min_shanten, -1);
    }

    #[test]
    fn test_tenpai_normal() {
        // 1m2m3m 4p5p6p 7s8s9s 東東東 白 (白単騎テンパイ: 0)
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
            TileName::East,
            TileName::White,
        ] {
            hand.push(t);
        }
        let res = calculate_shanten(&hand);
        assert_eq!(res.normal, 0);
        assert_eq!(res.min_shanten, 0);
    }

    #[test]
    fn test_ii_shanten_normal() {
        // 1m2m3m 4p5p6p 7s8s 東東東 白 (一向聴: 1)
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
            TileName::East,
            TileName::East,
            TileName::East,
            TileName::White,
            TileName::South,
        ] {
            hand.push(t);
        }
        let res = calculate_shanten(&hand);
        assert_eq!(res.normal, 1);
        assert_eq!(res.min_shanten, 1);
    }

    #[test]
    fn test_chitoitsu() {
        // 七対子テンパイ (6対子 + 1枚): 0向聴
        let mut hand = Hand::new();
        for &t in &[
            TileName::OneM,
            TileName::OneM,
            TileName::ThreeM,
            TileName::ThreeM,
            TileName::FiveP,
            TileName::FiveP,
            TileName::SevenP,
            TileName::SevenP,
            TileName::NineS,
            TileName::NineS,
            TileName::East,
            TileName::East,
            TileName::White,
        ] {
            hand.push(t);
        }
        let res = calculate_shanten(&hand);
        assert_eq!(res.chitoitsu, 0);
        assert_eq!(res.min_shanten, 0);

        // 七対子和了形 (-1向聴)
        hand.push(TileName::White);
        let res_agari = calculate_shanten(&hand);
        assert_eq!(res_agari.chitoitsu, -1);
        assert_eq!(res_agari.min_shanten, -1);
    }

    #[test]
    fn test_kokushi() {
        // 国士無双テンパイ (13面待ち): 0向聴
        let mut hand = Hand::new();
        for &t in &[
            TileName::OneM,
            TileName::NineM,
            TileName::OneP,
            TileName::NineP,
            TileName::OneS,
            TileName::NineS,
            TileName::East,
            TileName::South,
            TileName::West,
            TileName::North,
            TileName::White,
            TileName::Green,
            TileName::Red,
        ] {
            hand.push(t);
        }
        let res = calculate_shanten(&hand);
        assert_eq!(res.kokushi, 0);
        assert_eq!(res.min_shanten, 0);

        // 国士無双和了形 (-1向聴)
        hand.push(TileName::OneM);
        let res_agari = calculate_shanten(&hand);
        assert_eq!(res_agari.kokushi, -1);
        assert_eq!(res_agari.min_shanten, -1);
    }

    #[test]
    fn test_open_melds() {
        // チーした状態でのシャンテン数計算
        let mut hand = Hand::new();
        // 手牌10枚
        for &t in &[
            TileName::OneM,
            TileName::TwoM,
            TileName::ThreeM,
            TileName::FourP,
            TileName::FiveP,
            TileName::SixP,
            TileName::SevenS,
            TileName::EightS,
            TileName::East,
            TileName::East,
        ] {
            hand.push(t);
        }
        // 1面子副露
        hand.open_melds.push(crate::hand::Meld::Pon(TileName::White));

        let res = calculate_shanten(&hand);
        assert_eq!(res.normal, 0); // テンパイ (7s-8s の両面待ち)
        assert_eq!(res.chitoitsu, 99); // 鳴きがあるので七対子は不可
        assert_eq!(res.kokushi, 99); // 鳴きがあるので国士は不可
        assert_eq!(res.min_shanten, 0);
    }
}
