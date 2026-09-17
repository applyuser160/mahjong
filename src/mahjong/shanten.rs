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

use crate::suit_table::{encode_suit_key, get_suit_table, SuitEntry};

/// 5の累乗定数配列（0..=8）
pub const POW5: [usize; 9] = [1, 5, 25, 125, 625, 3125, 15625, 78125, 390625];

/// 么九牌判定
#[inline(always)]
pub fn is_terminal_or_honor(tile_idx: usize) -> bool {
    matches!(
        tile_idx,
        1 | 9 | 10 | 18 | 19 | 27 | 28 | 29 | 30 | 31 | 32 | 33 | 34
    )
}

/// 各スーツの SuitEntry と字牌集計から通常形向聴数を計算
pub fn eval_normal_shanten_from_suits(
    suits: &[SuitEntry; 3],
    z_melds: usize,
    z_pairs: usize,
    open_melds_count: usize,
) -> i8 {
    let target_melds = 4 - open_melds_count;
    let mut best_shanten = 8 - 2 * open_melds_count as i8;

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

/// 一般手（4面子1雀頭）の向聴数を計算します。
/// 和了形は -1、テンパイは 0。
pub fn calculate_normal_shanten(counts: &[u8; 35], open_melds_count: usize) -> i8 {
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

    eval_normal_shanten_from_suits(
        &[m_entry, p_entry, s_entry],
        z_melds,
        z_pairs,
        open_melds_count,
    )
}

/// 手牌状態をキャッシュし、仮ツモによる向聴数を O(1) で差分計算する構造体
#[derive(Clone, Debug)]
pub struct ShantenState<'a> {
    pub counts: &'a [u8; 35],
    pub open_melds_count: usize,
    pub is_closed: bool,

    pub suit_keys: [usize; 3],
    pub suit_entries: [SuitEntry; 3],
    pub z_melds: usize,
    pub z_pairs: usize,

    pub chitoitsu_pairs: i8,
    pub chitoitsu_kinds: i8,

    pub kokushi_kinds: i8,
    pub kokushi_has_pair: bool,

    pub current_result: ShantenResult,
}

impl<'a> ShantenState<'a> {
    pub fn new(counts: &'a [u8; 35], open_melds_count: usize) -> Self {
        let is_closed = open_melds_count == 0;
        let table = get_suit_table();

        let m_key = encode_suit_key(&counts[1..=9]);
        let p_key = encode_suit_key(&counts[10..=18]);
        let s_key = encode_suit_key(&counts[19..=27]);

        let suit_keys = [m_key, p_key, s_key];
        let suit_entries = [table[m_key], table[p_key], table[s_key]];

        let mut z_melds = 0;
        let mut z_pairs = 0;
        for &c in &counts[28..=34] {
            if c >= 3 {
                z_melds += 1;
            } else if c == 2 {
                z_pairs += 1;
            }
        }

        let normal =
            eval_normal_shanten_from_suits(&suit_entries, z_melds, z_pairs, open_melds_count);

        let mut chitoitsu_pairs = 0;
        let mut chitoitsu_kinds = 0;
        let chitoitsu = if is_closed {
            for &c in &counts[1..=34] {
                if c >= 2 {
                    chitoitsu_pairs += 1;
                    chitoitsu_kinds += 1;
                } else if c == 1 {
                    chitoitsu_kinds += 1;
                }
            }
            let mut s = 6 - chitoitsu_pairs;
            if chitoitsu_kinds < 7 {
                s += 7 - chitoitsu_kinds;
            }
            s
        } else {
            99
        };

        let mut kokushi_kinds = 0;
        let mut kokushi_has_pair = false;
        let kokushi = if is_closed {
            for &idx in &TERMINAL_AND_HONOR_INDICES {
                let c = counts[idx];
                if c >= 1 {
                    kokushi_kinds += 1;
                }
                if c >= 2 {
                    kokushi_has_pair = true;
                }
            }
            13 - kokushi_kinds - if kokushi_has_pair { 1 } else { 0 }
        } else {
            99
        };

        let min_shanten = normal.min(chitoitsu).min(kokushi);
        let current_result = ShantenResult {
            min_shanten,
            normal,
            chitoitsu,
            kokushi,
        };

        Self {
            counts,
            open_melds_count,
            is_closed,
            suit_keys,
            suit_entries,
            z_melds,
            z_pairs,
            chitoitsu_pairs,
            chitoitsu_kinds,
            kokushi_kinds,
            kokushi_has_pair,
            current_result,
        }
    }

    /// 牌 tile_idx を1枚仮ツモした後の向聴数を O(1) 差分計算
    #[inline]
    pub fn after_draw(&self, tile_idx: usize) -> ShantenResult {
        let c = self.counts[tile_idx];
        debug_assert!(c < 4);

        let table = get_suit_table();

        // 1. 通常形
        let normal = if tile_idx <= 27 {
            let suit = (tile_idx - 1) / 9;
            let pos = (tile_idx - 1) % 9;
            let new_key = self.suit_keys[suit] + POW5[pos];
            let new_entry = table[new_key];

            let mut suits = self.suit_entries;
            suits[suit] = new_entry;
            eval_normal_shanten_from_suits(
                &suits,
                self.z_melds,
                self.z_pairs,
                self.open_melds_count,
            )
        } else {
            let (new_z_melds, new_z_pairs) = match c {
                1 => (self.z_melds, self.z_pairs + 1),
                2 => (self.z_melds + 1, self.z_pairs.saturating_sub(1)),
                _ => (self.z_melds, self.z_pairs),
            };
            eval_normal_shanten_from_suits(
                &self.suit_entries,
                new_z_melds,
                new_z_pairs,
                self.open_melds_count,
            )
        };

        // 2. 七対子
        let chitoitsu = if self.is_closed {
            let (new_pairs, new_kinds) = match c {
                0 => (self.chitoitsu_pairs, self.chitoitsu_kinds + 1),
                1 => (self.chitoitsu_pairs + 1, self.chitoitsu_kinds),
                _ => (self.chitoitsu_pairs, self.chitoitsu_kinds),
            };
            let mut s = 6 - new_pairs;
            if new_kinds < 7 {
                s += 7 - new_kinds;
            }
            s
        } else {
            99
        };

        // 3. 国士無双
        let kokushi = if self.is_closed && is_terminal_or_honor(tile_idx) {
            let (new_kinds, new_pair) = match c {
                0 => (self.kokushi_kinds + 1, self.kokushi_has_pair),
                1 => (self.kokushi_kinds, true),
                _ => (self.kokushi_kinds, self.kokushi_has_pair),
            };
            13 - new_kinds - if new_pair { 1 } else { 0 }
        } else if self.is_closed {
            self.current_result.kokushi
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
