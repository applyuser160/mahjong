use crate::hand::Hand;

/// 13種の公九牌のインデックス一覧（1m, 9m, 1p, 9p, 1s, 9s, 東, 南, 西, 北, 白, 発, 中）
pub const TERMINAL_AND_HONOR_INDICES: [usize; 13] = [
    1, 9, // 1m, 9m
    10, 18, // 1p, 9p
    19, 27, // 1s, 9s
    28, 29, 30, 31, // 東, 南, 西, 北
    32, 33, 34, // 白, 発, 中
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

/// 七対子の向聴数を計算します（スカラー実装）。
#[inline]
pub fn calculate_chitoitsu_shanten_scalar(counts: &[u8; 35]) -> i8 {
    let mut pairs = 0;
    let mut kinds = 0;

    for &c in &counts[1..=34] {
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

/// 七対子の向聴数を計算します（x86_64 AVX2 実装）。
///
/// # Safety
/// 呼び出し元で CPU の AVX2 命令セットサポートが保証されている必要があります。
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
pub unsafe fn calculate_chitoitsu_shanten_avx2(counts: &[u8; 35]) -> i8 {
    use std::arch::x86_64::*;

    // counts[1..=32] の 32 バイトを一括ロード (萬子9, 筒子9, 索子9, 東南西北白)
    // [u8; 35] のアライメントは 1 なので未整列ロード _mm256_loadu_si256 を使用
    let ptr = counts.as_ptr().add(1) as *const __m256i;
    let v = _mm256_loadu_si256(ptr);

    // c >= 1: 0 より大きい要素のバイトマスク (0xFF where c > 0)
    let zero = _mm256_setzero_si256();
    let mask_kinds = _mm256_cmpgt_epi8(v, zero);
    let kinds_bits = _mm256_movemask_epi8(mask_kinds) as u32;

    // c >= 2: 1 より大きい要素のバイトマスク (0xFF where c > 1)
    let one = _mm256_set1_epi8(1);
    let mask_pairs = _mm256_cmpgt_epi8(v, one);
    let pairs_bits = _mm256_movemask_epi8(mask_pairs) as u32;

    // POPCNT 命令により 1 サイクルでビット数を集計
    let mut kinds = kinds_bits.count_ones() as i8;
    let mut pairs = pairs_bits.count_ones() as i8;

    // 残り 2 要素 (33: 発, 34: 中) をスカラー加算
    let c33 = *counts.get_unchecked(33);
    let c34 = *counts.get_unchecked(34);

    kinds += (c33 >= 1) as i8 + (c34 >= 1) as i8;
    pairs += (c33 >= 2) as i8 + (c34 >= 2) as i8;

    let mut shanten = 6 - pairs;
    if kinds < 7 {
        shanten += 7 - kinds;
    }
    shanten
}

/// 七対子の向聴数を計算します。
/// 7種類の対子が必要。同一牌が4枚ある場合は1組のみカウント可能。
#[inline(always)]
pub fn calculate_chitoitsu_shanten(counts: &[u8; 35]) -> i8 {
    #[cfg(all(target_arch = "x86_64", target_feature = "avx2"))]
    {
        unsafe { calculate_chitoitsu_shanten_avx2(counts) }
    }
    #[cfg(all(target_arch = "x86_64", not(target_feature = "avx2")))]
    {
        if is_x86_feature_detected!("avx2") {
            unsafe { calculate_chitoitsu_shanten_avx2(counts) }
        } else {
            calculate_chitoitsu_shanten_scalar(counts)
        }
    }
    #[cfg(not(target_arch = "x86_64"))]
    {
        calculate_chitoitsu_shanten_scalar(counts)
    }
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

use crate::suit_table::{encode_suit_key, get_suit_table};

/// 一般手（4面子1雀頭）の向聴数を計算します。
/// 和了形は -1、テンパイは 0。
///
/// 各数牌スーツ（萬子・筒子・索子）の事前計算済みルックアップテーブル（LUT）を参照し、
/// 字牌の O(7) 簡易走査と雀頭候補の探索を行うことで O(1) で高速に算出します。
pub fn calculate_normal_shanten(counts: &[u8; 35], open_melds_count: usize) -> i8 {
    let target_melds = 4 - open_melds_count;
    let table = get_suit_table();
    let m_entry = table[encode_suit_key(&counts[1..=9])];
    let p_entry = table[encode_suit_key(&counts[10..=18])];
    let s_entry = table[encode_suit_key(&counts[19..=27])];

    // 字牌 (28..=34) の刻子・対子を集計
    let mut z_melds = 0;
    let mut z_pairs = 0;
    for &c in &counts[28..=34] {
        if c >= 3 {
            z_melds += 1;
        } else if c == 2 {
            z_pairs += 1;
        }
    }

    let mut best_shanten = 8 - 2 * open_melds_count as i8;
    let suits = [m_entry, p_entry, s_entry];

    // 雀頭候補の走査:
    // 0: 萬子に雀頭, 1: 筒子に雀頭, 2: 索子に雀頭,
    // 3: 字牌対子を雀頭, 4: 字牌刻子を崩して雀頭, 5: 雀頭なし
    for head_choice in 0..6 {
        let (has_head, cur_z_melds, cur_z_taatsu) = match head_choice {
            3 => {
                if z_pairs > 0 {
                    (true, z_melds, z_pairs - 1)
                } else {
                    continue;
                }
            }
            4 => {
                if z_melds > 0 {
                    (true, z_melds - 1, z_pairs)
                } else {
                    continue;
                }
            }
            5 => (false, z_melds, z_pairs),
            _ => (true, z_melds, z_pairs),
        };

        let mut suit_taatsu = [[-1i8; 5]; 3];
        let mut possible = true;
        for (s, taatsu) in suit_taatsu.iter_mut().enumerate() {
            if head_choice == s {
                *taatsu = suits[s].with_head;
                if taatsu.iter().all(|&t| t == -1) {
                    possible = false;
                    break;
                }
            } else {
                *taatsu = suits[s].no_head;
            }
        }
        if !possible {
            continue;
        }

        // 3スーツで作る面子数 (m0, m1, m2) の組み合わせを探索
        for m0 in 0..=4 {
            let t0 = suit_taatsu[0][m0];
            if t0 == -1 {
                continue;
            }
            for m1 in 0..=4 - m0 {
                let t1 = suit_taatsu[1][m1];
                if t1 == -1 {
                    continue;
                }
                let max_m2 = 4 - m0 - m1;
                for (m2, &t2) in suit_taatsu[2].iter().enumerate().take(max_m2 + 1) {
                    if t2 == -1 {
                        continue;
                    }

                    let total_melds = cur_z_melds + m0 + m1 + m2;
                    let total_taatsu = cur_z_taatsu + (t0 + t1 + t2) as usize;

                    let shanten =
                        evaluate_normal_shanten(has_head, total_melds, total_taatsu, target_melds);
                    if shanten < best_shanten {
                        best_shanten = shanten;
                        if best_shanten == -1 {
                            return -1;
                        }
                    }
                }
            }
        }
    }

    best_shanten
}
/// 面子数・搭子数・雀頭の有無からシャンテン数を評価
#[inline(always)]
fn evaluate_normal_shanten(has_head: bool, melds: usize, taatsu: usize, target_melds: usize) -> i8 {
    let max_taatsu = target_melds.saturating_sub(melds);
    let valid_taatsu = taatsu.min(max_taatsu);
    let base = (target_melds as i8) * 2;
    base - (melds as i8) * 2 - (valid_taatsu as i8) - if has_head { 1 } else { 0 }
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
        hand.open_melds
            .push(crate::hand::Meld::Pon(TileName::White));

        let res = calculate_shanten(&hand);
        assert_eq!(res.normal, 0); // テンパイ (7s-8s の両面待ち)
        assert_eq!(res.chitoitsu, 99); // 鳴きがあるので七対子は不可
        assert_eq!(res.kokushi, 99); // 鳴きがあるので国士は不可
        assert_eq!(res.min_shanten, 0);
    }

    #[test]
    fn test_chinitsu_chuuren_shanten() {
        // 1112345678999m (九蓮宝燈テンパイ: 0向聴)
        let mut hand = Hand::new();
        for &t in &[
            TileName::OneM,
            TileName::OneM,
            TileName::OneM,
            TileName::TwoM,
            TileName::ThreeM,
            TileName::FourM,
            TileName::FiveM,
            TileName::SixM,
            TileName::SevenM,
            TileName::EightM,
            TileName::NineM,
            TileName::NineM,
            TileName::NineM,
        ] {
            hand.push(t);
        }
        let res = calculate_shanten(&hand);
        assert_eq!(res.normal, 0);
        assert_eq!(res.min_shanten, 0);

        // 1mツモで和了形: -1向聴
        hand.push(TileName::OneM);
        let res_agari = calculate_shanten(&hand);
        assert_eq!(res_agari.normal, -1);
        assert_eq!(res_agari.min_shanten, -1);
    }

    #[test]
    fn test_honor_pairs_as_taatsu() {
        // 東東(雀頭) 白白(搭子) 1m2m(搭子) 4p5p(搭子) 7s8s(搭子) -> 一向聴 (1)
        let mut hand = Hand::new();
        for &t in &[
            TileName::East,
            TileName::East,
            TileName::White,
            TileName::White,
            TileName::OneM,
            TileName::TwoM,
            TileName::FourP,
            TileName::FiveP,
            TileName::SevenS,
            TileName::EightS,
            TileName::ThreeS,
            TileName::ThreeS,
            TileName::ThreeS,
        ] {
            hand.push(t);
        }
        // 3s3s3s(1面子), 東東(雀頭), 白白/1m2m/4p5p/7s8s(搭子4組 -> 有効3組)
        // 向聴数 = 8 - 2*1 - 3 - 1 = 2 (向聴数 2)
        let res = calculate_shanten(&hand);
        assert_eq!(res.normal, 2);
    }

    #[test]
    fn test_identical_tiles_overflow_does_not_panic() {
        // 同一牌が5枚以上の異常入力（例: 9mが5枚）でもパニック（out of bounds）せず安全に処理されること
        let mut counts = [0u8; 35];
        counts[TileName::NineM as usize] = 5;
        let res = calculate_shanten_from_counts(&counts, 0);
        // パニックせず結果が返ること
        assert!(res.min_shanten >= 0);
    }
}
